use std::sync::Arc;
use std::time::Duration;

use tokio::time::sleep;

use fylge_core::{
    CheckpointStore, Entity, EntityStore, EventStore, NodeId, PeerConfig, StorageError,
};

use crate::checkpoint::CheckpointManager;
use crate::materializer::EntityMaterializer;
use crate::protocol::{PullRequest, PullResponse};

/// Error type for replication operations.
#[derive(Debug)]
pub enum ReplicationError {
    Storage(StorageError),
    Network(String),
}

impl From<StorageError> for ReplicationError {
    fn from(e: StorageError) -> Self {
        ReplicationError::Storage(e)
    }
}

/// Trait for fetching events from a remote peer.
pub trait PeerClient: Send + Sync {
    fn pull_events(
        &self,
        request: PullRequest,
    ) -> impl std::future::Future<Output = Result<PullResponse, ReplicationError>> + Send;
}

/// Pull-based replicator that syncs events from peers.
pub struct PullReplicator<E, S, C, P>
where
    E: EventStore,
    S: EntityStore,
    C: CheckpointStore,
    P: PeerClient,
{
    node_id: NodeId,
    event_store: Arc<E>,
    entity_store: Arc<S>,
    checkpoint_store: Arc<C>,
    peer_client: Arc<P>,
    peers: Vec<PeerConfig>,
}

