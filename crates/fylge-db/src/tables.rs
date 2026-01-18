use redb::TableDefinition;

/// Table for storing events.
/// Key: (node_id, sequence) as bytes
/// Value: serialized Event as bytes
pub const EVENTS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("events");

/// Table for storing materialized entities.
/// Key: entity UUID as bytes
/// Value: serialized Entity as bytes
pub const ENTITIES_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("entities");

/// Table for storing replication checkpoints.
/// Key: peer node_id as bytes
/// Value: serialized ReplicationCheckpoint as bytes
pub const CHECKPOINTS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("checkpoints");

/// Table for storing sequence counters per node.
/// Key: node_id as u64
/// Value: last sequence number as u64
pub const SEQUENCES_TABLE: TableDefinition<u64, u64> = TableDefinition::new("sequences");

/// Encode an event key (node_id, sequence) to bytes.
pub fn encode_event_key(node_id: u64, sequence: u64) -> [u8; 16] {
    let mut key = [0u8; 16];
    key[..8].copy_from_slice(&node_id.to_be_bytes());
    key[8..].copy_from_slice(&sequence.to_be_bytes());
    key
}

/// Decode an event key from bytes.
pub fn decode_event_key(bytes: &[u8]) -> (u64, u64) {
    let node_id = u64::from_be_bytes(bytes[..8].try_into().unwrap());
    let sequence = u64::from_be_bytes(bytes[8..].try_into().unwrap());
    (node_id, sequence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_key_roundtrip() {
        let key = encode_event_key(42, 100);
        let (node_id, seq) = decode_event_key(&key);
        assert_eq!(node_id, 42);
        assert_eq!(seq, 100);
    }

    #[test]
    fn test_event_key_ordering() {
        // Keys should sort by node_id first, then sequence
        let k1 = encode_event_key(1, 100);
        let k2 = encode_event_key(1, 101);
        let k3 = encode_event_key(2, 1);

        assert!(k1 < k2);
        assert!(k2 < k3);
    }
}
