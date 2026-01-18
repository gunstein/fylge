use uuid::Uuid;

use crate::entity::Entity;
use crate::error::StorageError;
use crate::event::Event;
use crate::node::NodeId;

/// Checkpoint for tracking replication progress.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ReplicationCheckpoint {
    /// Map from node_id to last seen sequence number.
    pub node_sequences: std::collections::HashMap<NodeId, u64>,
}

impl ReplicationCheckpoint {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the last seen sequence for a node.
    pub fn last_seq_for(&self, node_id: NodeId) -> u64 {
        self.node_sequences.get(&node_id).copied().unwrap_or(0)
    }

    /// Update the sequence for a node.
    pub fn update(&mut self, node_id: NodeId, seq: u64) {
        self.node_sequences.insert(node_id, seq);
    }
}

use crate::event::Payload;
use crate::hlc::HlcTimestamp;

/// Result of appending a local event.
#[derive(Debug, Clone)]
pub struct AppendResult {
    /// The event that was created and stored.
    pub event: Event,
    /// The sequence number assigned.
    pub sequence: u64,
}

/// Trait for storing and retrieving events.
pub trait EventStore: Send + Sync {
    /// Append an event to the store.
    /// Returns Ok(true) if the event was inserted, Ok(false) if it was a duplicate.
    fn append(&self, event: Event) -> Result<bool, StorageError>;

    /// Atomically create and append a local event.
    /// This reserves a sequence number and appends the event in a single transaction,
    /// avoiding race conditions between next_sequence() and append().
    fn append_local(
        &self,
        node_id: NodeId,
        entity_id: Uuid,
        hlc: HlcTimestamp,
        payload: Payload,
    ) -> Result<AppendResult, StorageError>;

    /// Get all events for a specific entity.
    fn get_events_for_entity(&self, entity_id: Uuid) -> Result<Vec<Event>, StorageError>;

    /// Get events from a specific node since a given sequence number.
    fn get_events_since(&self, node_id: NodeId, since_seq: u64)
        -> Result<Vec<Event>, StorageError>;

    /// Get all events in the store.
    fn get_all_events(&self) -> Result<Vec<Event>, StorageError>;

    /// Get the next sequence number for this node.
    /// Note: For local events, prefer append_local() to avoid race conditions.
    fn next_sequence(&self, node_id: NodeId) -> Result<u64, StorageError>;
}

/// Trait for storing and retrieving materialized entities.
pub trait EntityStore: Send + Sync {
    /// Get an entity by ID.
    fn get(&self, id: Uuid) -> Result<Option<Entity>, StorageError>;

    /// Get all entities.
    fn get_all(&self) -> Result<Vec<Entity>, StorageError>;

    /// Upsert an entity (insert or update if newer).
    fn upsert(&self, entity: Entity) -> Result<(), StorageError>;

    /// Delete an entity.
    fn delete(&self, id: Uuid) -> Result<bool, StorageError>;
}

/// Trait for storing replication checkpoints.
pub trait CheckpointStore: Send + Sync {
    /// Get the checkpoint for a peer node.
    fn get_checkpoint(&self, peer: NodeId) -> Result<ReplicationCheckpoint, StorageError>;

    /// Save the checkpoint for a peer node.
    fn save_checkpoint(
        &self,
        peer: NodeId,
        checkpoint: ReplicationCheckpoint,
    ) -> Result<(), StorageError>;
}

// In-memory implementations for testing
#[cfg(any(test, feature = "test-utils"))]
pub mod memory {
    use super::*;
    use std::collections::HashMap;
    use std::sync::RwLock;

    /// In-memory event store for testing.
    #[derive(Default)]
    pub struct InMemoryEventStore {
        events: RwLock<Vec<Event>>,
        sequences: RwLock<HashMap<NodeId, u64>>,
    }

    impl InMemoryEventStore {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl EventStore for InMemoryEventStore {
        fn append(&self, event: Event) -> Result<bool, StorageError> {
            let mut events = self.events.write().unwrap();

            // Check for duplicate
            if events
                .iter()
                .any(|e| e.id.node_id == event.id.node_id && e.id.sequence == event.id.sequence)
            {
                return Ok(false);
            }

            // Update sequence tracker
            {
                let mut seqs = self.sequences.write().unwrap();
                let current = seqs.get(&event.id.node_id).copied().unwrap_or(0);
                if event.id.sequence > current {
                    seqs.insert(event.id.node_id, event.id.sequence);
                }
            }

            events.push(event);
            Ok(true)
        }

        fn get_events_for_entity(&self, entity_id: Uuid) -> Result<Vec<Event>, StorageError> {
            let events = self.events.read().unwrap();
            Ok(events
                .iter()
                .filter(|e| e.entity_id == entity_id)
                .cloned()
                .collect())
        }

