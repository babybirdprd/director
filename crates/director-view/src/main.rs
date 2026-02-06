//! Director View - Web-based preview server for Director scripts
//!
//! This crate provides a local web server that:
//! - Loads and executes Rhai scripts
//! - Renders frames on demand for preview
//! - Serves a React frontend for interactive editing
//! - Supports safe file read/write and export APIs
//!
//! ## Architecture
//!
//! The server uses an actor pattern with a dedicated Director thread:
//! - Main thread: Axum HTTP server handling requests
//! - Director thread: Owns the Director instance, processes render requests
//!
//! Communication happens via mpsc channels to avoid blocking the HTTP server.
//!
//! ## API
//! - `GET/POST /api/init`
//! - `GET /api/render`
//! - `GET /api/scenes`
//! - `GET/POST /api/file`
//! - `POST /api/export`
//! - `GET /api/health`

use axum::{
    extract::{Query, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use director_core::{
    director::Director,
    export::render_export,
    scripting::{self, MovieHandle},
    video_wrapper::{FFmpegDriver, RenderMode},
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use tempfile::NamedTempFile;
use tokio::sync::{mpsc, oneshot};
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

const DEFAULT_MAX_SCRIPT_BYTES: usize = 1_000_000;

#[derive(Clone, Debug, Serialize)]
struct ApiError {
    error: String,
    line: Option<usize>,
    column: Option<usize>,
    snippet: Option<String>,
}

impl ApiError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            error: message.into(),
            line: None,
            column: None,
            snippet: None,
        }
    }
}

fn error_response(status: StatusCode, err: ApiError) -> Response {
    (status, Json(err)).into_response()
}

// --- Actor Messages ---
enum DirectorMessage {
    /// Initialize from a file path
    InitFromPath {
        path: PathBuf,
        resp: oneshot::Sender<Result<InitResponse, ApiError>>,
    },
    /// Initialize from script content (writes to temp file)
    InitFromContent {
        content: String,
        resp: oneshot::Sender<Result<InitResponse, ApiError>>,
    },
    /// Render a single frame
    RenderFrame {
        time: f64,
        resp: oneshot::Sender<Result<Vec<u8>, ApiError>>,
    },
    /// Get scene information
    GetScenes {
        resp: oneshot::Sender<Result<Vec<SceneInfo>, ApiError>>,
    },
    /// Export current timeline to a video
    ExportVideo {
        output_path: PathBuf,
        resp: oneshot::Sender<Result<ExportResponse, ApiError>>,
    },
}

#[derive(Clone, Serialize)]
struct InitResponse {
    status: String,
    duration: f64,
}

#[derive(Clone, Serialize)]
struct SceneInfo {
    index: usize,
    #[serde(rename = "startTime")]
    start_time: f64,
    duration: f64,
    name: Option<String>,
}

#[derive(Clone, Serialize)]
struct ExportResponse {
    status: String,
    output: String,
}

