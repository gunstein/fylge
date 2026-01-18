use std::sync::Arc;

use redb::{Database, ReadableTable};
use uuid::Uuid;

use fylge_core::{
    AppendResult, Event, EventId, EventStore, HlcTimestamp, NodeId, Payload, StorageError,
};

use crate::tables::{decode_event_key, encode_event_key, EVENTS_TABLE, SEQUENCES_TABLE};

/// redb implementation of EventStore.
pub struct RedbEventStore {
    db: Arc<Database>,
}

impl RedbEventStore {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Initialize the database tables.
    pub fn init_tables(db: &Database) -> Result<(), StorageError> {
        let write_txn = db
            .begin_write()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        {
            // Create tables if they don't exist
            let _ = write_txn
                .open_table(EVENTS_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;
            let _ = write_txn
                .open_table(SEQUENCES_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(())
    }
}

impl EventStore for RedbEventStore {
    fn append(&self, event: Event) -> Result<bool, StorageError> {
        let key = encode_event_key(event.id.node_id.0, event.id.sequence);
        let value =
            serde_json::to_vec(&event).map_err(|e| StorageError::Database(e.to_string()))?;

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        {
            let mut table = write_txn
                .open_table(EVENTS_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;

            // Check if event already exists
            if table
                .get(key.as_slice())
                .map_err(|e| StorageError::Database(e.to_string()))?
                .is_some()
            {
                return Ok(false);
            }

            table
                .insert(key.as_slice(), value.as_slice())
                .map_err(|e| StorageError::Database(e.to_string()))?;

            // Update sequence counter
            let mut seq_table = write_txn
                .open_table(SEQUENCES_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let current_seq = seq_table
                .get(event.id.node_id.0)
                .map_err(|e| StorageError::Database(e.to_string()))?
                .map(|v| v.value())
                .unwrap_or(0);

            if event.id.sequence > current_seq {
                seq_table
                    .insert(event.id.node_id.0, event.id.sequence)
                    .map_err(|e| StorageError::Database(e.to_string()))?;
            }
        }

        write_txn
            .commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(true)
    }

    fn get_events_for_entity(&self, entity_id: Uuid) -> Result<Vec<Event>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let table = read_txn
            .open_table(EVENTS_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut events = Vec::new();
        for entry in table
            .iter()
            .map_err(|e| StorageError::Database(e.to_string()))?
        {
            let (_, value) = entry.map_err(|e| StorageError::Database(e.to_string()))?;
            let event: Event = serde_json::from_slice(value.value())
                .map_err(|e| StorageError::Database(e.to_string()))?;
            if event.entity_id == entity_id {
                events.push(event);
            }
        }

        Ok(events)
    }

    fn get_events_since(
        &self,
        node_id: NodeId,
        since_seq: u64,
    ) -> Result<Vec<Event>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let table = read_txn
            .open_table(EVENTS_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        // Create range start key
        let start_key = encode_event_key(node_id.0, since_seq + 1);
        // End key is start of next node
        let end_key = encode_event_key(node_id.0 + 1, 0);

        let mut events = Vec::new();
        let range = table
            .range(start_key.as_slice()..end_key.as_slice())
            .map_err(|e| StorageError::Database(e.to_string()))?;

        for entry in range {
            let (key, value) = entry.map_err(|e| StorageError::Database(e.to_string()))?;
            let (key_node_id, _) = decode_event_key(key.value());

            // Only include events from the requested node
            if key_node_id == node_id.0 {
                let event: Event = serde_json::from_slice(value.value())
                    .map_err(|e| StorageError::Database(e.to_string()))?;
                events.push(event);
            }
        }

        Ok(events)
    }

    fn get_all_events(&self) -> Result<Vec<Event>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let table = read_txn
            .open_table(EVENTS_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut events = Vec::new();
        for entry in table
            .iter()
            .map_err(|e| StorageError::Database(e.to_string()))?
        {
            let (_, value) = entry.map_err(|e| StorageError::Database(e.to_string()))?;
            let event: Event = serde_json::from_slice(value.value())
                .map_err(|e| StorageError::Database(e.to_string()))?;
            events.push(event);
        }

        Ok(events)
    }

    fn next_sequence(&self, node_id: NodeId) -> Result<u64, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let table = read_txn
            .open_table(SEQUENCES_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let current = table
            .get(node_id.0)
            .map_err(|e| StorageError::Database(e.to_string()))?
            .map(|v| v.value())
            .unwrap_or(0);

        Ok(current + 1)
    }

    fn append_local(
        &self,
        node_id: NodeId,
        entity_id: Uuid,
        hlc: HlcTimestamp,
        payload: Payload,
    ) -> Result<AppendResult, StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let (event, sequence) = {
            // Get current sequence and increment atomically
            let mut seq_table = write_txn
                .open_table(SEQUENCES_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let current_seq = seq_table
                .get(node_id.0)
                .map_err(|e| StorageError::Database(e.to_string()))?
                .map(|v| v.value())
                .unwrap_or(0);

            let sequence = current_seq + 1;

            // Create event
            let event = Event::new(EventId::new(node_id, sequence), entity_id, hlc, payload);

            // Store event
            let key = encode_event_key(node_id.0, sequence);
            let value =
                serde_json::to_vec(&event).map_err(|e| StorageError::Database(e.to_string()))?;

            let mut events_table = write_txn
                .open_table(EVENTS_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;

            events_table
                .insert(key.as_slice(), value.as_slice())
                .map_err(|e| StorageError::Database(e.to_string()))?;

            // Update sequence counter
            seq_table
                .insert(node_id.0, sequence)
                .map_err(|e| StorageError::Database(e.to_string()))?;

            (event, sequence)
        };

        write_txn
            .commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(AppendResult { event, sequence })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fylge_core::{EventId, HlcTimestamp, Payload};
    use tempfile::tempdir;

    fn create_test_db() -> Arc<Database> {
        let dir = tempdir().unwrap();
        let db = Database::create(dir.path().join("test.redb")).unwrap();
        RedbEventStore::init_tables(&db).unwrap();
        Arc::new(db)
    }

    fn make_event(node: u64, seq: u64, entity_id: Uuid) -> Event {
        Event::new(
            EventId::new(NodeId(node), seq),
            entity_id,
            HlcTimestamp::new(1000 + seq, 0, NodeId(node)),
            Payload::new(59.9, 10.7, "ship".to_string(), None),
        )
    }

    #[test]
    fn test_append_and_retrieve() {
        let db = create_test_db();
        let store = RedbEventStore::new(db);

        let entity_id = Uuid::new_v4();
        let event = make_event(1, 1, entity_id);

        assert!(store.append(event.clone()).unwrap());

        let events = store.get_events_for_entity(entity_id).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id.sequence, 1);
    }

    #[test]
    fn test_duplicate_rejected() {
        let db = create_test_db();
        let store = RedbEventStore::new(db);

        let event = make_event(1, 1, Uuid::new_v4());

        assert!(store.append(event.clone()).unwrap());
        assert!(!store.append(event).unwrap());
    }

    #[test]
    fn test_get_events_since() {
        let db = create_test_db();
        let store = RedbEventStore::new(db);

        let entity_id = Uuid::new_v4();
        store.append(make_event(1, 1, entity_id)).unwrap();
        store.append(make_event(1, 2, entity_id)).unwrap();
        store.append(make_event(1, 3, entity_id)).unwrap();
        store.append(make_event(2, 1, Uuid::new_v4())).unwrap();

        let events = store.get_events_since(NodeId(1), 1).unwrap();
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|e| e.id.sequence > 1));
    }

    #[test]
    fn test_next_sequence() {
        let db = create_test_db();
        let store = RedbEventStore::new(db);

        assert_eq!(store.next_sequence(NodeId(1)).unwrap(), 1);

        store.append(make_event(1, 1, Uuid::new_v4())).unwrap();
        assert_eq!(store.next_sequence(NodeId(1)).unwrap(), 2);

        store.append(make_event(1, 2, Uuid::new_v4())).unwrap();
        assert_eq!(store.next_sequence(NodeId(1)).unwrap(), 3);
    }
}
