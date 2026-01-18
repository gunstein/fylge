use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::db;
use crate::models::{CreateMarkerRequest, UpdateMarkerRequest};
use crate::state::AppState;

/// POST /markers - Create a new marker.
pub async fn create_marker(
    State(state): State<AppState>,
    Json(req): Json<CreateMarkerRequest>,
) -> Response {
    let uuid = req
        .uuid
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or_else(Uuid::new_v4);
    let globe_id = req.globe_id.as_deref().unwrap_or("default");

    // Validate coordinates
    if !(-90.0..=90.0).contains(&req.lat) {
        return (StatusCode::BAD_REQUEST, "Invalid latitude").into_response();
    }
    if !(-180.0..=180.0).contains(&req.lon) {
        return (StatusCode::BAD_REQUEST, "Invalid longitude").into_response();
    }
    if req.icon_id.is_empty() {
        return (StatusCode::BAD_REQUEST, "icon_id is required").into_response();
    }

    match db::insert_marker(
        &state.pool,
        globe_id,
        uuid,
        req.lat,
        req.lon,
        &req.icon_id,
        req.label.as_deref(),
    )
    .await
    {
        Ok(id) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "id": id, "uuid": uuid.to_string() })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to create marker: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

/// PUT /markers/:uuid - Update an existing marker.
pub async fn update_marker(
    State(state): State<AppState>,
    Path(uuid_str): Path<String>,
    Json(req): Json<UpdateMarkerRequest>,
) -> Response {
    let uuid = match Uuid::parse_str(&uuid_str) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid UUID").into_response(),
    };
    let globe_id = req.globe_id.as_deref().unwrap_or("default");

    // Check if marker exists
    match db::marker_exists(&state.pool, globe_id, uuid).await {
        Ok(false) => return (StatusCode::NOT_FOUND, "Marker not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to check marker: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
        Ok(true) => {}
    }

    // Validate coordinates if provided
    if let Some(lat) = req.lat {
        if !(-90.0..=90.0).contains(&lat) {
            return (StatusCode::BAD_REQUEST, "Invalid latitude").into_response();
        }
    }
    if let Some(lon) = req.lon {
        if !(-180.0..=180.0).contains(&lon) {
            return (StatusCode::BAD_REQUEST, "Invalid longitude").into_response();
        }
    }

    match db::update_marker(
        &state.pool,
        globe_id,
        uuid,
        req.lat,
        req.lon,
        req.icon_id.as_deref(),
        req.label.as_deref(),
    )
    .await
    {
        Ok(id) => Json(serde_json::json!({ "id": id, "uuid": uuid.to_string() })).into_response(),
        Err(e) => {
            tracing::error!("Failed to update marker: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

/// DELETE /markers/:uuid - Delete a marker.
pub async fn delete_marker(
    State(state): State<AppState>,
    Path(uuid_str): Path<String>,
) -> Response {
    let uuid = match Uuid::parse_str(&uuid_str) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid UUID").into_response(),
    };
    let globe_id = "default"; // Could be passed as query param if needed

    // Check if marker exists
    match db::marker_exists(&state.pool, globe_id, uuid).await {
        Ok(false) => return (StatusCode::NOT_FOUND, "Marker not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to check marker: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
        Ok(true) => {}
    }

    match db::delete_marker(&state.pool, globe_id, uuid).await {
        Ok(id) => Json(serde_json::json!({ "id": id, "uuid": uuid.to_string() })).into_response(),
        Err(e) => {
            tracing::error!("Failed to delete marker: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}
