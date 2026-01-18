pub mod api;
pub mod markers;

use axum::{
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
    Router,
};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Index page
        .route("/", get(index))
        // Marker CRUD
        .route("/markers", post(markers::create_marker))
        .route("/markers/{uuid}", put(markers::update_marker))
        .route("/markers/{uuid}", delete(markers::delete_marker))
        // API endpoints
        .route("/api/log", get(api::get_log))
        .route("/api/markers", get(api::get_markers))
        .route("/api/icons", get(api::get_icons))
        // Health check
        .route("/health", get(health))
        .with_state(state)
}

async fn index() -> impl IntoResponse {
    Html(include_str!("../../static/index.html"))
}

async fn health() -> &'static str {
    "OK"
}
