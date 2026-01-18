//! Fylge DB - redb implementation of storage traits.

pub mod checkpoint_store;
pub mod entity_store;
pub mod event_store;
pub mod tables;

pub use checkpoint_store::RedbCheckpointStore;
pub use entity_store::RedbEntityStore;
pub use event_store::RedbEventStore;

use std::path::Path;
use std::sync::Arc;

use redb::Database;

use fylge_core::StorageError;

/// Initialize a database with all required tables.
pub fn init_database(path: impl AsRef<Path>) -> Result<Arc<Database>, StorageError> {
    let db = Database::create(path).map_err(|e| StorageError::Database(e.to_string()))?;

    RedbEventStore::init_tables(&db)?;
    RedbEntityStore::init_tables(&db)?;
    RedbCheckpointStore::init_tables(&db)?;

    Ok(Arc::new(db))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init_database() {
        let dir = tempdir().unwrap();
        let db = init_database(dir.path().join("test.redb")).unwrap();

        // Verify we can create stores
        let _event_store = RedbEventStore::new(db.clone());
        let _entity_store = RedbEntityStore::new(db.clone());
        let _checkpoint_store = RedbCheckpointStore::new(db);
    }
}