        fn get_events_since(
            &self,
            node_id: NodeId,
            since_seq: u64,
        ) -> Result<Vec<Event>, StorageError> {
            let events = self.events.read().unwrap();
            Ok(events
                .iter()
                .filter(|e| e.id.node_id == node_id && e.id.sequence > since_seq)
                .cloned()
                .collect())
        }

        fn get_all_events(&self) -> Result<Vec<Event>, StorageError> {
            Ok(self.events.read().unwrap().clone())
        }

        fn next_sequence(&self, node_id: NodeId) -> Result<u64, StorageError> {
            let seqs = self.sequences.read().unwrap();
            Ok(seqs.get(&node_id).copied().unwrap_or(0) + 1)
        }

        fn append_local(
            &self,
            node_id: NodeId,
            entity_id: Uuid,
            hlc: HlcTimestamp,
            payload: Payload,
        ) -> Result<AppendResult, StorageError> {
            let mut events = self.events.write().unwrap();
            let mut seqs = self.sequences.write().unwrap();

            // Get next sequence atomically
            let sequence = seqs.get(&node_id).copied().unwrap_or(0) + 1;

            // Create event
            let event = Event::new(EventId::new(node_id, sequence), entity_id, hlc, payload);

            // Store event and update sequence
            events.push(event.clone());
            seqs.insert(node_id, sequence);

            Ok(AppendResult { event, sequence })
        }
    }

    /// In-memory entity store for testing.
    #[derive(Default)]
    pub struct InMemoryEntityStore {
        entities: RwLock<HashMap<Uuid, Entity>>,
    }

    impl InMemoryEntityStore {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl EntityStore for InMemoryEntityStore {
        fn get(&self, id: Uuid) -> Result<Option<Entity>, StorageError> {
            Ok(self.entities.read().unwrap().get(&id).cloned())
        }

        fn get_all(&self) -> Result<Vec<Entity>, StorageError> {
            Ok(self.entities.read().unwrap().values().cloned().collect())
        }

        fn upsert(&self, entity: Entity) -> Result<(), StorageError> {
            let mut entities = self.entities.write().unwrap();

            // Only update if newer
            if let Some(existing) = entities.get(&entity.id) {
                if entity.hlc <= existing.hlc {
                    return Ok(());
                }
            }

            entities.insert(entity.id, entity);
            Ok(())
        }

        fn delete(&self, id: Uuid) -> Result<bool, StorageError> {
            Ok(self.entities.write().unwrap().remove(&id).is_some())
        }
    }

    /// In-memory checkpoint store for testing.
    #[derive(Default)]
    pub struct InMemoryCheckpointStore {
        checkpoints: RwLock<HashMap<NodeId, ReplicationCheckpoint>>,
    }

    impl InMemoryCheckpointStore {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl CheckpointStore for InMemoryCheckpointStore {
        fn get_checkpoint(&self, peer: NodeId) -> Result<ReplicationCheckpoint, StorageError> {
            Ok(self
                .checkpoints
                .read()
                .unwrap()
                .get(&peer)
                .cloned()
                .unwrap_or_default())
        }

        fn save_checkpoint(
            &self,
            peer: NodeId,
            checkpoint: ReplicationCheckpoint,
        ) -> Result<(), StorageError> {
            self.checkpoints.write().unwrap().insert(peer, checkpoint);
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::event::{EventId, Payload};
        use crate::hlc::HlcTimestamp;

        fn make_event(node: u64, seq: u64) -> Event {
            Event::new(
                EventId::new(NodeId(node), seq),
                Uuid::new_v4(),
                HlcTimestamp::new(1000 + seq, 0, NodeId(node)),
                Payload::new(59.9, 10.7, "ship".to_string(), None),
            )
        }

        #[test]
        fn test_event_store_append() {
            let store = InMemoryEventStore::new();

            let event = make_event(1, 1);
            assert!(store.append(event.clone()).unwrap());

            // Duplicate should return false
            assert!(!store.append(event).unwrap());
        }

        #[test]
        fn test_event_store_get_events_since() {
            let store = InMemoryEventStore::new();

            store.append(make_event(1, 1)).unwrap();
            store.append(make_event(1, 2)).unwrap();
            store.append(make_event(1, 3)).unwrap();
            store.append(make_event(2, 1)).unwrap();

            let events = store.get_events_since(NodeId(1), 1).unwrap();
            assert_eq!(events.len(), 2);
        }

        #[test]
        fn test_entity_store_upsert() {
            let store = InMemoryEntityStore::new();

            let event1 = make_event(1, 1);
            let entity1 = Entity::from_event(&event1);
            store.upsert(entity1.clone()).unwrap();

            // Older event should not replace
            let mut older_event = event1.clone();
            older_event.hlc = HlcTimestamp::new(500, 0, NodeId(1));
            let older_entity = Entity::from_event(&older_event);
            store.upsert(older_entity).unwrap();

            let stored = store.get(entity1.id).unwrap().unwrap();
            assert_eq!(stored.hlc.wall_time, 1001); // Original timestamp
        }
    }
}
