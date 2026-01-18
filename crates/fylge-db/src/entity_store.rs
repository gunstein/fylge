use std::sync::Arc;

use redb::{Database, ReadableTable};
use uuid::Uuid;

use fylge_core::{Entity, EntityStore, StorageError};

use crate::tables::ENTITIES_TABLE;

/// redb implementation of EntityStore.
pub struct RedbEntityStore {
    db: Arc<Database>,
}

impl RedbEntityStore {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Initialize the database tables.
    pub fn init_tables(db: &Database) -> Result<(), StorageError> {
        let write_txn = db
            .begin_write()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        {
            let _ = write_txn
                .open_table(ENTITIES_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(())
    }
}

impl EntityStore for RedbEntityStore {
    fn get(&self, id: Uuid) -> Result<Option<Entity>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let table = read_txn
            .open_table(ENTITIES_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let key = id.as_bytes().as_slice();
        match table
            .get(key)
            .map_err(|e| StorageError::Database(e.to_string()))?
        {
            Some(value) => {
                let entity: Entity = serde_json::from_slice(value.value())
                    .map_err(|e| StorageError::Database(e.to_string()))?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    fn get_all(&self) -> Result<Vec<Entity>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let table = read_txn
            .open_table(ENTITIES_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let mut entities = Vec::new();
        for entry in table
            .iter()
            .map_err(|e| StorageError::Database(e.to_string()))?
        {
            let (_, value) = entry.map_err(|e| StorageError::Database(e.to_string()))?;
            let entity: Entity = serde_json::from_slice(value.value())
                .map_err(|e| StorageError::Database(e.to_string()))?;
            entities.push(entity);
        }

        Ok(entities)
    }

    fn upsert(&self, entity: Entity) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        {
            let mut table = write_txn
                .open_table(ENTITIES_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let key = entity.id.as_bytes().as_slice();

            // Check if existing entity is newer
            if let Some(existing_value) = table
                .get(key)
                .map_err(|e| StorageError::Database(e.to_string()))?
            {
                let existing: Entity = serde_json::from_slice(existing_value.value())
                    .map_err(|e| StorageError::Database(e.to_string()))?;
                if entity.hlc <= existing.hlc {
                    return Ok(());
                }
            }

            let value =
                serde_json::to_vec(&entity).map_err(|e| StorageError::Database(e.to_string()))?;
            table
                .insert(key, value.as_slice())
                .map_err(|e| StorageError::Database(e.to_string()))?;
        }

        write_txn
            .commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }

    fn delete(&self, id: Uuid) -> Result<bool, StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let removed;
        {
            let mut table = write_txn
                .open_table(ENTITIES_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let key = id.as_bytes().as_slice();
            let result = table
                .remove(key)
                .map_err(|e| StorageError::Database(e.to_string()))?;
            removed = result.is_some();
        }

        write_txn
            .commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fylge_core::{Event, EventId, HlcTimestamp, NodeId, Payload};
    use tempfile::tempdir;

    fn create_test_db() -> Arc<Database> {
        let dir = tempdir().unwrap();
        let db = Database::create(dir.path().join("test.redb")).unwrap();
        RedbEntityStore::init_tables(&db).unwrap();
        Arc::new(db)
    }

    fn make_entity(id: Uuid, wall_time: u64) -> Entity {
        let event = Event::new(
            EventId::new(NodeId(1), 1),
            id,
            HlcTimestamp::new(wall_time, 0, NodeId(1)),
            Payload::new(59.9, 10.7, "ship".to_string(), Some("Oslo".to_string())),
        );
        Entity::from_event(&event)
    }

    #[test]
    fn test_upsert_and_get() {
        let db = create_test_db();
        let store = RedbEntityStore::new(db);

        let id = Uuid::new_v4();
        let entity = make_entity(id, 1000);

        store.upsert(entity.clone()).unwrap();

        let retrieved = store.get(id).unwrap().unwrap();
        assert_eq!(retrieved.id, id);
        assert_eq!(retrieved.lat, 59.9);
        assert_eq!(retrieved.label, Some("Oslo".to_string()));
    }

    #[test]
    fn test_upsert_older_ignored() {
        let db = create_test_db();
        let store = RedbEntityStore::new(db);

        let id = Uuid::new_v4();
        let newer = make_entity(id, 2000);
        let older = make_entity(id, 1000);

        store.upsert(newer.clone()).unwrap();
        store.upsert(older).unwrap();

        let retrieved = store.get(id).unwrap().unwrap();
        assert_eq!(retrieved.hlc.wall_time, 2000);
    }

    #[test]
    fn test_get_all() {
        let db = create_test_db();
        let store = RedbEntityStore::new(db);

        store.upsert(make_entity(Uuid::new_v4(), 1000)).unwrap();
        store.upsert(make_entity(Uuid::new_v4(), 1001)).unwrap();
        store.upsert(make_entity(Uuid::new_v4(), 1002)).unwrap();

        let all = store.get_all().unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_delete() {
        let db = create_test_db();
        let store = RedbEntityStore::new(db);

        let id = Uuid::new_v4();
        store.upsert(make_entity(id, 1000)).unwrap();

        assert!(store.delete(id).unwrap());
        assert!(store.get(id).unwrap().is_none());

        // Deleting non-existent returns false
        assert!(!store.delete(Uuid::new_v4()).unwrap());
    }
}
