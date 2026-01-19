use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use crate::db;
use crate::models::{
    GetIconsResponse, GetLogResponse, GetMarkersAtResponse, GetMarkersResponse, Icon, LogQuery,
    MarkersAtQuery,
};
use crate::state::AppState;

/// GET /api/markers - Get markers from the last 24 hours.
pub async fn get_markers(State(state): State<AppState>) -> Response {
    let server_time = match db::get_server_time(&state.pool).await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to get server time: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
                .into_response();
        }
    };

    match db::get_markers_last_24h(&state.pool).await {
        Ok((markers, max_id)) => {
            let response = GetMarkersResponse {
                window_hours: 24,
                server_time,
                max_id,
                markers,
            };
            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get markers: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
                .into_response()
        }
    }
}

/// GET /api/markers_at?at=<ISO timestamp> - Get markers visible at a specific time.
pub async fn get_markers_at(
    State(state): State<AppState>,
    Query(query): Query<MarkersAtQuery>,
) -> Response {
    match db::get_markers_at(&state.pool, &query.at).await {
        Ok(markers) => {
            let response = GetMarkersAtResponse {
                at: query.at,
                window_hours: 24,
                markers,
            };
            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get markers at time: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
                .into_response()
        }
    }
}

/// GET /api/log?after_id=...&limit=... - Get log entries for polling/sync.
pub async fn get_log(State(state): State<AppState>, Query(query): Query<LogQuery>) -> Response {
    let server_time = match db::get_server_time(&state.pool).await {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to get server time: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
                .into_response();
        }
    };

    match db::get_log_after(&state.pool, query.after_id, query.limit).await {
        Ok((entries, max_id, has_more)) => {
            let response = GetLogResponse {
                after_id: query.after_id,
                limit: query.limit,
                server_time,
                max_id,
                has_more,
                entries,
            };
            Json(response).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get log: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
                .into_response()
        }
    }
}

/// GET /api/icons - Get available icons.
pub async fn get_icons(State(state): State<AppState>) -> Json<GetIconsResponse> {
    Json(GetIconsResponse {
        icons: state.icons.clone(),
    })
}

/// Load icons from icons.json file.
pub fn load_icons() -> Vec<Icon> {
    let icons_path = std::path::Path::new("static/icons/icons.json");
    if icons_path.exists() {
        match std::fs::read_to_string(icons_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(icons) => return icons,
                Err(e) => {
                    tracing::warn!("Failed to parse icons.json: {}", e);
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read icons.json: {}", e);
            }
        }
    }

    // Default icons if file doesn't exist or fails to load
    vec![
        Icon {
            id: "marker".to_string(),
            name: "Marker".to_string(),
            url: "/static/icons/marker.svg".to_string(),
        },
        Icon {
            id: "ship".to_string(),
            name: "Ship".to_string(),
            url: "/static/icons/ship.svg".to_string(),
        },
        Icon {
            id: "plane".to_string(),
            name: "Plane".to_string(),
            url: "/static/icons/plane.svg".to_string(),
        },
    ]
}
