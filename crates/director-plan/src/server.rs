use axum::{
    extract::{Path, State, Multipart, DefaultBodyLimit},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::process::Command;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use tracing::{info, error};

use crate::types::{Ticket, Status, FrontendTicket, Artifacts};

#[derive(Clone)]
struct AppState {
    workspace_root: PathBuf,
}

pub async fn create_app(workspace_root: PathBuf) -> anyhow::Result<Router> {
    let assets_dir = workspace_root.join("assets");
    if !assets_dir.exists() {
        fs::create_dir_all(&assets_dir).await?;
    }

    // Ensure artifacts directory exists for serving visual diffs
    let artifacts_dir = workspace_root.join("target/public/artifacts");
    if !artifacts_dir.exists() {
        fs::create_dir_all(&artifacts_dir).await?;
    }

    let state = Arc::new(AppState {
        workspace_root: workspace_root.clone(),
    });

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any) // For dev purposes, allows frontend dev server
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/api/tickets", get(list_tickets))
        .route("/api/tickets/:id", get(get_ticket).patch(update_ticket))
        .route("/api/tickets/:id/verify", post(verify_ticket))
        .route("/api/assets", post(upload_asset).get(list_assets))
        .nest_service("/artifacts", ServeDir::new(workspace_root.join("target/public/artifacts")))
        .nest_service("/assets", ServeDir::new(workspace_root.join("assets")))
        // SPA Fallback for everything else to dist/
        .fallback_service(ServeDir::new(workspace_root.join("apps/director-studio/dist")).fallback(ServeFile::new(workspace_root.join("apps/director-studio/dist/index.html"))))
        .layer(cors)
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB limit for uploads
        .with_state(state);

    Ok(app)
}

pub async fn start_server(workspace_root: PathBuf) -> anyhow::Result<()> {
    // tracing_subscriber is initialized in main now
    let app = create_app(workspace_root).await?;

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// --- Helpers ---

async fn enrich_ticket_artifacts(ticket: &mut FrontendTicket, state: &AppState) {
    let artifacts_dir = state.workspace_root.join(format!("target/public/artifacts/{}", ticket.id));
    if artifacts_dir.exists() {
        let golden = artifacts_dir.join("golden.png");
        let actual = artifacts_dir.join("actual.png");
        let diff = artifacts_dir.join("diff.png");

        if golden.exists() && actual.exists() {
            ticket.artifacts = Some(Artifacts {
                before_image: format!("/artifacts/{}/golden.png", ticket.id),
                after_image: format!("/artifacts/{}/actual.png", ticket.id),
                diff_image: if diff.exists() { Some(format!("/artifacts/{}/diff.png", ticket.id)) } else { None },
            });
        }
    }
}

fn validate_id(id: &str) -> Result<(), AppError> {
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError(anyhow::anyhow!("Invalid ID format"), StatusCode::BAD_REQUEST));
    }
    // Prevent directory traversal
    if id.contains("..") || id.starts_with('/') {
        return Err(AppError(anyhow::anyhow!("Invalid ID format"), StatusCode::BAD_REQUEST));
    }
    Ok(())
}

async fn load_ticket_with_history(state: &AppState, id: &str) -> Result<Ticket, AppError> {
    let ticket_path = state.workspace_root.join(format!("plan/tickets/{}.toml", id));

    if !ticket_path.exists() {
        return Err(AppError(anyhow::anyhow!("Ticket not found"), StatusCode::NOT_FOUND));
    }

    let content = fs::read_to_string(&ticket_path).await?;
    let mut ticket: Ticket = toml_edit::de::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse ticket: {}", e))?;

    // Load history
    let history_path = state.workspace_root.join(format!("plan/history/{}.log", id));
    if history_path.exists() {
        if let Ok(history_content) = fs::read_to_string(&history_path).await {
            ticket.history.log = history_content.lines().map(String::from).collect();
        }
    }

    Ok(ticket)
}

// --- Handlers ---