impl<E, S, C, P> PullReplicator<E, S, C, P>
where
    E: EventStore,
    S: EntityStore,
    C: CheckpointStore,
    P: PeerClient,
{
    pub fn new(
        node_id: NodeId,
        event_store: Arc<E>,
        entity_store: Arc<S>,
        checkpoint_store: Arc<C>,
        peer_client: Arc<P>,
        peers: Vec<PeerConfig>,
    ) -> Self {
        Self {
            node_id,
            event_store,
            entity_store,
            checkpoint_store,
            peer_client,
            peers,
        }
    }

    /// Sync once from all peers.
    pub async fn sync_once(&self) -> Result<SyncStats, ReplicationError> {
        let mut stats = SyncStats::default();

        for peer in &self.peers {
            match self.sync_from_peer(peer).await {
                Ok(peer_stats) => {
                    stats.events_received += peer_stats.events_received;
                    stats.entities_updated += peer_stats.entities_updated;
                    stats.peers_synced += 1;
                }
                Err(e) => {
                    stats.peers_failed += 1;
                    // Log error but continue with other peers
                    tracing::warn!("Failed to sync from peer {:?}: {:?}", peer.node_id, e);
                }
            }
        }

        Ok(stats)
    }

    /// Sync from a single peer.
    async fn sync_from_peer(&self, peer: &PeerConfig) -> Result<SyncStats, ReplicationError> {
        let mut stats = SyncStats::default();

        // Get current checkpoint for this peer
        let mut checkpoint = self.checkpoint_store.get_checkpoint(peer.node_id)?;
        let since_seq = checkpoint.last_seq_for(peer.node_id);

        // Request events
        let request = PullRequest::new(self.node_id, peer.node_id, since_seq);
        let response = self.peer_client.pull_events(request).await?;

        if response.events.is_empty() {
            return Ok(stats);
        }

        stats.events_received = response.events.len();

        // Store events (idempotent)
        for event in &response.events {
            self.event_store.append(event.clone())?;
        }

        // Update entities
        for event in &response.events {
            let entity = Entity::from_event(event);

            // Check if we should update
            if let Some(existing) = self.entity_store.get(event.entity_id)? {
                if !EntityMaterializer::should_replace(&existing, event) {
                    continue;
                }
            }

            self.entity_store.upsert(entity)?;
            stats.entities_updated += 1;
        }

        // Update checkpoint (only contiguous sequences)
        CheckpointManager::update_contiguous(&mut checkpoint, peer.node_id, &response.events);
        self.checkpoint_store
            .save_checkpoint(peer.node_id, checkpoint)?;

        Ok(stats)
    }

    /// Run continuous sync loop.
    pub async fn run(&self, interval: Duration) {
        loop {
            match self.sync_once().await {
                Ok(stats) => {
                    if stats.events_received > 0 {
                        tracing::info!(
                            "Synced {} events, updated {} entities from {} peers",
                            stats.events_received,
                            stats.entities_updated,
                            stats.peers_synced
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Sync error: {:?}", e);
                }
            }

            sleep(interval).await;
        }
    }
}

/// Statistics from a sync operation.
#[derive(Debug, Default)]
pub struct SyncStats {
    pub events_received: usize,
    pub entities_updated: usize,
    pub peers_synced: usize,
    pub peers_failed: usize,
}

/// HTTP peer client that knows about peer endpoints.
pub struct HttpPeerClientWithEndpoints {
    client: reqwest::Client,
    endpoints: std::collections::HashMap<NodeId, String>,
}

impl HttpPeerClientWithEndpoints {
    pub fn new(peers: &[PeerConfig]) -> Self {
        let mut endpoints = std::collections::HashMap::new();
        for peer in peers {
            endpoints.insert(peer.node_id, peer.endpoint.clone());
        }
        Self {
            client: reqwest::Client::new(),
            endpoints,
        }
    }
}

impl PeerClient for HttpPeerClientWithEndpoints {
    async fn pull_events(&self, request: PullRequest) -> Result<PullResponse, ReplicationError> {
        let endpoint = self.endpoints.get(&request.target_node).ok_or_else(|| {
            ReplicationError::Network(format!("No endpoint for node {:?}", request.target_node))
        })?;

        let url = format!(
            "{}/replication/events?since_seq={}",
            endpoint.trim_end_matches('/'),
            request.since_seq
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ReplicationError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ReplicationError::Network(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let pull_response: PullResponse = response
            .json()
            .await
            .map_err(|e| ReplicationError::Network(e.to_string()))?;

        Ok(pull_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fylge_core::{
        Event, EventId, HlcTimestamp, InMemoryCheckpointStore, InMemoryEntityStore,
        InMemoryEventStore, Payload,
    };
    use std::collections::HashMap;
    use std::sync::RwLock;
    use uuid::Uuid;

    struct MockPeerClient {
        responses: RwLock<HashMap<NodeId, Vec<Event>>>,
    }

    impl MockPeerClient {
        fn new() -> Self {
            Self {
                responses: RwLock::new(HashMap::new()),
            }
        }

        fn add_events(&self, node_id: NodeId, events: Vec<Event>) {
            self.responses.write().unwrap().insert(node_id, events);
        }
    }

    impl PeerClient for MockPeerClient {
        async fn pull_events(
            &self,
            request: PullRequest,
        ) -> Result<PullResponse, ReplicationError> {
            let responses = self.responses.read().unwrap();
            let events = responses
                .get(&request.target_node)
                .map(|evts| {
                    evts.iter()
                        .filter(|e| e.id.sequence > request.since_seq)
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();

            Ok(PullResponse::new(request.target_node, events, false))
        }
    }

    fn make_event(node: u64, seq: u64, entity_id: Uuid) -> Event {
        Event::new(
            EventId::new(NodeId(node), seq),
            entity_id,
            HlcTimestamp::new(1000 + seq, 0, NodeId(node)),
            Payload::new(59.9, 10.7, "ship".to_string(), None),
        )
    }

    #[tokio::test]
    async fn test_sync_from_peer() {
        let event_store = Arc::new(InMemoryEventStore::new());
        let entity_store = Arc::new(InMemoryEntityStore::new());
        let checkpoint_store = Arc::new(InMemoryCheckpointStore::new());
        let peer_client = Arc::new(MockPeerClient::new());

        let entity_id = Uuid::new_v4();
        peer_client.add_events(
            NodeId(2),
            vec![make_event(2, 1, entity_id), make_event(2, 2, entity_id)],
        );

        let peers = vec![PeerConfig {
            node_id: NodeId(2),
            endpoint: "http://localhost:3002".to_string(),
            pull_interval_secs: 5,
        }];

        let replicator = PullReplicator::new(
            NodeId(1),
            event_store.clone(),
            entity_store.clone(),
            checkpoint_store.clone(),
            peer_client,
            peers,
        );

        let stats = replicator.sync_once().await.unwrap();

        assert_eq!(stats.events_received, 2);
        assert_eq!(stats.entities_updated, 2);
        assert_eq!(stats.peers_synced, 1);

        // Check entity was stored
        let entity = entity_store.get(entity_id).unwrap().unwrap();
        assert_eq!(entity.hlc.wall_time, 1002); // Latest event

        // Check checkpoint was updated
        let checkpoint = checkpoint_store.get_checkpoint(NodeId(2)).unwrap();
        assert_eq!(checkpoint.last_seq_for(NodeId(2)), 2);
    }
}
