use std::sync::Arc;

use redb::Database;

use fylge_core::{CheckpointStore, NodeId, ReplicationCheckpoint, StorageError};

use crate::tables::CHECKPOINTS_TABLE;

/// redb implementation of CheckpointStore.
pub struct RedbCheckpointStore {
    db: Arc<Database>,
}

impl RedbCheckpointStore {
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
                .open_table(CHECKPOINTS_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;
        Ok(())
    }
}

impl CheckpointStore for RedbCheckpointStore {
    fn get_checkpoint(&self, peer: NodeId) -> Result<ReplicationCheckpoint, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let table = read_txn
            .open_table(CHECKPOINTS_TABLE)
            .map_err(|e| StorageError::Database(e.to_string()))?;

        let key = peer.0.to_be_bytes();
        match table
            .get(key.as_slice())
            .map_err(|e| StorageError::Database(e.to_string()))?
        {
            Some(value) => {
                let checkpoint: ReplicationCheckpoint = serde_json::from_slice(value.value())
                    .map_err(|e| StorageError::Database(e.to_string()))?;
                Ok(checkpoint)
            }
            None => Ok(ReplicationCheckpoint::new()),
        }
    }

    fn save_checkpoint(
        &self,
        peer: NodeId,
        checkpoint: ReplicationCheckpoint,
    ) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        {
            let mut table = write_txn
                .open_table(CHECKPOINTS_TABLE)
                .map_err(|e| StorageError::Database(e.to_string()))?;

            let key = peer.0.to_be_bytes();
            let value = serde_json::to_vec(&checkpoint)
                .map_err(|e| StorageError::Database(e.to_string()))?;

            table
                .insert(key.as_slice(), value.as_slice())
                .map_err(|e| StorageError::Database(e.to_string()))?;
        }

        write_txn
            .commit()
            .map_err(|e| StorageError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_db() -> Arc<Database> {
        let dir = tempdir().unwrap();
        let db = Database::create(dir.path().join("test.redb")).unwrap();
        RedbCheckpointStore::init_tables(&db).unwrap();
        Arc::new(db)
    }

    #[test]
    fn test_save_and_get_checkpoint() {
        let db = create_test_db();
        let store = RedbCheckpointStore::new(db);

        let peer = NodeId(2);
        let mut checkpoint = ReplicationCheckpoint::new();
        checkpoint.update(NodeId(1), 100);
        checkpoint.update(NodeId(3), 50);

        store.save_checkpoint(peer, checkpoint.clone()).unwrap();

        let retrieved = store.get_checkpoint(peer).unwrap();
        assert_eq!(retrieved.last_seq_for(NodeId(1)), 100);
        assert_eq!(retrieved.last_seq_for(NodeId(3)), 50);
    }

    #[test]
    fn test_get_nonexistent_returns_empty() {
        let db = create_test_db();
        let store = RedbCheckpointStore::new(db);

        let checkpoint = store.get_checkpoint(NodeId(99)).unwrap();
        assert_eq!(checkpoint.last_seq_for(NodeId(1)), 0);
    }
}
