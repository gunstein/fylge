use serde::{Deserialize, Serialize};

/// Metadata for an icon that can be used as a marker on the globe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Icon {
    /// Unique identifier for the icon (e.g., "ship")
    pub id: String,
    /// Display name (e.g., "Skip")
    pub name: String,
    /// Filename in the icons directory (e.g., "ship.png")
    pub filename: String,
}

impl Icon {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        filename: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            filename: filename.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_creation() {
        let icon = Icon::new("ship", "Skip", "ship.png");

        assert_eq!(icon.id, "ship");
        assert_eq!(icon.name, "Skip");
        assert_eq!(icon.filename, "ship.png");
    }
}
