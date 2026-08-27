//! Lightweight local HTTP server at 127.0.0.1:9876 for IDE/terminal event hooks.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{Method, StatusCode},
    routing::{get, post},
    Json, Router,
};
use tauri::{AppHandle, Emitter};
use tower_http::cors::{Any, CorsLayer};

use crate::models::{CooldownEventPayload, DevEvent, ErrorCategory};
use crate::state::AppState;

const LISTEN_ADDR: &str = "127.0.0.1:9876";

struct HttpContext {
    state: Arc<AppState>,
    app: AppHandle,
}

pub fn start(state: Arc<AppState>, app: AppHandle) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(1)
            .thread_name("cooldown-http")
            .build()
            .expect("failed to build tokio runtime");

        rt.block_on(async {
            if let Err(e) = run_server(state, app).await {
                eprintln!("[cooldown] HTTP server error: {e}");
            }
        });
    });
}

async fn run_server(
    state: Arc<AppState>,
    app: AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin(Any)
        .allow_headers(Any);

    let ctx = Arc::new(HttpContext { state, app });

    let router = Router::new()
        .route("/event", post(handle_event))
        .route("/health", get(health))
        .layer(cors)
        .with_state(ctx);

    let listener = tokio::net::TcpListener::bind(LISTEN_ADDR).await?;
    eprintln!("[cooldown] listening on http://{LISTEN_ADDR}/event");
    axum::serve(listener, router).await?;
    Ok(())
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn handle_event(
    State(ctx): State<Arc<HttpContext>>,
    Json(payload): Json<DevEvent>,
) -> StatusCode {
    let category = ErrorCategory::from_event(&payload);
    ctx.state.record_dev_event(payload.clone());

    let dashboard = ctx.state.dashboard();
    let event_payload = CooldownEventPayload {
        event: payload,
        category,
        ts: chrono::Utc::now().timestamp(),
        fatigue_score: dashboard.fatigue_score,
        errors_last_hour: dashboard.errors_last_hour,
        dashboard: dashboard.clone(),
    };

    // Push immediately to all webview listeners (don't wait for the 2 s emit loop).
    let _ = ctx.app.emit("cooldown-event", &event_payload);
    let _ = ctx.app.emit("fatigue-update", &dashboard);

    StatusCode::OK
}
