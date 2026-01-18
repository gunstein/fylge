//! Fylge Core - Domain models, traits, and validation.
//!
//! This crate contains the core domain logic for the Fylge distributed
//! marker system. It has no dependencies on other Fylge crates.

pub mod entity;
pub mod error;
pub mod event;
pub mod hlc;
pub mod icon;
pub mod node;
pub mod storage;
pub mod validation;

// Re-exports for convenience
pub use entity::Entity;
pub use error::{ClockError, CoreError, StorageError, ValidationError};
pub use event::{Event, EventId, MarkerData, Payload};
pub use hlc::{Hlc, HlcTimestamp};
pub use icon::Icon;
pub use node::{NodeConfig, NodeId, PeerConfig};
pub use storage::{AppendResult, CheckpointStore, EntityStore, EventStore, ReplicationCheckpoint};
pub use validation::Validator;

#[cfg(any(test, feature = "test-utils"))]
pub use storage::memory::{InMemoryCheckpointStore, InMemoryEntityStore, InMemoryEventStore};