#[tracing::instrument(skip(state))]
async fn list_tickets(State(state): State<Arc<AppState>>) -> Result<Json<Vec<FrontendTicket>>, AppError> {
    let tickets_dir = state.workspace_root.join("plan/tickets");
    let mut tickets = Vec::new();

    if tickets_dir.exists() {
        let mut entries = fs::read_dir(tickets_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "toml") {
                let content = fs::read_to_string(&path).await?;
                // Parse leniently or log errors
                match toml_edit::de::from_str::<Ticket>(&content) {
                    Ok(mut ticket) => {
                         // Load history
                         // Sanitize ticket ID from file content just in case, though file system list is safe-ish
                        if validate_id(&ticket.meta.id).is_ok() {
                            let history_path = state.workspace_root.join(format!("plan/history/{}.log", ticket.meta.id));
                            if history_path.exists() {
                                if let Ok(history_content) = fs::read_to_string(&history_path).await {
                                    ticket.history.log = history_content.lines().map(String::from).collect();
                                }
                            }
                        }
                        let mut ft = FrontendTicket::from(ticket);
                        enrich_ticket_artifacts(&mut ft, &state).await;
                        tickets.push(ft);
                    },
                    Err(e) => error!("Failed to parse ticket {:?}: {}", path, e),
                }
            }
        }
    }

    // Sort by ID
    tickets.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(Json(tickets))
}

#[tracing::instrument(skip(state))]
async fn get_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<FrontendTicket>, AppError> {
    validate_id(&id)?;
    let ticket = load_ticket_with_history(&state, &id).await?;
    let mut ft = FrontendTicket::from(ticket);
    enrich_ticket_artifacts(&mut ft, &state).await;
    Ok(Json(ft))
}

#[derive(Deserialize)]
struct UpdateTicketPayload {
    status: Option<Status>,
    owner: Option<String>,
}

#[tracing::instrument(skip(state, payload))]
async fn update_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTicketPayload>,
) -> Result<Json<FrontendTicket>, AppError> {
    validate_id(&id)?;

    let ticket_path = state.workspace_root.join(format!("plan/tickets/{}.toml", id));

    if !ticket_path.exists() {
        return Err(AppError(anyhow::anyhow!("Ticket not found"), StatusCode::NOT_FOUND));
    }

    let content = fs::read_to_string(&ticket_path).await?;
    let mut doc = content.parse::<toml_edit::DocumentMut>()
        .map_err(|e| anyhow::anyhow!("Failed to parse TOML: {}", e))?;

    if let Some(status) = payload.status {
        doc["meta"]["status"] = toml_edit::value(status.to_string());
    }

    if let Some(owner) = payload.owner {
        doc["meta"]["owner"] = toml_edit::value(owner);
    }

    fs::write(&ticket_path, doc.to_string()).await?;

    // Return the updated ticket using helper to ensure consistency
    let ticket = load_ticket_with_history(&state, &id).await?;
    let mut ft = FrontendTicket::from(ticket);
    enrich_ticket_artifacts(&mut ft, &state).await;

    Ok(Json(ft))
}