#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<DirectorMessage>,
    allowed_roots: Arc<Vec<PathBuf>>,
    max_script_bytes: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    FFmpegDriver::ensure_available().ok();

    let allowed_roots = Arc::new(discover_allowed_roots());
    for root in allowed_roots.iter() {
        info!("Allowed filesystem root: {}", root.display());
    }

    // Channel for communicating with the Director thread
    let (tx, mut rx) = mpsc::channel::<DirectorMessage>(100);

    // Spawn Director Thread
    thread::spawn(move || {
        let loader = Arc::new(director_core::DefaultAssetLoader);
        // Initial Default Director
        let mut current_director_arc = Arc::new(Mutex::new(Director::new(
            1920,
            1080,
            30,
            loader.clone(),
            RenderMode::Preview,
            None,
        )));

        info!("Director thread started.");

        while let Some(msg) = rx.blocking_recv() {
            match msg {
                DirectorMessage::InitFromPath { path, resp } => {
                    let result = init_script_from_path(&path, &loader, &mut current_director_arc);
                    let _ = resp.send(result);
                }
                DirectorMessage::InitFromContent { content, resp } => {
                    // Write content to a unique temp file and execute.
                    let mut temp_file = match NamedTempFile::new() {
                        Ok(file) => file,
                        Err(e) => {
                            let _ = resp.send(Err(ApiError::new(format!(
                                "Failed to create temp file: {}",
                                e
                            ))));
                            continue;
                        }
                    };

                    if let Err(e) = temp_file.write_all(content.as_bytes()) {
                        let _ =
                            resp.send(Err(ApiError::new(format!("Failed to write temp file: {}", e))));
                        continue;
                    }

                    let result =
                        init_script_from_path(temp_file.path(), &loader, &mut current_director_arc);
                    let _ = resp.send(result);
                }
                DirectorMessage::RenderFrame { time, resp } => {
                    let mut d = match current_director_arc.lock() {
                        Ok(director) => director,
                        Err(_) => {
                            let _ = resp.send(Err(ApiError::new(
                                "Director state lock poisoned during render",
                            )));
                            continue;
                        }
                    };
                    let mut layout_engine = director_core::systems::layout::LayoutEngine::new();

                    // Update
                    d.update(time);

                    // Render to surface
                    let w = d.width as i32;
                    let h = d.height as i32;
                    let info = skia_safe::ImageInfo::new(
                        (w, h),
                        skia_safe::ColorType::RGBA8888,
                        skia_safe::AlphaType::Premul,
                        None,
                    );

                    if let Some(mut surface) = skia_safe::surfaces::raster(&info, None, None) {
                        surface.canvas().clear(skia_safe::Color::BLACK);
                        let mut transition = None;

                        if let Err(e) = director_core::systems::renderer::render_at_time(
                            &mut *d,
                            &mut layout_engine,
                            time,
                            surface.canvas(),
                            &mut transition,
                        ) {
                            let _ = resp.send(Err(ApiError::new(e.to_string())));
                            continue;
                        }

                        let data = match surface
                            .image_snapshot()
                            .encode(None, skia_safe::EncodedImageFormat::JPEG, 80)
                        {
                            Some(data) => data,
                            None => {
                                let _ = resp
                                    .send(Err(ApiError::new("Failed to encode preview image")));
                                continue;
                            }
                        };
                        let _ = resp.send(Ok(data.as_bytes().to_vec()));
                    } else {
                        let _ = resp.send(Err(ApiError::new("Failed to create render surface")));
                    }
                }
                DirectorMessage::GetScenes { resp } => {
                    let d = match current_director_arc.lock() {
                        Ok(director) => director,
                        Err(_) => {
                            let _ = resp.send(Err(ApiError::new(
                                "Director state lock poisoned while loading scenes",
                            )));
                            continue;
                        }
                    };

                    let scenes = d
                        .timeline
                        .iter()
                        .enumerate()
                        .map(|(i, item)| SceneInfo {
                            index: i,
                            start_time: item.start_time,
                            duration: item.duration,
                            name: item.name.clone(),
                        })
                        .collect::<Vec<_>>();
                    let _ = resp.send(Ok(scenes));
                }
                DirectorMessage::ExportVideo { output_path, resp } => {
                    let mut d = match current_director_arc.lock() {
                        Ok(director) => director,
                        Err(_) => {
                            let _ = resp.send(Err(ApiError::new(
                                "Director state lock poisoned during export",
                            )));
                            continue;
                        }
                    };

                    match render_export(&mut d, output_path.clone(), None, None) {
                        Ok(_) => {
                            let _ = resp.send(Ok(ExportResponse {
                                status: "Export complete".to_string(),
                                output: output_path.to_string_lossy().to_string(),
                            }));
                        }
                        Err(e) => {
                            let _ = resp.send(Err(ApiError::new(format!("Export failed: {}", e))));
                        }
                    }
                }
            }
        }
    });

    let app_state = AppState {
        tx,
        allowed_roots,
        max_script_bytes: DEFAULT_MAX_SCRIPT_BYTES,
    };

    let allow_origin = std::env::var("DIRECTOR_VIEW_ALLOW_ORIGIN")
        .ok()
        .and_then(|origin| origin.parse::<HeaderValue>().ok())
        .unwrap_or_else(|| HeaderValue::from_static("*"));

    let app = Router::new()
        // Legacy HTML endpoint
        .route("/", get(index_handler))
        // API endpoints
        .route("/api/init", get(init_handler_get).post(init_handler_post))
        .route("/api/render", get(render_handler))
        .route("/api/scenes", get(scenes_handler))
        .route("/api/file", get(file_handler).post(write_file_handler))
        .route("/api/export", post(export_handler))
        .route("/api/health", get(health_handler))
        // Static files
        .nest_service(
            "/static",
            tower_http::services::ServeDir::new("crates/director-view/static"),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(allow_origin)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .with_state(app_state);

    info!("Starting Director View server on http://localhost:3000");
    info!("Frontend dev server: cd crates/director-view/frontend && npm run dev");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Initialize a script from a file path
fn init_script_from_path(
    path: &Path,
    loader: &Arc<director_core::DefaultAssetLoader>,
    current_director_arc: &mut Arc<Mutex<Director>>,
) -> Result<InitResponse, ApiError> {
    let loader_clone = loader.clone();

    // Always start fresh for a new script load
    let fresh_director = Director::new(
        1920,
        1080,
        30,
        loader_clone.clone(),
        RenderMode::Preview,
        None,
    );
    *current_director_arc = Arc::new(Mutex::new(fresh_director));

    // Init Engine locally
    let mut engine = rhai::Engine::new();
    scripting::register_rhai_api(&mut engine, loader_clone);

    if !path.exists() {
        return Err(ApiError::new(format!("Script not found: {}", path.display())));
    }

    let script_source = std::fs::read_to_string(path).map_err(|e| {
        ApiError::new(format!(
            "Failed to read script file {}: {}",
            path.display(),
            e
        ))
    })?;

    let mut scope = rhai::Scope::new();

    info!("Evaluating script: {}", path.display());
    let res = engine.eval_file_with_scope::<rhai::Dynamic>(&mut scope, path.to_path_buf());

    match res {
        Ok(val) => {
            if val.is::<MovieHandle>() {
                let returned_movie = val.cast::<MovieHandle>();
                info!("Script loaded successfully. Swapping active Director.");
                *current_director_arc = returned_movie.director;
            } else {
                return Err(ApiError::new(
                    "Script must return a 'Movie' object (e.g. `let m = new_director(...); ... m`)",
                ));
            }

            // Calculate total duration
            let mut max_duration = 0.0;
            {
                let d = current_director_arc.lock().map_err(|_| {
                    ApiError::new("Director state lock poisoned while computing timeline duration")
                })?;
                for item in &d.timeline {
                    let end = item.start_time + item.duration;
                    if end > max_duration {
                        max_duration = end;
                    }
                }
            }
            if max_duration == 0.0 {
                max_duration = 10.0;
            } // Default

            Ok(InitResponse {
                status: "Initialized".to_string(),
                duration: max_duration,
            })
        }
        Err(e) => Err(format_rhai_error(e.as_ref(), &script_source)),
    }
}

fn format_rhai_error(err: &rhai::EvalAltResult, source: &str) -> ApiError {
    let message = err.to_string();
    let (line, column) = extract_line_column(&message);
    let snippet = line.and_then(|line_num| source.lines().nth(line_num.saturating_sub(1)))
        .map(|line_text| line_text.to_string());

    ApiError {
        error: message,
        line,
        column,
        snippet,
    }
}

fn extract_line_column(message: &str) -> (Option<usize>, Option<usize>) {
    let lower = message.to_lowercase();
    let line = parse_number_after(&lower, "line ")
        .or_else(|| parse_number_after(&lower, "line:"));
    let column = parse_number_after(&lower, "position ")
        .or_else(|| parse_number_after(&lower, "column "))
        .or_else(|| parse_number_after(&lower, "position:"))
        .or_else(|| parse_number_after(&lower, "column:"));
    (line, column)
}

fn parse_number_after(haystack: &str, marker: &str) -> Option<usize> {
    let idx = haystack.find(marker)?;
    let tail = &haystack[idx + marker.len()..];
    let digits: String = tail
        .chars()
        .skip_while(|ch| !ch.is_ascii_digit())
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn discover_allowed_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(canonical) = cwd.canonicalize() {
            roots.push(canonical);
        }
    }

    if let Ok(extra_roots) = std::env::var("DIRECTOR_VIEW_ALLOWED_ROOTS") {
        for raw_root in extra_roots.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            match PathBuf::from(raw_root).canonicalize() {
                Ok(path) => roots.push(path),
                Err(e) => warn!("Skipping invalid allowed root '{}': {}", raw_root, e),
            }
        }
    }

    roots.sort();
    roots.dedup();
    roots
}

