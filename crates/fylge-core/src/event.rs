use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::hlc::HlcTimestamp;
use crate::node::NodeId;

/// Unique identifier for an event, composed of node ID and sequence number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId {
    pub node_id: NodeId,
    pub sequence: u64,
}

impl EventId {
    pub fn new(node_id: NodeId, sequence: u64) -> Self {
        Self { node_id, sequence }
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.node_id, self.sequence)
    }
}

/// Data for a marker (used in Upsert payload).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkerData {
    pub lat: f64,
    pub lon: f64,
    pub icon_id: String,
    pub label: Option<String>,
}

impl MarkerData {
    pub fn new(lat: f64, lon: f64, icon_id: String, label: Option<String>) -> Self {
        Self {
            lat,
            lon,
            icon_id,
            label,
        }
    }
}

/// Payload for an event - either an upsert or a tombstone (delete).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Payload {
    /// Create or update a marker.
    Upsert(MarkerData),
    /// Delete a marker (tombstone).
    Tombstone,
}

impl Payload {
    /// Create an upsert payload with marker data.
    pub fn new(lat: f64, lon: f64, icon_id: String, label: Option<String>) -> Self {
        Payload::Upsert(MarkerData::new(lat, lon, icon_id, label))
    }

    /// Create a tombstone payload for deletion.
    pub fn tombstone() -> Self {
        Payload::Tombstone
    }

    /// Check if this is a tombstone (delete) payload.
    pub fn is_tombstone(&self) -> bool {
        matches!(self, Payload::Tombstone)
    }

    /// Get the marker data if this is an upsert payload.
    pub fn marker_data(&self) -> Option<&MarkerData> {
        match self {
            Payload::Upsert(data) => Some(data),
            Payload::Tombstone => None,
        }
    }
}

/// An immutable event in the append-only log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub entity_id: Uuid,
    pub hlc: HlcTimestamp,
    pub payload: Payload,
}

impl Event {
    pub fn new(id: EventId, entity_id: Uuid, hlc: HlcTimestamp, payload: Payload) -> Self {
        Self {
            id,
            entity_id,
            hlc,
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id_display() {
        let id = EventId::new(NodeId(1), 42);
        assert_eq!(id.to_string(), "node-1:42");
    }

    #[test]
    fn test_payload_creation() {
        let payload = Payload::new(59.9, 10.7, "ship".to_string(), Some("Oslo".to_string()));

        let data = payload.marker_data().expect("Expected Upsert payload");
        assert_eq!(data.lat, 59.9);
        assert_eq!(data.lon, 10.7);
        assert_eq!(data.icon_id, "ship");
        assert_eq!(data.label, Some("Oslo".to_string()));
    }

    #[test]
    fn test_tombstone_payload() {
        let payload = Payload::tombstone();
        assert!(payload.is_tombstone());
        assert!(payload.marker_data().is_none());
    }
}
