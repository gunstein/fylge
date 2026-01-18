use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Clock error: {0}")]
    Clock(#[from] ClockError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

#[derive(Error, Debug, PartialEq)]
pub enum ValidationError {
    #[error("Invalid latitude {0}: must be between -90 and 90")]
    InvalidLatitude(f64),

    #[error("Invalid longitude {0}: must be between -180 and 180")]
    InvalidLongitude(f64),

    #[error("Invalid icon_id: {0}")]
    InvalidIconId(String),

    #[error("Icon not found: {0}")]
    IconNotFound(String),

    #[error("Label too long: {0} characters (max 256)")]
    LabelTooLong(usize),
}

#[derive(Error, Debug)]
pub enum ClockError {
    #[error("Excessive clock drift detected: local time is behind by {0}ms")]
    ExcessiveDrift(u64),

    #[error("Remote clock is too far ahead: {0}ms")]
    RemoteClockAhead(u64),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Event not found: node={0}, seq={1}")]
    EventNotFound(u64, u64),

    #[error("Entity not found: {0}")]
    EntityNotFound(uuid::Uuid),

    #[error("Duplicate event: node={0}, seq={1}")]
    DuplicateEvent(u64, u64),

    #[error("Database error: {0}")]
    Database(String),
}
