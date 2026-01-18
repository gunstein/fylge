pub mod api;
pub mod health;
pub mod markers;
pub mod pages;
pub mod replication;

use axum::Router;

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(pages::routes())
        .merge(api::routes())
        .merge(markers::routes())
        .merge(health::routes())
        .merge(replication::routes())
        .with_state(state)
}
