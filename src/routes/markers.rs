use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::db;
use crate::models::{CreateMarkerRequest, CreateMarkerResponse};
use crate::state::AppState;

/// POST /markers - Create a new marker (idempotent).
pub async fn create_marker(
    State(state): State<AppState>,
    Json(req): Json<CreateMarkerRequest>,
) -> Response {
    // Validate UUID format
    if uuid::Uuid::parse_str(&req.uuid).is_err() {
        return (StatusCode::BAD_REQUEST, "Invalid UUID format").into_response();
    }

    // Validate coordinates
    if !(-90.0..=90.0).contains(&req.lat) {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid latitude: must be between -90 and 90",
        )
            .into_response();
    }
    if !(-180.0..=180.0).contains(&req.lon) {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid longitude: must be between -180 and 180",
        )
            .into_response();
    }

    // Validate icon_id
    if req.icon_id.is_empty() {
        return (StatusCode::BAD_REQUEST, "icon_id is required").into_response();
    }
    if req.icon_id.len() > 64 {
        return (StatusCode::BAD_REQUEST, "icon_id too long (max 64 chars)").into_response();
    }

    // Validate label if present
    if let Some(ref label) = req.label {
        if label.len() > 256 {
            return (StatusCode::BAD_REQUEST, "label too long (max 256 chars)").into_response();
        }
    }

    match db::insert_marker(
        &state.pool,
        &req.uuid,
        req.lat,
        req.lon,
        &req.icon_id,
        req.label.as_deref(),
    )
    .await
    {
        Ok((marker, created)) => {
            let status_code = if created {
                StatusCode::CREATED
            } else {
                StatusCode::OK
            };
            let response = CreateMarkerResponse {
                status: if created { "created" } else { "exists" },
                marker,
            };
            (status_code, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create marker: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
                .into_response()
        }
    }
}
