use std::net::SocketAddr;
use std::path::PathBuf;

use fylge_core::{NodeId, PeerConfig};

/// Server configuration, loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub node_id: NodeId,
    pub listen_addr: SocketAddr,
    pub db_path: PathBuf,
    pub peers: Vec<PeerConfig>,
    pub pull_interval_secs: u64,
}

impl Config {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        let node_id = std::env::var("FYLGE_NODE_ID")
            .map_err(|_| ConfigError::Missing("FYLGE_NODE_ID"))?
            .parse::<u64>()
            .map_err(|_| ConfigError::Invalid("FYLGE_NODE_ID", "must be a valid u64"))?;

        let listen_addr = std::env::var("FYLGE_LISTEN_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
            .parse()
            .map_err(|_| {
                ConfigError::Invalid("FYLGE_LISTEN_ADDR", "must be a valid socket address")
            })?;

        let db_path = std::env::var("FYLGE_DB_PATH")
            .unwrap_or_else(|_| "./fylge.redb".to_string())
            .into();

        let pull_interval_secs = std::env::var("FYLGE_PULL_INTERVAL_SECS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);

        let peers = Self::parse_peers()?;

        Ok(Config {
            node_id: NodeId(node_id),
            listen_addr,
            db_path,
            peers,
            pull_interval_secs,
        })
    }

    fn parse_peers() -> Result<Vec<PeerConfig>, ConfigError> {
        let peers_str = match std::env::var("FYLGE_PEERS") {
            Ok(s) if !s.is_empty() => s,
            _ => return Ok(Vec::new()),
        };

        let mut peers = Vec::new();
        for peer_entry in peers_str.split(',') {
            let peer_entry = peer_entry.trim();
            if peer_entry.is_empty() {
                continue;
            }

            // Expected format: "node_id@endpoint" e.g. "2@http://localhost:3002"
            let (node_id, endpoint) = if let Some(at_pos) = peer_entry.find('@') {
                let node_id_str = &peer_entry[..at_pos];
                let endpoint = &peer_entry[at_pos + 1..];

                let node_id = node_id_str.parse::<u64>().map_err(|_| {
                    ConfigError::Invalid(
                        "FYLGE_PEERS",
                        "node_id must be a valid u64 (format: node_id@endpoint)",
                    )
                })?;

                (NodeId(node_id), endpoint.to_string())
            } else {
                return Err(ConfigError::Invalid(
                    "FYLGE_PEERS",
                    "expected format: node_id@endpoint (e.g. 2@http://localhost:3002)",
                ));
            };

            peers.push(PeerConfig {
                node_id,
                endpoint,
                pull_interval_secs: 5,
            });
        }

        Ok(peers)
    }

    /// Create a test configuration.
    #[cfg(test)]
    pub fn for_testing() -> Self {
        Config {
            node_id: NodeId(1),
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            db_path: PathBuf::from("/tmp/fylge-test.redb"),
            peers: Vec::new(),
            pull_interval_secs: 5,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Missing(&'static str),
    Invalid(&'static str, &'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Missing(var) => {
                write!(f, "Missing required environment variable: {}", var)
            }
            ConfigError::Invalid(var, msg) => write!(f, "Invalid value for {}: {}", var, msg),
        }
    }
}

impl std::error::Error for ConfigError {}
