/// Lightweight localhost HTTP API (127.0.0.1:8765) consumed by the Office.js
/// add-in task pane running inside Word/Excel/PowerPoint.
///
/// All endpoints accept and return JSON.  The server is started as a
/// background tokio task from the Tauri setup closure and shares the same
/// application state (TM, glossary, settings) via `Arc`.
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use cheshire_tm::{GlossaryStore, Matcher};

use crate::settings::Settings;

// ── Shared state ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ApiState {
    pub matcher: Arc<RwLock<Matcher>>,
    pub glossary: Arc<GlossaryStore>,
    pub settings: Arc<RwLock<Settings>>,
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Deserialize)]
struct SearchReq {
    query: String,
    /// Override the threshold from settings (0–100). Optional.
    threshold: Option<u8>,
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct GlossaryReq {
    text: String,
}

#[derive(Serialize)]
struct HealthResp {
    status: &'static str,
    version: &'static str,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn health_handler() -> Json<HealthResp> {
    Json(HealthResp {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn search_handler(
    State(s): State<ApiState>,
    Json(req): Json<SearchReq>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let settings = s.settings.read().await;
    let threshold = req.threshold.unwrap_or(settings.fuzzy_threshold);
    let limit = req.limit.unwrap_or(settings.max_matches);
    drop(settings);

    let matcher = s.matcher.read().await;
    let matches = matcher.search(&req.query, threshold as f32 / 100.0, limit);
    Ok(Json(serde_json::json!({ "matches": matches })))
}

async fn glossary_handler(
    State(s): State<ApiState>,
    Json(req): Json<GlossaryReq>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let settings = s.settings.read().await;
    let source_lang = settings.source_lang.clone();
    let target_lang = settings.target_lang.clone();
    drop(settings);

    let hits = s
        .glossary
        .lookup_in(&req.text, &source_lang, &target_lang)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "hits": hits })))
}

// ── Office add-in static files ────────────────────────────────────────────────

/// The Office.js task pane HTML is embedded at compile time so the server can
/// serve it without needing access to the source tree at runtime.
static TASKPANE_HTML: &str = include_str!("../../office-addin/taskpane.html");

async fn addin_taskpane() -> axum::response::Html<&'static str> {
    axum::response::Html(TASKPANE_HTML)
}

// ── Server startup ────────────────────────────────────────────────────────────

/// Start the HTTP API server on `127.0.0.1:{port}`.
///
/// Serves both the JSON REST API (used by Tauri commands and the Office.js
/// add-in) and the add-in task pane HTML at `/addin/taskpane.html`.
///
/// This function never returns (runs until the process exits). Call it with
/// `tokio::spawn` from the Tauri setup closure.
pub async fn start(state: ApiState, port: u16) {
    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/search", post(search_handler))
        .route("/api/glossary", post(glossary_handler))
        .route("/addin/taskpane.html", get(addin_taskpane))
        .with_state(state);

    let addr = format!("127.0.0.1:{port}");
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind HTTP API on {addr}: {e}");
            return;
        }
    };

    tracing::info!("Felix HTTP API listening on http://{addr}");

    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("HTTP API server error: {e}");
    }
}
