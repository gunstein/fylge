use sqlx::SqlitePool;
use std::collections::HashSet;
use std::sync::Arc;

use crate::models::Icon;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub icons: Arc<Vec<Icon>>,
    pub icon_ids: Arc<HashSet<String>>,
}

impl AppState {
    pub fn new(pool: SqlitePool, icons: Vec<Icon>) -> Self {
        let icon_ids: HashSet<String> = icons.iter().map(|i| i.id.clone()).collect();
        Self {
            pool,
            icons: Arc::new(icons),
            icon_ids: Arc::new(icon_ids),
        }
    }
}
