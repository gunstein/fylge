use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Operation type for marker log entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Insert,
    Update,
    Delete,
}

/// A single entry in the marker log (append-only).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LogEntry {
    pub id: i64,
    pub globe_id: String,
    pub uuid: Uuid,
    pub operation: Operation,
    pub ts: DateTime<Utc>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub icon_id: Option<String>,
    pub label: Option<String>,
}

/// Current state of a marker (computed from log).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Marker {
    pub uuid: Uuid,
    pub lat: f64,
    pub lon: f64,
    pub icon_id: String,
    pub label: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new marker.
#[derive(Debug, Deserialize)]
pub struct CreateMarkerRequest {
    pub uuid: Option<Uuid>,
    pub globe_id: Option<String>,
    pub lat: f64,
    pub lon: f64,
    pub icon_id: String,
    pub label: Option<String>,
}

/// Request to update an existing marker.
#[derive(Debug, Deserialize)]
pub struct UpdateMarkerRequest {
    pub globe_id: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub icon_id: Option<String>,
    pub label: Option<String>,
}

/// Query parameters for log endpoint.
#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub globe_id: Option<String>,
    pub after_id: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    100
}

/// Query parameters for markers endpoint.
#[derive(Debug, Deserialize)]
pub struct MarkersQuery {
    pub globe_id: Option<String>,
}
