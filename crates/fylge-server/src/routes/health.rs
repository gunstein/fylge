use axum::{Json, Router, routing::get};
use serde::Serialize;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn ready() -> Json<HealthResponse> {
    // For now, always ready. Could check peer connectivity later.
    Json(HealthResponse { status: "ok" })
}
