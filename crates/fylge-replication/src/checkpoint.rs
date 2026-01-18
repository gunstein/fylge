use fylge_core::{Event, NodeId, ReplicationCheckpoint};

/// Logic for managing replication checkpoints.
pub struct CheckpointManager;

impl CheckpointManager {
    /// Update a checkpoint with events, only committing contiguous sequences.
    ///
    /// This ensures we don't create "holes" in our replication state.
    /// For example, if we have events with seq [1, 2, 4, 5] and our checkpoint is at 0,
    /// we only update to seq 2 (not 5) because seq 3 is missing.
    pub fn update_contiguous(
        checkpoint: &mut ReplicationCheckpoint,
        node_id: NodeId,
        events: &[Event],
    ) {
        if events.is_empty() {
            return;
        }

        // Filter events for the target node and sort by sequence
        let mut node_events: Vec<_> = events.iter().filter(|e| e.id.node_id == node_id).collect();
        node_events.sort_by_key(|e| e.id.sequence);

        let current_seq = checkpoint.last_seq_for(node_id);
        let mut new_seq = current_seq;

        for event in node_events {
            if event.id.sequence == new_seq + 1 {
                new_seq = event.id.sequence;
            } else if event.id.sequence > new_seq + 1 {
                // Gap detected, stop here
                break;
            }
            // If event.id.sequence <= new_seq, it's already processed, skip
        }

        if new_seq > current_seq {
            checkpoint.update(node_id, new_seq);
        }
    }

    /// Get the next expected sequence number for a node.
    pub fn next_expected_seq(checkpoint: &ReplicationCheckpoint, node_id: NodeId) -> u64 {
        checkpoint.last_seq_for(node_id) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fylge_core::{EventId, HlcTimestamp, Payload};
    use uuid::Uuid;

    fn make_event(node: u64, seq: u64) -> Event {
        Event::new(
            EventId::new(NodeId(node), seq),
            Uuid::new_v4(),
            HlcTimestamp::new(1000 + seq, 0, NodeId(node)),
            Payload::new(59.9, 10.7, "ship".to_string(), None),
        )
    }

    #[test]
    fn test_update_contiguous_sequential() {
        let mut checkpoint = ReplicationCheckpoint::new();

        let events = vec![make_event(1, 1), make_event(1, 2), make_event(1, 3)];

        CheckpointManager::update_contiguous(&mut checkpoint, NodeId(1), &events);

        assert_eq!(checkpoint.last_seq_for(NodeId(1)), 3);
    }

    #[test]
    fn test_update_contiguous_with_gap() {
        let mut checkpoint = ReplicationCheckpoint::new();

        // Missing seq 3
        let events = vec![
            make_event(1, 1),
            make_event(1, 2),
            make_event(1, 4),
            make_event(1, 5),
        ];

        CheckpointManager::update_contiguous(&mut checkpoint, NodeId(1), &events);

        // Should only commit up to 2
        assert_eq!(checkpoint.last_seq_for(NodeId(1)), 2);
    }

    #[test]
    fn test_update_contiguous_continues_from_checkpoint() {
        let mut checkpoint = ReplicationCheckpoint::new();
        checkpoint.update(NodeId(1), 5);

        let events = vec![make_event(1, 6), make_event(1, 7), make_event(1, 8)];

        CheckpointManager::update_contiguous(&mut checkpoint, NodeId(1), &events);

        assert_eq!(checkpoint.last_seq_for(NodeId(1)), 8);
    }

    #[test]
    fn test_update_contiguous_ignores_old_events() {
        let mut checkpoint = ReplicationCheckpoint::new();
        checkpoint.update(NodeId(1), 5);

        let events = vec![
            make_event(1, 3), // Old
            make_event(1, 4), // Old
            make_event(1, 5), // Current
            make_event(1, 6), // New
        ];

        CheckpointManager::update_contiguous(&mut checkpoint, NodeId(1), &events);

        assert_eq!(checkpoint.last_seq_for(NodeId(1)), 6);
    }

    #[test]
    fn test_update_contiguous_empty_events() {
        let mut checkpoint = ReplicationCheckpoint::new();
        checkpoint.update(NodeId(1), 5);

        CheckpointManager::update_contiguous(&mut checkpoint, NodeId(1), &[]);

        assert_eq!(checkpoint.last_seq_for(NodeId(1)), 5);
    }

    #[test]
    fn test_next_expected_seq() {
        let mut checkpoint = ReplicationCheckpoint::new();
        assert_eq!(
            CheckpointManager::next_expected_seq(&checkpoint, NodeId(1)),
            1
        );

        checkpoint.update(NodeId(1), 10);
        assert_eq!(
            CheckpointManager::next_expected_seq(&checkpoint, NodeId(1)),
            11
        );
    }
}