fn resolve_read_path(raw_path: &str, allowed_roots: &[PathBuf]) -> Result<PathBuf, ApiError> {
    let absolute_path = to_absolute_path(raw_path, allowed_roots)?;
    let canonical = absolute_path.canonicalize().map_err(|e| {
        ApiError::new(format!(
            "Failed to resolve file path '{}': {}",
            absolute_path.display(),
            e
        ))
    })?;
    ensure_within_allowed_roots(&canonical, allowed_roots)?;
    if !canonical.is_file() {
        return Err(ApiError::new(format!(
            "Path is not a file: {}",
            canonical.display()
        )));
    }
    Ok(canonical)
}

fn resolve_write_path(raw_path: &str, allowed_roots: &[PathBuf]) -> Result<PathBuf, ApiError> {
    let absolute_path = to_absolute_path(raw_path, allowed_roots)?;
    let parent = absolute_path.parent().ok_or_else(|| {
        ApiError::new(format!(
            "Path has no parent directory: {}",
            absolute_path.display()
        ))
    })?;
    let canonical_parent = parent.canonicalize().map_err(|e| {
        ApiError::new(format!(
            "Failed to resolve parent directory '{}': {}",
            parent.display(),
            e
        ))
    })?;
    ensure_within_allowed_roots(&canonical_parent, allowed_roots)?;

    let file_name = absolute_path.file_name().ok_or_else(|| {
        ApiError::new(format!(
            "Output path is missing file name: {}",
            absolute_path.display()
        ))
    })?;
    Ok(canonical_parent.join(file_name))
}

