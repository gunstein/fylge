use fylge_core::{Entity, Event};

/// Materializes entities from events using Last-Write-Wins semantics.
pub struct EntityMaterializer;

impl EntityMaterializer {
    /// Materialize an entity from a set of events for the same entity_id.
    /// Returns the entity derived from the event with the highest HLC timestamp.
    pub fn materialize(events: impl IntoIterator<Item = Event>) -> Option<Entity> {
        events
            .into_iter()
            .max_by_key(|e| e.hlc)
            .map(|e| Entity::from_event(&e))
    }

    /// Materialize an entity from event references.
    pub fn materialize_ref<'a>(events: impl IntoIterator<Item = &'a Event>) -> Option<Entity> {
        events
            .into_iter()
            .max_by_key(|e| e.hlc)
            .map(|e| Entity::from_event(e))
    }

    /// Determine if a new event should replace an existing entity.
    pub fn should_replace(existing: &Entity, new_event: &Event) -> bool {
        new_event.hlc > existing.hlc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fylge_core::{EventId, HlcTimestamp, NodeId, Payload};
    use uuid::Uuid;

    fn make_event(node: u64, seq: u64, wall_time: u64, entity_id: Uuid) -> Event {
        Event::new(
            EventId::new(NodeId(node), seq),
            entity_id,
            HlcTimestamp::new(wall_time, 0, NodeId(node)),
            Payload::new(59.9, 10.7, "ship".to_string(), None),
        )
    }

    #[test]
    fn test_materialize_single_event() {
        let entity_id = Uuid::new_v4();
        let event = make_event(1, 1, 1000, entity_id);

        let entity = EntityMaterializer::materialize(vec![event.clone()]).unwrap();

        assert_eq!(entity.id, entity_id);
        assert_eq!(entity.hlc.wall_time, 1000);
    }

    #[test]
    fn test_materialize_picks_latest() {
        let entity_id = Uuid::new_v4();
        let events = vec![
            make_event(1, 1, 1000, entity_id),
            make_event(1, 2, 2000, entity_id), // Latest
            make_event(1, 3, 1500, entity_id),
        ];

        let entity = EntityMaterializer::materialize(events).unwrap();

        assert_eq!(entity.hlc.wall_time, 2000);
    }

    #[test]
    fn test_materialize_tiebreak_by_node_id() {
        let entity_id = Uuid::new_v4();
        let events = vec![
            make_event(1, 1, 1000, entity_id),
            make_event(2, 1, 1000, entity_id), // Same time, higher node_id wins
        ];

        let entity = EntityMaterializer::materialize(events).unwrap();

        assert_eq!(entity.source_event.node_id, NodeId(2));
    }

    #[test]
    fn test_materialize_empty_returns_none() {
        let entity = EntityMaterializer::materialize(Vec::<Event>::new());
        assert!(entity.is_none());
    }

    #[test]
    fn test_should_replace() {
        let entity_id = Uuid::new_v4();
        let event1 = make_event(1, 1, 1000, entity_id);
        let entity = Entity::from_event(&event1);

        let newer = make_event(1, 2, 2000, entity_id);
        let older = make_event(1, 3, 500, entity_id);

        assert!(EntityMaterializer::should_replace(&entity, &newer));
        assert!(!EntityMaterializer::should_replace(&entity, &older));
    }
}
