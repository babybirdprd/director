//! Director View - Web-based preview server for Director scripts
//!
//! This crate provides a local web server that:
//! - Loads and executes Rhai scripts
//! - Renders frames on demand for preview
//! - Serves a React frontend for interactive editing
//!
//! ## Architecture
//!
//! The server uses an actor pattern with a dedicated Director thread:
//! - Main thread: Axum HTTP server handling requests
//! - Director thread: Owns the Director instance, processes render requests
//!
//! Communication happens via mpsc channels to avoid blocking the HTTP server.

use axum::{
    extract::{Query, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use director_core::{
    director::Director,
    scripting::{self, MovieHandle},
    video_wrapper::{FFmpegDriver, RenderMode},
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::{mpsc, oneshot};
use tower_http::cors::CorsLayer;
use tracing::info;

// --- Actor Messages ---
enum DirectorMessage {
    /// Initialize from a file path
    InitFromPath {
        path: String,
        resp: oneshot::Sender<Result<InitResponse, String>>,
    },
    /// Initialize from script content (writes to temp file)
    InitFromContent {
        content: String,
        resp: oneshot::Sender<Result<InitResponse, String>>,
    },
    /// Render a single frame
    RenderFrame {
        time: f64,
        resp: oneshot::Sender<Result<Vec<u8>, String>>,
    },
    /// Get scene information
    GetScenes {
        resp: oneshot::Sender<Vec<SceneInfo>>,
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

#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<DirectorMessage>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    FFmpegDriver::ensure_available().ok();

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
                    // Write content to a temp file and execute
                    let temp_path = std::env::temp_dir().join("director_temp_script.rhai");
                    match std::fs::File::create(&temp_path) {
                        Ok(mut file) => {
                            if let Err(e) = file.write_all(content.as_bytes()) {
                                let _ = resp.send(Err(format!("Failed to write temp file: {}", e)));
                                continue;
                            }
                            let result = init_script_from_path(
                                temp_path.to_str().unwrap(),
                                &loader,
                                &mut current_director_arc,
                            );
                            let _ = resp.send(result);
                        }
                        Err(e) => {
                            let _ = resp.send(Err(format!("Failed to create temp file: {}", e)));
                        }
                    }
                }
                DirectorMessage::RenderFrame { time, resp } => {
                    let mut d = current_director_arc.lock().unwrap();
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
                            let _ = resp.send(Err(e.to_string()));
                            continue;
                        }

                        let data = surface
                            .image_snapshot()
                            .encode(None, skia_safe::EncodedImageFormat::JPEG, 80)
                            .unwrap();
                        let _ = resp.send(Ok(data.as_bytes().to_vec()));
                    } else {
                        let _ = resp.send(Err("Failed to create surface".into()));
                    }
                }
                DirectorMessage::GetScenes { resp } => {
                    let d = current_director_arc.lock().unwrap();
                    let scenes: Vec<SceneInfo> = d
                        .timeline
                        .iter()
                        .enumerate()
                        .map(|(i, item)| SceneInfo {
                            index: i,
                            start_time: item.start_time,
                            duration: item.duration,
                            name: None, // TODO: Add scene names to TimelineItem
                        })
                        .collect();
                    let _ = resp.send(scenes);
                }
            }
        }
    });

    let app_state = AppState { tx };

    let app = Router::new()
        // Legacy HTML endpoint
        .route("/", get(index_handler))
        // API endpoints
        .route("/init", get(init_handler_get).post(init_handler_post))
        .route("/render", get(render_handler))
        .route("/scenes", get(scenes_handler))
        .route("/file", get(file_handler))
        .route("/health", get(health_handler))
        // Static files
        .nest_service(
            "/static",
            tower_http::services::ServeDir::new("crates/director-view/static"),
        )
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .with_state(app_state);

    info!("Starting Director View server on http://localhost:5173");
    info!("Frontend dev server: cd crates/director-view/frontend && npm run dev");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Initialize a script from a file path
fn init_script_from_path(
    path: &str,
    loader: &Arc<director_core::DefaultAssetLoader>,
    current_director_arc: &mut Arc<Mutex<Director>>,
) -> Result<InitResponse, String> {
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

    let script_path = std::path::Path::new(path);
    if !script_path.exists() {
        return Err(format!("Script not found: {:?}", script_path));
    }

    let mut scope = rhai::Scope::new();

    info!("Evaluating script: {:?}", script_path);
    let res = engine.eval_file_with_scope::<rhai::Dynamic>(&mut scope, script_path.to_path_buf());

    match res {
        Ok(val) => {
            if val.is::<MovieHandle>() {
                let returned_movie = val.cast::<MovieHandle>();
                info!("Script loaded successfully. Swapping active Director.");
                *current_director_arc = returned_movie.director;
            } else {
                return Err(
                    "Script must return a 'Movie' object (e.g. `let m = new_director(...); ... m`)"
                        .to_string(),
                );
            }

            // Calculate total duration
            let mut max_duration = 0.0;
            {
                let d = current_director_arc.lock().unwrap();
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
        Err(e) => Err(e.to_string()),
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
    let (tx, rx) = oneshot::channel();
    let _ = state
        .tx
        .send(DirectorMessage::InitFromPath {
            path: params.script_path,
            resp: tx,
        })
        .await;

    match rx.await {
        Ok(Ok(response)) => (StatusCode::OK, Json(response)).into_response(),
        Ok(Err(e)) => (StatusCode::BAD_REQUEST, e).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Channel closed").into_response(),
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
        Ok(Err(e)) => (StatusCode::BAD_REQUEST, e).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Channel closed").into_response(),
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
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Channel closed").into_response(),
    }
}

async fn scenes_handler(State(state): State<AppState>) -> impl IntoResponse {
    let (tx, rx) = oneshot::channel();
    let _ = state.tx.send(DirectorMessage::GetScenes { resp: tx }).await;

    match rx.await {
        Ok(scenes) => Json(scenes).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Channel closed").into_response(),
    }
}

#[derive(Deserialize)]
struct FileParams {
    path: String,
}

async fn file_handler(Query(params): Query<FileParams>) -> impl IntoResponse {
    match std::fs::read_to_string(&params.path) {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, format!("Failed to read file: {}", e)).into_response(),
    }
}

async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}
