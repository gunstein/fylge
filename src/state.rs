use sqlx::SqlitePool;

use crate::models::Icon;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub icons: Vec<Icon>,
}

impl AppState {
    pub fn new(pool: SqlitePool, icons: Vec<Icon>) -> Self {
        Self { pool, icons }
    }
}
