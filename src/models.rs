use serde::{Deserialize, Serialize};

/// A marker in the log (append-only).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Marker {
    pub id: i64,
    pub uuid: String,
    pub ts: String,
    pub lat: f64,
    pub lon: f64,
    pub icon_id: String,
    pub label: Option<String>,
}

/// Request to create a new marker.
#[derive(Debug, Deserialize)]
pub struct CreateMarkerRequest {
    pub uuid: String,
    pub lat: f64,
    pub lon: f64,
    pub icon_id: String,
    pub label: Option<String>,
}

/// Response for creating a marker.
#[derive(Debug, Serialize)]
pub struct CreateMarkerResponse {
    pub status: &'static str, // "created" or "exists"
    pub marker: Marker,
}

/// Response for getting markers (last 24h).
#[derive(Debug, Serialize)]
pub struct GetMarkersResponse {
    pub window_hours: u32,
    pub server_time: String,
    pub max_id: i64,
    pub markers: Vec<Marker>,
}

/// Response for getting markers at a specific time.
#[derive(Debug, Serialize)]
pub struct GetMarkersAtResponse {
    pub at: String,
    pub window_hours: u32,
    pub markers: Vec<Marker>,
}

/// Query parameters for markers_at endpoint.
#[derive(Debug, Deserialize)]
pub struct MarkersAtQuery {
    pub at: String,
}

/// Query parameters for log endpoint.
#[derive(Debug, Deserialize)]
pub struct LogQuery {
    #[serde(default)]
    pub after_id: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    100
}

/// Response for log endpoint.
#[derive(Debug, Serialize)]
pub struct GetLogResponse {
    pub after_id: i64,
    pub limit: i64,
    pub server_time: String,
    pub max_id: i64,
    pub has_more: bool,
    pub entries: Vec<Marker>,
}

/// Icon metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Icon {
    pub id: String,
    pub name: String,
    pub url: String,
}

/// Response for icons endpoint.
#[derive(Debug, Serialize)]
pub struct GetIconsResponse {
    pub icons: Vec<Icon>,
}