fn to_absolute_path(raw_path: &str, allowed_roots: &[PathBuf]) -> Result<PathBuf, ApiError> {
    let trimmed = raw_path.trim();
    if trimmed.is_empty() {
        return Err(ApiError::new("Path cannot be empty"));
    }
    if allowed_roots.is_empty() {
        return Err(ApiError::new(
            "No filesystem roots configured; refusing file operation",
        ));
    }
    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(allowed_roots[0].join(path))
    }
}

fn ensure_within_allowed_roots(path: &Path, allowed_roots: &[PathBuf]) -> Result<(), ApiError> {
    if allowed_roots.iter().any(|root| path.starts_with(root)) {
        Ok(())
    } else {
        Err(ApiError::new(format!(
            "Access denied: '{}' is outside allowed roots",
            path.display()
        )))
    }
}

// --- Handlers ---

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

#[derive(Deserialize)]
struct InitParamsGet {
    script_path: String,
}

async fn init_handler_get(
    State(state): State<AppState>,
    Query(params): Query<InitParamsGet>,
) -> impl IntoResponse {
    let script_path = match resolve_read_path(&params.script_path, state.allowed_roots.as_ref()) {
        Ok(path) => path,
        Err(err) => return error_response(StatusCode::BAD_REQUEST, err),
    };

    let (tx, rx) = oneshot::channel();
    let _ = state
        .tx
        .send(DirectorMessage::InitFromPath {
            path: script_path,
            resp: tx,
        })
        .await;

    match rx.await {
        Ok(Ok(response)) => (StatusCode::OK, Json(response)).into_response(),
        Ok(Err(err)) => error_response(StatusCode::BAD_REQUEST, err),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::new("Director channel closed"),
        ),
    }
}

#[derive(Deserialize)]
struct InitParamsPost {
    script: String,
}

async fn init_handler_post(
    State(state): State<AppState>,
    Json(params): Json<InitParamsPost>,
) -> impl IntoResponse {
    if params.script.len() > state.max_script_bytes {
        return error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::new(format!(
                "Script payload too large (limit: {} bytes)",
                state.max_script_bytes
            )),
        );
    }

    let (tx, rx) = oneshot::channel();
    let _ = state
        .tx
        .send(DirectorMessage::InitFromContent {
            content: params.script,
            resp: tx,
        })
        .await;

    match rx.await {
        Ok(Ok(response)) => (StatusCode::OK, Json(response)).into_response(),
        Ok(Err(err)) => error_response(StatusCode::BAD_REQUEST, err),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::new("Director channel closed"),
        ),
    }
}

#[derive(Deserialize)]
struct RenderParams {
    time: f64,
}

