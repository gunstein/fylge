use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::db;
use crate::models::{ApiError, CreateMarkerRequest, CreateMarkerResponse};
use crate::state::AppState;

/// POST /markers - Create a new marker (idempotent).
pub async fn create_marker(
    State(state): State<AppState>,
    Json(req): Json<CreateMarkerRequest>,
) -> Response {
    // Validate request including icon_id against available icons
    if let Err(e) = req.validate_with_icons(&state.icon_ids) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiError::from_validation_error(&e)),
        )
            .into_response();
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
                Json(ApiError::new(format!("Database error: {}", e))),
            )
                .into_response()
        }
    }
}
