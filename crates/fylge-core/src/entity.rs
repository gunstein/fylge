use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::event::{Event, EventId, Payload};
use crate::hlc::HlcTimestamp;

/// A materialized entity representing the current state of a marker.
///
/// This is derived from the event log using last-write-wins semantics
/// based on HLC timestamps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub lat: f64,
    pub lon: f64,
    pub icon_id: String,
    pub label: Option<String>,
    pub hlc: HlcTimestamp,
    pub source_event: EventId,
    /// Whether this entity has been deleted (tombstone).
    #[serde(default)]
    pub deleted: bool,
}

impl Entity {
    /// Create an entity from an event.
    /// For tombstone events, creates a deleted entity with zeroed coordinates.
    pub fn from_event(event: &Event) -> Self {
        match &event.payload {
            Payload::Upsert(data) => Self {
                id: event.entity_id,
                lat: data.lat,
                lon: data.lon,
                icon_id: data.icon_id.clone(),
                label: data.label.clone(),
                hlc: event.hlc,
                source_event: event.id,
                deleted: false,
            },
            Payload::Tombstone => Self {
                id: event.entity_id,
                lat: 0.0,
                lon: 0.0,
                icon_id: String::new(),
                label: None,
                hlc: event.hlc,
                source_event: event.id,
                deleted: true,
            },
        }
    }

    /// Check if this entity should be replaced by one derived from the given event.
    /// Returns true if the event's HLC is greater than this entity's HLC.
    pub fn should_replace_with(&self, event: &Event) -> bool {
        event.hlc > self.hlc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeId;

    fn make_event(node: u64, seq: u64, wall_time: u64, counter: u32) -> Event {
        Event::new(
            EventId::new(NodeId(node), seq),
            Uuid::new_v4(),
            HlcTimestamp::new(wall_time, counter, NodeId(node)),
            Payload::new(59.9, 10.7, "ship".to_string(), None),
        )
    }

    #[test]
    fn test_entity_from_event() {
        let event = make_event(1, 1, 1000, 0);
        let entity = Entity::from_event(&event);

        assert_eq!(entity.id, event.entity_id);
        assert_eq!(entity.lat, 59.9);
        assert_eq!(entity.lon, 10.7);
        assert_eq!(entity.icon_id, "ship");
        assert_eq!(entity.source_event, event.id);
        assert!(!entity.deleted);
    }

    #[test]
    fn test_entity_from_tombstone_event() {
        let entity_id = Uuid::new_v4();
        let event = Event::new(
            EventId::new(NodeId(1), 1),
            entity_id,
            HlcTimestamp::new(1000, 0, NodeId(1)),
            Payload::tombstone(),
        );
        let entity = Entity::from_event(&event);

        assert_eq!(entity.id, entity_id);
        assert!(entity.deleted);
        assert_eq!(entity.source_event, event.id);
    }

    #[test]
    fn test_should_replace_with() {
        let event1 = make_event(1, 1, 1000, 0);
        let entity = Entity::from_event(&event1);

        // Later event should replace
        let event2 = Event::new(
            EventId::new(NodeId(1), 2),
            event1.entity_id,
            HlcTimestamp::new(1001, 0, NodeId(1)),
            Payload::new(60.0, 11.0, "plane".to_string(), None),
        );
        assert!(entity.should_replace_with(&event2));

        // Earlier event should not replace
        let event3 = Event::new(
            EventId::new(NodeId(2), 1),
            event1.entity_id,
            HlcTimestamp::new(999, 0, NodeId(2)),
            Payload::new(60.0, 11.0, "plane".to_string(), None),
        );
        assert!(!entity.should_replace_with(&event3));
    }

    #[test]
    fn test_tombstone_replaces_entity() {
        let entity_id = Uuid::new_v4();
        let event1 = Event::new(
            EventId::new(NodeId(1), 1),
            entity_id,
            HlcTimestamp::new(1000, 0, NodeId(1)),
            Payload::new(59.9, 10.7, "ship".to_string(), None),
        );
        let entity = Entity::from_event(&event1);
        assert!(!entity.deleted);

        // Later tombstone should replace
        let tombstone = Event::new(
            EventId::new(NodeId(1), 2),
            entity_id,
            HlcTimestamp::new(1001, 0, NodeId(1)),
            Payload::tombstone(),
        );
        assert!(entity.should_replace_with(&tombstone));
    }
}
