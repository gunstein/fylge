use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::ClockError;
use crate::node::NodeId;

/// Hybrid Logical Clock timestamp.
///
/// Provides causally consistent ordering across distributed nodes.
/// The ordering is: wall_time -> counter -> node_id (for deterministic tie-breaking).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HlcTimestamp {
    /// Physical time component (milliseconds since epoch)
    pub wall_time: u64,
    /// Logical counter for events at same wall_time
    pub counter: u32,
    /// Node ID for deterministic tie-breaking
    pub node_id: NodeId,
}

impl HlcTimestamp {
    pub fn new(wall_time: u64, counter: u32, node_id: NodeId) -> Self {
        Self {
            wall_time,
            counter,
            node_id,
        }
    }

    /// Create a zero timestamp (useful for initialization).
    pub fn zero(node_id: NodeId) -> Self {
        Self {
            wall_time: 0,
            counter: 0,
            node_id,
        }
    }
}

impl Ord for HlcTimestamp {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.wall_time
            .cmp(&other.wall_time)
            .then(self.counter.cmp(&other.counter))
            .then(self.node_id.cmp(&other.node_id))
    }
}

impl PartialOrd for HlcTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Hybrid Logical Clock for generating causally consistent timestamps.
pub struct Hlc {
    node_id: NodeId,
    last_timestamp: Mutex<HlcTimestamp>,
    /// Maximum allowed drift from local clock (milliseconds)
    max_drift_ms: u64,
}

impl Hlc {
    /// Create a new HLC for the given node.
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            last_timestamp: Mutex::new(HlcTimestamp::zero(node_id)),
            max_drift_ms: 60_000, // 1 minute default
        }
    }

    /// Create a new HLC with custom max drift.
    pub fn with_max_drift(node_id: NodeId, max_drift_ms: u64) -> Self {
        Self {
            node_id,
            last_timestamp: Mutex::new(HlcTimestamp::zero(node_id)),
            max_drift_ms,
        }
    }

    /// Generate a new timestamp for a local event.
    pub fn now(&self) -> Result<HlcTimestamp, ClockError> {
        let mut last = self.last_timestamp.lock().unwrap();
        let physical = system_time_millis();

        // Check for excessive drift (last timestamp is too far ahead of physical time)
        if last.wall_time > physical + self.max_drift_ms {
            return Err(ClockError::ExcessiveDrift(last.wall_time - physical));
        }

        let new_ts = if physical > last.wall_time {
            HlcTimestamp {
                wall_time: physical,
                counter: 0,
                node_id: self.node_id,
            }
        } else {
            HlcTimestamp {
                wall_time: last.wall_time,
                counter: last.counter.saturating_add(1),
                node_id: self.node_id,
            }
        };

        *last = new_ts;
        Ok(new_ts)
    }

    /// Update the clock based on a received remote timestamp.
    /// Returns a new timestamp that is causally after the remote timestamp.
    pub fn receive(&self, remote: HlcTimestamp) -> Result<HlcTimestamp, ClockError> {
        let mut last = self.last_timestamp.lock().unwrap();
        let physical = system_time_millis();

        // Check if remote clock is too far ahead
        if remote.wall_time > physical + self.max_drift_ms {
            return Err(ClockError::RemoteClockAhead(remote.wall_time - physical));
        }

        let max_wall = physical.max(last.wall_time).max(remote.wall_time);

        let new_ts =
            if max_wall == physical && physical > last.wall_time && physical > remote.wall_time {
                // Physical time has advanced past both
                HlcTimestamp {
                    wall_time: physical,
                    counter: 0,
                    node_id: self.node_id,
                }
            } else if max_wall == last.wall_time && last.wall_time == remote.wall_time {
                // All three are equal, increment counter
                HlcTimestamp {
                    wall_time: max_wall,
                    counter: last.counter.max(remote.counter).saturating_add(1),
                    node_id: self.node_id,
                }
            } else if max_wall == last.wall_time {
                // Last is ahead of remote
                HlcTimestamp {
                    wall_time: max_wall,
                    counter: last.counter.saturating_add(1),
                    node_id: self.node_id,
                }
            } else {
                // Remote is ahead
                HlcTimestamp {
                    wall_time: max_wall,
                    counter: remote.counter.saturating_add(1),
                    node_id: self.node_id,
                }
            };

        *last = new_ts;
        Ok(new_ts)
    }

    /// Get the node ID for this clock.
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}

fn system_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hlc_timestamp_ordering() {
        let t1 = HlcTimestamp::new(100, 0, NodeId(1));
        let t2 = HlcTimestamp::new(100, 1, NodeId(1));
        let t3 = HlcTimestamp::new(101, 0, NodeId(1));

        assert!(t1 < t2);
        assert!(t2 < t3);
    }

    #[test]
    fn test_hlc_timestamp_node_tiebreak() {
        let t1 = HlcTimestamp::new(100, 0, NodeId(1));
        let t2 = HlcTimestamp::new(100, 0, NodeId(2));

        assert!(t1 < t2);
    }

    #[test]
    fn test_hlc_now_increments() {
        let hlc = Hlc::new(NodeId(1));

        let t1 = hlc.now().unwrap();
        let t2 = hlc.now().unwrap();

        assert!(t1 < t2);
    }

    #[test]
    fn test_hlc_receive_advances_clock() {
        let hlc = Hlc::new(NodeId(1));

        // Create a remote timestamp far in the "future" (but within drift limit)
        let remote = HlcTimestamp::new(
            system_time_millis() + 1000, // 1 second ahead
            5,
            NodeId(2),
        );

        let local = hlc.receive(remote).unwrap();

        // Local should be after remote
        assert!(local > remote);
    }

    #[test]
    fn test_hlc_receive_rejects_excessive_drift() {
        let hlc = Hlc::with_max_drift(NodeId(1), 1000); // 1 second max drift

        let remote = HlcTimestamp::new(
            system_time_millis() + 10_000, // 10 seconds ahead
            0,
            NodeId(2),
        );

        assert!(hlc.receive(remote).is_err());
    }
}
