use serde::{Deserialize, Serialize};

/// Unique identifier for a node in the distributed system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node-{}", self.0)
    }
}

/// Configuration for a peer node.
#[derive(Debug, Clone)]
pub struct PeerConfig {
    pub node_id: NodeId,
    pub endpoint: String,
    pub pull_interval_secs: u64,
}

/// Configuration for this node.
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub id: NodeId,
    pub listen_addr: std::net::SocketAddr,
    pub db_path: std::path::PathBuf,
    pub peers: Vec<PeerConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_ordering() {
        let n1 = NodeId(1);
        let n2 = NodeId(2);
        assert!(n1 < n2);
    }

    #[test]
    fn test_node_id_display() {
        let n = NodeId(42);
        assert_eq!(n.to_string(), "node-42");
    }
}
