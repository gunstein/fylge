use serde::{Deserialize, Serialize};

/// Validation error type.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    InvalidUuid(String),
    InvalidLatitude(f64),
    InvalidLongitude(f64),
    EmptyIconId,
    IconIdTooLong(usize),
    LabelTooLong(usize),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidUuid(s) => write!(f, "Invalid UUID format: {}", s),
            ValidationError::InvalidLatitude(lat) => {
                write!(f, "Invalid latitude: {} (must be between -90 and 90)", lat)
            }
            ValidationError::InvalidLongitude(lon) => {
                write!(
                    f,
                    "Invalid longitude: {} (must be between -180 and 180)",
                    lon
                )
            }
            ValidationError::EmptyIconId => write!(f, "icon_id is required"),
            ValidationError::IconIdTooLong(len) => {
                write!(f, "icon_id too long: {} chars (max 64)", len)
            }
            ValidationError::LabelTooLong(len) => {
                write!(f, "label too long: {} chars (max 256)", len)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// A marker in the log (append-only).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, PartialEq)]
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
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CreateMarkerRequest {
    pub uuid: String,
    pub lat: f64,
    pub lon: f64,
    pub icon_id: String,
    pub label: Option<String>,
}

impl CreateMarkerRequest {
    /// Validate the request and return a list of validation errors.
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate UUID format
        if uuid::Uuid::parse_str(&self.uuid).is_err() {
            return Err(ValidationError::InvalidUuid(self.uuid.clone()));
        }

        // Validate latitude
        if !(-90.0..=90.0).contains(&self.lat) {
            return Err(ValidationError::InvalidLatitude(self.lat));
        }

        // Validate longitude
        if !(-180.0..=180.0).contains(&self.lon) {
            return Err(ValidationError::InvalidLongitude(self.lon));
        }

        // Validate icon_id
        if self.icon_id.is_empty() {
            return Err(ValidationError::EmptyIconId);
        }
        if self.icon_id.len() > 64 {
            return Err(ValidationError::IconIdTooLong(self.icon_id.len()));
        }

        // Validate label if present
        if let Some(ref label) = self.label {
            if label.len() > 256 {
                return Err(ValidationError::LabelTooLong(label.len()));
            }
        }

        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> CreateMarkerRequest {
        CreateMarkerRequest {
            uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            lat: 59.91,
            lon: 10.75,
            icon_id: "marker".to_string(),
            label: Some("Oslo".to_string()),
        }
    }

    #[test]
    fn test_valid_request() {
        let req = valid_request();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_valid_request_without_label() {
        let req = CreateMarkerRequest {
            label: None,
            ..valid_request()
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_invalid_uuid() {
        let req = CreateMarkerRequest {
            uuid: "not-a-uuid".to_string(),
            ..valid_request()
        };
        assert_eq!(
            req.validate(),
            Err(ValidationError::InvalidUuid("not-a-uuid".to_string()))
        );
    }

    #[test]
    fn test_invalid_latitude_too_high() {
        let req = CreateMarkerRequest {
            lat: 91.0,
            ..valid_request()
        };
        assert_eq!(req.validate(), Err(ValidationError::InvalidLatitude(91.0)));
    }

    #[test]
    fn test_invalid_latitude_too_low() {
        let req = CreateMarkerRequest {
            lat: -91.0,
            ..valid_request()
        };
        assert_eq!(req.validate(), Err(ValidationError::InvalidLatitude(-91.0)));
    }

    #[test]
    fn test_valid_latitude_boundary() {
        let req_max = CreateMarkerRequest {
            lat: 90.0,
            ..valid_request()
        };
        assert!(req_max.validate().is_ok());

        let req_min = CreateMarkerRequest {
            lat: -90.0,
            ..valid_request()
        };
        assert!(req_min.validate().is_ok());
    }

    #[test]
    fn test_invalid_longitude_too_high() {
        let req = CreateMarkerRequest {
            lon: 181.0,
            ..valid_request()
        };
        assert_eq!(
            req.validate(),
            Err(ValidationError::InvalidLongitude(181.0))
        );
    }

    #[test]
    fn test_invalid_longitude_too_low() {
        let req = CreateMarkerRequest {
            lon: -181.0,
            ..valid_request()
        };
        assert_eq!(
            req.validate(),
            Err(ValidationError::InvalidLongitude(-181.0))
        );
    }

    #[test]
    fn test_valid_longitude_boundary() {
        let req_max = CreateMarkerRequest {
            lon: 180.0,
            ..valid_request()
        };
        assert!(req_max.validate().is_ok());

        let req_min = CreateMarkerRequest {
            lon: -180.0,
            ..valid_request()
        };
        assert!(req_min.validate().is_ok());
    }

    #[test]
    fn test_empty_icon_id() {
        let req = CreateMarkerRequest {
            icon_id: "".to_string(),
            ..valid_request()
        };
        assert_eq!(req.validate(), Err(ValidationError::EmptyIconId));
    }

    #[test]
    fn test_icon_id_too_long() {
        let req = CreateMarkerRequest {
            icon_id: "a".repeat(65),
            ..valid_request()
        };
        assert_eq!(req.validate(), Err(ValidationError::IconIdTooLong(65)));
    }

    #[test]
    fn test_icon_id_max_length() {
        let req = CreateMarkerRequest {
            icon_id: "a".repeat(64),
            ..valid_request()
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_label_too_long() {
        let req = CreateMarkerRequest {
            label: Some("a".repeat(257)),
            ..valid_request()
        };
        assert_eq!(req.validate(), Err(ValidationError::LabelTooLong(257)));
    }

    #[test]
    fn test_label_max_length() {
        let req = CreateMarkerRequest {
            label: Some("a".repeat(256)),
            ..valid_request()
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_marker_serialization() {
        let marker = Marker {
            id: 1,
            uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            ts: "2024-01-19T12:00:00.000Z".to_string(),
            lat: 59.91,
            lon: 10.75,
            icon_id: "marker".to_string(),
            label: Some("Oslo".to_string()),
        };

        let json = serde_json::to_string(&marker).unwrap();
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"uuid\":\"550e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"lat\":59.91"));
        assert!(json.contains("\"label\":\"Oslo\""));
    }

    #[test]
    fn test_marker_deserialization() {
        let json = r#"{
            "id": 1,
            "uuid": "550e8400-e29b-41d4-a716-446655440000",
            "ts": "2024-01-19T12:00:00.000Z",
            "lat": 59.91,
            "lon": 10.75,
            "icon_id": "marker",
            "label": "Oslo"
        }"#;

        let marker: Marker = serde_json::from_str(json).unwrap();
        assert_eq!(marker.id, 1);
        assert_eq!(marker.lat, 59.91);
        assert_eq!(marker.label, Some("Oslo".to_string()));
    }

    #[test]
    fn test_log_query_defaults() {
        let json = "{}";
        let query: LogQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.after_id, 0);
        assert_eq!(query.limit, 100);
    }

    #[test]
    fn test_log_query_custom_values() {
        let json = r#"{"after_id": 50, "limit": 25}"#;
        let query: LogQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.after_id, 50);
        assert_eq!(query.limit, 25);
    }

    #[test]
    fn test_icon_serialization() {
        let icon = Icon {
            id: "ship".to_string(),
            name: "Ship".to_string(),
            url: "/static/icons/ship.svg".to_string(),
        };

        let json = serde_json::to_string(&icon).unwrap();
        assert!(json.contains("\"id\":\"ship\""));
        assert!(json.contains("\"name\":\"Ship\""));
        assert!(json.contains("\"url\":\"/static/icons/ship.svg\""));
    }
}
