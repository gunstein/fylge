pub mod api;
pub mod markers;

use axum::{
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Index page
        .route("/", get(index))
        // Marker creation (append-only, no update/delete)
        .route("/markers", post(markers::create_marker))
        // API endpoints
        .route("/api/markers", get(api::get_markers))
        .route("/api/markers_at", get(api::get_markers_at))
        .route("/api/log", get(api::get_log))
        .route("/api/icons", get(api::get_icons))
        // Health check
        .route("/health", get(health))
        .with_state(state)
}

async fn index() -> impl IntoResponse {
    Html(include_str!("../../static/dist/index.html"))
}

async fn health() -> &'static str {
    "OK"
}
