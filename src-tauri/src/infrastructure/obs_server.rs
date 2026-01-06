use arc_swap::ArcSwap;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use log::{error, info};
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

use crate::usecase::monitor::MonitorSharedState;

#[derive(RustEmbed)]
#[folder = "../src-ui/dist"]
struct Assets;

pub struct ObsServer {
    shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
}

impl ObsServer {
    pub fn new() -> Self {
        Self {
            shutdown_tx: Mutex::new(None),
        }
    }

    pub async fn start(
        &self,
        port: u16,
        state: Arc<ArcSwap<MonitorSharedState>>,
    ) -> Result<(), String> {
        let mut shutdown_guard = self.shutdown_tx.lock().await; // Async lock
        if shutdown_guard.is_some() {
            return Err("OBS Server is already running".to_string());
        }

        let (tx, rx) = oneshot::channel();
        *shutdown_guard = Some(tx);

        let app = Router::new()
            .route("/api/stats", get(stats_handler))
            .route("/", get(index_handler))
            .route("/{*file}", get(static_handler))
            .layer(tower_http::cors::CorsLayer::permissive())
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        info!("Starting OBS Server on {}", addr);

        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                *shutdown_guard = None; // Reset state on failure
                return Err(format!("Failed to bind to {}: {}", addr, e));
            }
        };

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    rx.await.ok();
                })
                .await
            {
                error!("OBS Server error: {}", e);
            }
            info!("OBS Server stopped");
        });

        Ok(())
    }

    pub async fn stop(&self) {
        let mut shutdown_guard = self.shutdown_tx.lock().await;
        if let Some(tx) = shutdown_guard.take() {
            let _ = tx.send(());
            info!("OBS Server stop signal sent");
        }
    }

    pub async fn is_running(&self) -> bool {
        self.shutdown_tx.lock().await.is_some()
    }
}

async fn index_handler() -> impl IntoResponse {
    serve_asset("overlay.html")
}

async fn static_handler(Path(file): Path<String>) -> impl IntoResponse {
    serve_asset(&file)
}

fn serve_asset(file: &str) -> Response {
    let file_path = if file.starts_with('/') {
        file.trim_start_matches('/')
    } else {
        file
    };

    match Assets::get(file_path) {
        Some(content) => {
            let mime_type = mime_guess::from_path(file_path)
                .first_or_octet_stream()
                .to_string();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type)
                .body(Body::from(content.data))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404 Not Found"))
            .unwrap(),
    }
}

async fn stats_handler(State(state): State<Arc<ArcSwap<MonitorSharedState>>>) -> impl IntoResponse {
    let snap = state.load();
    // Return the specific fields needed or the whole state
    // For now, return the whole state as requested in design
    Json((**snap).clone())
}
