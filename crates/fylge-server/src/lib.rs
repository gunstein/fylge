//! Fylge Server - Axum server with htmx and globe.gl.

pub mod config;
pub mod middleware;
pub mod routes;
pub mod state;

pub use config::Config;
pub use state::AppState;
