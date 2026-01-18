use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use fylge_core::EventStore;
use fylge_replication::protocol::PullResponse;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/replication/events", get(get_events))
}

#[derive(Deserialize)]
pub struct EventsQuery {
    /// Get events with sequence > since_seq (default 0).
    #[serde(default)]
    since_seq: u64,
    /// Maximum number of events to return (default 100).
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    100
}

async fn get_events(
    State(state): State<AppState>,
    Query(query): Query<EventsQuery>,
) -> Json<PullResponse> {
    let events = state
        .event_store
        .get_events_since(state.node_id, query.since_seq)
        .unwrap_or_default();

    let has_more = events.len() > query.limit;
    let events: Vec<_> = events.into_iter().take(query.limit).collect();

    Json(PullResponse::new(state.node_id, events, has_more))
}
