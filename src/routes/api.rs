use axum::{
    extract::{Query, State},
    Json,
};

use crate::db;
use crate::models::{LogEntry, LogQuery, Marker, MarkersQuery};
use crate::state::AppState;

/// GET /api/log - Get log entries for syncing.
pub async fn get_log(
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> Json<Vec<LogEntry>> {
    let globe_id = query.globe_id.as_deref().unwrap_or("default");
    let after_id = query.after_id.unwrap_or(0);

    match db::get_log(&state.pool, globe_id, after_id, query.limit).await {
        Ok(entries) => Json(entries),
        Err(e) => {
            tracing::error!("Failed to get log: {}", e);
            Json(vec![])
        }
    }
}

/// GET /api/markers - Get current state of all markers.
pub async fn get_markers(
    State(state): State<AppState>,
    Query(query): Query<MarkersQuery>,
) -> Json<Vec<Marker>> {
    let globe_id = query.globe_id.as_deref().unwrap_or("default");

    match db::get_markers(&state.pool, globe_id).await {
        Ok(markers) => Json(markers),
        Err(e) => {
            tracing::error!("Failed to get markers: {}", e);
            Json(vec![])
        }
    }
}

/// GET /api/icons - Get available icons.
pub async fn get_icons() -> Json<Vec<IconResponse>> {
    // Hardcoded for now
    Json(vec![
        IconResponse {
            id: "marker".to_string(),
            name: "Marker".to_string(),
            url: "/static/icons/marker.svg".to_string(),
        },
        IconResponse {
            id: "ship".to_string(),
            name: "Ship".to_string(),
            url: "/static/icons/ship.svg".to_string(),
        },
        IconResponse {
            id: "plane".to_string(),
            name: "Plane".to_string(),
            url: "/static/icons/plane.svg".to_string(),
        },
    ])
}

#[derive(serde::Serialize)]
pub struct IconResponse {
    id: String,
    name: String,
    url: String,
}