async fn render_handler(
    State(state): State<AppState>,
    Query(params): Query<RenderParams>,
) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let _ = state
        .tx
        .send(DirectorMessage::RenderFrame {
            time: params.time,
            resp: tx,
        })
        .await;

    match rx.await {
        Ok(Ok(bytes)) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/jpeg")],
            bytes,
        )
            .into_response(),
        Ok(Err(err)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::new("Director channel closed"),
        ),
    }
}

async fn scenes_handler(State(state): State<AppState>) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let _ = state.tx.send(DirectorMessage::GetScenes { resp: tx }).await;

    match rx.await {
        Ok(Ok(scenes)) => Json(scenes).into_response(),
        Ok(Err(err)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::new("Director channel closed"),
        ),
    }
}

#[derive(Deserialize)]
struct FileParams {
    path: String,
}

async fn file_handler(
    State(state): State<AppState>,
    Query(params): Query<FileParams>,
) -> impl IntoResponse {
    let resolved = match resolve_read_path(&params.path, state.allowed_roots.as_ref()) {
        Ok(path) => path,
        Err(err) => return error_response(StatusCode::BAD_REQUEST, err),
    };

    match std::fs::read_to_string(&resolved) {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(e) => error_response(
            StatusCode::NOT_FOUND,
            ApiError::new(format!("Failed to read file '{}': {}", resolved.display(), e)),
        ),
    }
}

#[derive(Deserialize)]
struct WriteFileRequest {
    path: String,
    content: String,
}

async fn write_file_handler(
    State(state): State<AppState>,
    Json(params): Json<WriteFileRequest>,
) -> impl IntoResponse {
    let resolved = match resolve_write_path(&params.path, state.allowed_roots.as_ref()) {
        Ok(path) => path,
        Err(err) => return error_response(StatusCode::BAD_REQUEST, err),
    };

    match std::fs::write(&resolved, params.content) {
        Ok(_) => Json(serde_json::json!({
            "status": "saved",
            "path": resolved.to_string_lossy()
        }))
        .into_response(),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::new(format!("Failed to write file '{}': {}", resolved.display(), e)),
        ),
    }
}

#[derive(Deserialize)]
struct ExportRequest {
    output: String,
}

async fn export_handler(
    State(state): State<AppState>,
    Json(params): Json<ExportRequest>,
) -> impl IntoResponse {
    let output_path = match resolve_write_path(&params.output, state.allowed_roots.as_ref()) {
        Ok(path) => path,
        Err(err) => return error_response(StatusCode::BAD_REQUEST, err),
    };

    let (tx, rx) = oneshot::channel();
    let _ = state
        .tx
        .send(DirectorMessage::ExportVideo {
            output_path,
            resp: tx,
        })
        .await;

    match rx.await {
        Ok(Ok(response)) => Json(response).into_response(),
        Ok(Err(err)) => error_response(StatusCode::BAD_REQUEST, err),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::new("Director channel closed"),
        ),
    }
}

async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn read_path_is_restricted_to_allowed_roots() {
        let root = tempdir().expect("temp dir");
        let inside = root.path().join("inside.rhai");
        std::fs::write(&inside, "movie").expect("write inside");

        let outside_root = tempdir().expect("outside temp");
        let outside = outside_root.path().join("outside.rhai");
        std::fs::write(&outside, "movie").expect("write outside");

        let roots = vec![root.path().canonicalize().expect("canonical root")];
        assert!(resolve_read_path(inside.to_str().expect("inside str"), &roots).is_ok());
        assert!(resolve_read_path(outside.to_str().expect("outside str"), &roots).is_err());
    }

    #[test]
    fn write_path_requires_allowed_parent() {
        let root = tempdir().expect("temp dir");
        let nested = root.path().join("nested");
        std::fs::create_dir_all(&nested).expect("create nested");

        let valid = nested.join("script.rhai");
        let roots = vec![root.path().canonicalize().expect("canonical root")];
        assert!(resolve_write_path(valid.to_str().expect("valid path"), &roots).is_ok());

        let outside_root = tempdir().expect("outside temp");
        let outside_parent = outside_root.path().join("other");
        std::fs::create_dir_all(&outside_parent).expect("outside parent");
        let outside = outside_parent.join("script.rhai");
        assert!(resolve_write_path(outside.to_str().expect("outside path"), &roots).is_err());
    }
}