#[tracing::instrument(skip(state))]
async fn verify_ticket(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    validate_id(&id)?;

    // We don't need history for verification execution, but consistent loading is good.
    // However, verify reads raw TOML string to parse.
    // load_ticket_with_history is fine.
    let ticket = load_ticket_with_history(&state, &id).await?;

    let command_str = &ticket.verification.command;
    let parts: Vec<&str> = command_str.split_whitespace().collect();

    if parts.is_empty() {
        return Err(AppError(anyhow::anyhow!("Empty verification command"), StatusCode::BAD_REQUEST));
    }

    info!("Running verification for {}: {}", id, command_str);

    let output = if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["-Command", command_str])
            .current_dir(&state.workspace_root)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute command: {}", e))?
    } else {
        Command::new("sh")
            .args(["-c", command_str])
            .current_dir(&state.workspace_root)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute command: {}", e))?
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let target_artifact_dir = state.workspace_root.join(format!("target/public/artifacts/{}", id));

    // Always attempt to copy artifacts
    fs::create_dir_all(&target_artifact_dir).await?;

    // 1. Copy Golden Image
    if let Some(golden_path) = ticket.verification.golden_image {
            // Basic protection against golden path traversal
            if !golden_path.contains("..") && !golden_path.starts_with('/') {
                let source_golden = state.workspace_root.join(&golden_path);
                if source_golden.exists() {
                    if let Err(e) = fs::copy(&source_golden, target_artifact_dir.join("golden.png")).await {
                        error!("Failed to copy golden image: {}", e);
                    }
                }
            } else {
                error!("Invalid golden image path: {}", golden_path);
            }
    }

    // 2. Look for Actual/Diff images generated by the test.
    // We look in `target/artifacts/{id}` which is a reasonable convention,
    // or just `actual.png` in current dir (workspace root) if test output is local.
    // Assuming a convention here is necessary for "wiring".
    // Let's assume the test dumps `actual.png` and `diff.png` in `target/artifacts/{id}/`
    // OR we check the workspace root for `actual.png`.

    // Strategy: Check potential locations
    let potential_actuals = vec![
        state.workspace_root.join("actual.png"),
        state.workspace_root.join(format!("target/artifacts/{}/actual.png", id)),
    ];

    for src in potential_actuals {
        if src.exists() {
            if let Err(e) = fs::copy(&src, target_artifact_dir.join("actual.png")).await {
                error!("Failed to copy actual image: {}", e);
            }
            break;
        }
    }

    let potential_diffs = vec![
        state.workspace_root.join("diff.png"),
        state.workspace_root.join(format!("target/artifacts/{}/diff.png", id)),
    ];

    for src in potential_diffs {
            if src.exists() {
            if let Err(e) = fs::copy(&src, target_artifact_dir.join("diff.png")).await {
                error!("Failed to copy diff image: {}", e);
            }
            break;
        }
    }

    Ok(Json(json!({
        "success": output.status.success(),
        "stdout": stdout,
        "stderr": stderr,
        "artifacts_path": format!("/artifacts/{}", id)
    })))
}

#[tracing::instrument(skip(state, multipart))]
async fn upload_asset(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let assets_dir = state.workspace_root.join("assets");

    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| anyhow::anyhow!("Multipart error: {}", e))? {
        let file_name = if let Some(name) = field.file_name() {
            name.to_string()
        } else {
            continue;
        };

        let data = field.bytes().await.map_err(|e| anyhow::anyhow!("Read error: {}", e))?;

        // Sanitize filename
        let safe_name = PathBuf::from(&file_name)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown_file".to_string());

        let dest_path = assets_dir.join(&safe_name);
        fs::write(&dest_path, data).await?;

        uploaded_files.push(json!({
            "name": safe_name,
            "path": format!("assets/{}", safe_name),
            "url": format!("/assets/{}", safe_name)
        }));
    }

    Ok(Json(json!({ "uploaded": uploaded_files })))
}

async fn list_assets(State(state): State<Arc<AppState>>) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let assets_dir = state.workspace_root.join("assets");
    let mut assets = Vec::new();

    if assets_dir.exists() {
        let mut entries = fs::read_dir(assets_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Try to guess type
                    let mime = mime_guess::from_path(&path).first_or_octet_stream();
                    let asset_type = if mime.type_() == "image" {
                        "image"
                    } else if name.ends_with(".json") { // simplistic check for lottie/json
                        "lottie"
                    } else if mime.type_() == "font" || name.ends_with(".ttf") || name.ends_with(".otf") {
                        "font"
                    } else {
                        "other"
                    };

                    assets.push(json!({
                        "id": format!("A-{}", name), // Simple ID
                        "name": name,
                        "type": asset_type,
                        "path": format!("assets/{}", name),
                        "preview_url": if asset_type == "image" { Some(format!("/assets/{}", name)) } else { None },
                        "rust_id": format!("ASSET_{}", name.to_uppercase().replace(|c: char| !c.is_alphanumeric(), "_"))
                    }));
                }
            }
        }
    }

    Ok(Json(assets))
}

// --- Error Handling ---

struct AppError(anyhow::Error, StatusCode);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("API Error: {}", self.0);
        (
            self.1,
            Json(json!({
                "error": self.0.to_string()
            })),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}
