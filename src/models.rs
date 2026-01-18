use serde::{Deserialize, Serialize};

/// Operation type for marker log entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Insert,
    Update,
    Delete,
}

// Custom FromRow implementation for Operation from TEXT
impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for Operation {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let text = <&str as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        match text {
            "insert" => Ok(Operation::Insert),
            "update" => Ok(Operation::Update),
            "delete" => Ok(Operation::Delete),
            _ => Err(format!("Unknown operation: {}", text).into()),
        }
    }
}

impl sqlx::Type<sqlx::Sqlite> for Operation {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <&str as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

/// A single entry in the marker log (append-only).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LogEntry {
    pub id: i64,
    pub globe_id: String,
    pub uuid: String, // UUID stored as TEXT in SQLite
    pub operation: Operation,
    pub ts: String, // Timestamp stored as TEXT in SQLite
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub icon_id: Option<String>,
    pub label: Option<String>,
}

/// Current state of a marker (computed from log).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Marker {
    pub uuid: String, // UUID stored as TEXT in SQLite
    pub lat: f64,
    pub lon: f64,
    pub icon_id: String,
    pub label: Option<String>,
    pub updated_at: String, // Timestamp stored as TEXT in SQLite
}

/// Request to create a new marker.
#[derive(Debug, Deserialize)]
pub struct CreateMarkerRequest {
    pub uuid: Option<String>,
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
