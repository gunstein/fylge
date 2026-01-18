use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;

use fylge_core::{Entity, EntityStore, Icon};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/markers", get(get_markers))
        .route("/api/icons", get(get_icons))
}

#[derive(Serialize)]
struct MarkerResponse {
    id: String,
    lat: f64,
    lon: f64,
    icon_id: String,
    label: Option<String>,
}

impl From<Entity> for MarkerResponse {
    fn from(e: Entity) -> Self {
        Self {
            id: e.id.to_string(),
            lat: e.lat,
            lon: e.lon,
            icon_id: e.icon_id,
            label: e.label,
        }
    }
}

async fn get_markers(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
) -> Response {
    // Rate limiting
    if let Err(wait_time) = state.read_limiter.check(addr.ip()) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            format!("Rate limited. Try again in {:?}", wait_time),
        )
            .into_response();
    }

    let entities = state.entity_store.get_all().unwrap_or_default();
    // Filter out deleted entities (tombstones)
    let markers: Vec<MarkerResponse> = entities
        .into_iter()
        .filter(|e| !e.deleted)
        .map(Into::into)
        .collect();
    Json(markers).into_response()
}

#[derive(Serialize)]
struct IconResponse {
    id: String,
    name: String,
    url: String,
}

impl From<&Icon> for IconResponse {
    fn from(icon: &Icon) -> Self {
        Self {
            id: icon.id.clone(),
            name: icon.name.clone(),
            url: format!("/static/icons/{}", icon.filename),
        }
    }
}

async fn get_icons(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
) -> Response {
    // Rate limiting
    if let Err(wait_time) = state.read_limiter.check(addr.ip()) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            format!("Rate limited. Try again in {:?}", wait_time),
        )
            .into_response();
    }

    let icons: Vec<IconResponse> = state.icons.iter().map(Into::into).collect();
    Json(icons).into_response()
}
