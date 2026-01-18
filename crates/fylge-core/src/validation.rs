use crate::error::ValidationError;
use crate::event::{MarkerData, Payload};

/// Validator for marker data.
pub struct Validator;

impl Validator {
    /// Validate latitude value.
    pub fn validate_latitude(lat: f64) -> Result<(), ValidationError> {
        if !(-90.0..=90.0).contains(&lat) {
            return Err(ValidationError::InvalidLatitude(lat));
        }
        if lat.is_nan() {
            return Err(ValidationError::InvalidLatitude(lat));
        }
        Ok(())
    }

    /// Validate longitude value.
    pub fn validate_longitude(lon: f64) -> Result<(), ValidationError> {
        if !(-180.0..=180.0).contains(&lon) {
            return Err(ValidationError::InvalidLongitude(lon));
        }
        if lon.is_nan() {
            return Err(ValidationError::InvalidLongitude(lon));
        }
        Ok(())
    }

    /// Validate icon_id.
    /// Must be non-empty, max 64 chars, and only contain [a-zA-Z0-9_-].
    pub fn validate_icon_id(icon_id: &str) -> Result<(), ValidationError> {
        if icon_id.is_empty() {
            return Err(ValidationError::InvalidIconId(
                "icon_id cannot be empty".to_string(),
            ));
        }
        if icon_id.len() > 64 {
            return Err(ValidationError::InvalidIconId(format!(
                "icon_id too long: {} chars (max 64)",
                icon_id.len()
            )));
        }
        if !icon_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ValidationError::InvalidIconId(format!(
                "icon_id contains invalid characters: {}",
                icon_id
            )));
        }
        Ok(())
    }

    /// Validate optional label.
    /// If present, must be max 256 chars.
    pub fn validate_label(label: &Option<String>) -> Result<(), ValidationError> {
        if let Some(l) = label {
            if l.len() > 256 {
                return Err(ValidationError::LabelTooLong(l.len()));
            }
        }
        Ok(())
    }

    /// Validate marker data.
    pub fn validate_marker_data(data: &MarkerData) -> Result<(), ValidationError> {
        Self::validate_latitude(data.lat)?;
        Self::validate_longitude(data.lon)?;
        Self::validate_icon_id(&data.icon_id)?;
        Self::validate_label(&data.label)?;
        Ok(())
    }

    /// Validate a complete payload.
    pub fn validate_payload(payload: &Payload) -> Result<(), ValidationError> {
        match payload {
            Payload::Upsert(data) => Self::validate_marker_data(data),
            Payload::Tombstone => Ok(()), // Tombstone always valid
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_latitude() {
        assert!(Validator::validate_latitude(0.0).is_ok());
        assert!(Validator::validate_latitude(90.0).is_ok());
        assert!(Validator::validate_latitude(-90.0).is_ok());
        assert!(Validator::validate_latitude(59.9).is_ok());
    }

    #[test]
    fn test_invalid_latitude() {
        assert!(Validator::validate_latitude(90.1).is_err());
        assert!(Validator::validate_latitude(-90.1).is_err());
        assert!(Validator::validate_latitude(f64::NAN).is_err());
        assert!(Validator::validate_latitude(f64::INFINITY).is_err());
    }

    #[test]
    fn test_valid_longitude() {
        assert!(Validator::validate_longitude(0.0).is_ok());
        assert!(Validator::validate_longitude(180.0).is_ok());
        assert!(Validator::validate_longitude(-180.0).is_ok());
        assert!(Validator::validate_longitude(10.7).is_ok());
    }

    #[test]
    fn test_invalid_longitude() {
        assert!(Validator::validate_longitude(180.1).is_err());
        assert!(Validator::validate_longitude(-180.1).is_err());
        assert!(Validator::validate_longitude(f64::NAN).is_err());
    }

    #[test]
    fn test_valid_icon_id() {
        assert!(Validator::validate_icon_id("ship").is_ok());
        assert!(Validator::validate_icon_id("my-icon").is_ok());
        assert!(Validator::validate_icon_id("icon_123").is_ok());
        assert!(Validator::validate_icon_id("ABC").is_ok());
    }

    #[test]
    fn test_invalid_icon_id() {
        assert!(Validator::validate_icon_id("").is_err());
        assert!(Validator::validate_icon_id("icon with space").is_err());
        assert!(Validator::validate_icon_id("icon.png").is_err());
        assert!(Validator::validate_icon_id(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_valid_label() {
        assert!(Validator::validate_label(&None).is_ok());
        assert!(Validator::validate_label(&Some("Oslo".to_string())).is_ok());
        assert!(Validator::validate_label(&Some("A".repeat(256))).is_ok());
    }

    #[test]
    fn test_invalid_label() {
        assert!(Validator::validate_label(&Some("A".repeat(257))).is_err());
    }

    #[test]
    fn test_validate_payload() {
        let valid = Payload::new(59.9, 10.7, "ship".to_string(), Some("Oslo".to_string()));
        assert!(Validator::validate_payload(&valid).is_ok());

        let invalid_lat = Payload::new(100.0, 10.7, "ship".to_string(), None);
        assert!(Validator::validate_payload(&invalid_lat).is_err());

        let invalid_icon = Payload::new(59.9, 10.7, "".to_string(), None);
        assert!(Validator::validate_payload(&invalid_icon).is_err());
    }
}
