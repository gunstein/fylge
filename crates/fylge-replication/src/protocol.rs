use serde::{Deserialize, Serialize};

use fylge_core::{Event, NodeId};

/// Request to pull events from a peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// The requesting node's ID.
    pub from_node: NodeId,
    /// Request events from this node.
    pub target_node: NodeId,
    /// Get events with sequence > since_seq.
    pub since_seq: u64,
    /// Maximum number of events to return.
    pub limit: Option<usize>,
}

impl PullRequest {
    pub fn new(from_node: NodeId, target_node: NodeId, since_seq: u64) -> Self {
        Self {
            from_node,
            target_node,
            since_seq,
            limit: None,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Response containing events from a peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResponse {
    /// The responding node's ID.
    pub from_node: NodeId,
    /// Events matching the request.
    pub events: Vec<Event>,
    /// Whether there are more events available.
    pub has_more: bool,
}

impl PullResponse {
    pub fn new(from_node: NodeId, events: Vec<Event>, has_more: bool) -> Self {
        Self {
            from_node,
            events,
            has_more,
        }
    }

    pub fn empty(from_node: NodeId) -> Self {
        Self {
            from_node,
            events: Vec::new(),
            has_more: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pull_request() {
        let req = PullRequest::new(NodeId(1), NodeId(2), 100).with_limit(50);

        assert_eq!(req.from_node, NodeId(1));
        assert_eq!(req.target_node, NodeId(2));
        assert_eq!(req.since_seq, 100);
        assert_eq!(req.limit, Some(50));
    }

    #[test]
    fn test_pull_response_empty() {
        let resp = PullResponse::empty(NodeId(1));

        assert!(resp.events.is_empty());
        assert!(!resp.has_more);
    }
}
