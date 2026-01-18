use std::sync::Arc;

use fylge_core::{Hlc, Icon, NodeId};
use fylge_db::{RedbEntityStore, RedbEventStore};

use crate::middleware::{read_limiter, write_limiter, RateLimiter};

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub node_id: NodeId,
    pub hlc: Arc<Hlc>,
    pub event_store: Arc<RedbEventStore>,
    pub entity_store: Arc<RedbEntityStore>,
    pub icons: Arc<Vec<Icon>>,
    pub write_limiter: Arc<RateLimiter>,
    pub read_limiter: Arc<RateLimiter>,
}

impl AppState {
    pub fn new(
        node_id: NodeId,
        event_store: Arc<RedbEventStore>,
        entity_store: Arc<RedbEntityStore>,
        icons: Vec<Icon>,
    ) -> Self {
        Self {
            node_id,
            hlc: Arc::new(Hlc::new(node_id)),
            event_store,
            entity_store,
            icons: Arc::new(icons),
            write_limiter: Arc::new(write_limiter()),
            read_limiter: Arc::new(read_limiter()),
        }
    }
}
