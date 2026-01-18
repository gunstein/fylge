use axum::{
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, post},
    Form, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use fylge_core::{Entity, EntityStore, EventStore, Payload, Validator};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/markers", post(create_marker))
        .route("/markers/{id}", delete(delete_marker))
}

#[derive(Deserialize)]
pub struct CreateMarkerForm {
    lat: f64,
    lon: f64,
    icon_id: String,
    label: Option<String>,
}

async fn create_marker(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    Form(form): Form<CreateMarkerForm>,
) -> Response {
    // Rate limiting
    if let Err(wait_time) = state.write_limiter.check(addr.ip()) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            format!("Rate limited. Try again in {:?}", wait_time),
        )
            .into_response();
    }

    // Validate input
    let payload = Payload::new(form.lat, form.lon, form.icon_id.clone(), form.label.clone());

    if let Err(e) = Validator::validate_payload(&payload) {
        return (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)).into_response();
    }

    // Check if icon exists
    if !state.icons.iter().any(|i| i.id == form.icon_id) {
        return (
            StatusCode::BAD_REQUEST,
            format!("Icon not found: {}", form.icon_id),
        )
            .into_response();
    }

    // Generate HLC timestamp
    let hlc = match state.hlc.now() {
        Ok(ts) => ts,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Clock error: {}", e),
            )
                .into_response();
        }
    };

    // Atomically create and store event (avoids race condition with sequence)
    let entity_id = Uuid::new_v4();
    let result = match state
        .event_store
        .append_local(state.node_id, entity_id, hlc, payload)
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Storage error: {}", e),
            )
                .into_response();
        }
    };

    let event = result.event;

    // Update entity
    let entity = Entity::from_event(&event);
    if let Err(e) = state.entity_store.upsert(entity) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Storage error: {}", e),
        )
            .into_response();
    }

    // Return success with HX-Trigger to update the globe
    (
        StatusCode::CREATED,
        [("HX-Trigger", "markersChanged")],
        Html(format!(
            r#"<div class="success">Marker created at ({:.4}, {:.4})</div>"#,
            form.lat, form.lon
        )),
    )
        .into_response()
}

async fn delete_marker(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    Path(id): Path<String>,
) -> Response {
    // Rate limiting
    if let Err(wait_time) = state.write_limiter.check(addr.ip()) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            format!("Rate limited. Try again in {:?}", wait_time),
        )
            .into_response();
    }

    let entity_id = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid marker ID").into_response();
        }
    };

    // Check if entity exists
    match state.entity_store.get(entity_id) {
        Ok(Some(entity)) if !entity.deleted => {}
        Ok(Some(_)) => {
            return (StatusCode::NOT_FOUND, "Marker already deleted").into_response();
        }
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Marker not found").into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Storage error: {}", e),
            )
                .into_response();
        }
    }

    // Generate HLC timestamp
    let hlc = match state.hlc.now() {
        Ok(ts) => ts,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Clock error: {}", e),
            )
                .into_response();
        }
    };

    // Create tombstone event
    let payload = Payload::tombstone();
    let result = match state
        .event_store
        .append_local(state.node_id, entity_id, hlc, payload)
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Storage error: {}", e),
            )
                .into_response();
        }
    };

    // Update entity with tombstone
    let entity = Entity::from_event(&result.event);
    if let Err(e) = state.entity_store.upsert(entity) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Storage error: {}", e),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        [("HX-Trigger", "markersChanged")],
        Html("<div class=\"success\">Marker deleted</div>".to_string()),
    )
        .into_response()
}
