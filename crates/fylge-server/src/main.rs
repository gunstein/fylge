use std::sync::Arc;
use std::time::Duration;

use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use fylge_core::Icon;
use fylge_db::{init_database, RedbCheckpointStore, RedbEntityStore, RedbEventStore};
use fylge_replication::{HttpPeerClientWithEndpoints, PullReplicator};
use fylge_server::{routes, AppState, Config};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            eprintln!("Required: FYLGE_NODE_ID=<number>");
            eprintln!("Optional: FYLGE_LISTEN_ADDR, FYLGE_DB_PATH, FYLGE_PEERS");
            std::process::exit(1);
        }
    };

    tracing::info!("Starting Fylge server");
    tracing::info!("Node ID: {}", config.node_id);
    tracing::info!("Listen address: {}", config.listen_addr);
    tracing::info!("Database path: {}", config.db_path.display());

    // Initialize database
    let db = match init_database(&config.db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Database error: {}", e);
            std::process::exit(1);
        }
    };

    let event_store = Arc::new(RedbEventStore::new(db.clone()));
    let entity_store = Arc::new(RedbEntityStore::new(db.clone()));
    let checkpoint_store = Arc::new(RedbCheckpointStore::new(db));

    // Load icons (hardcoded for now, could be loaded from icons.json later)
    let icons = vec![
        Icon::new("marker", "Marker", "marker.svg"),
        Icon::new("ship", "Ship", "ship.svg"),
        Icon::new("plane", "Plane", "plane.svg"),
    ];

    // Create app state
    let state = AppState::new(
        config.node_id,
        event_store.clone(),
        entity_store.clone(),
        icons,
    );

    // Start replication if we have peers
    if !config.peers.is_empty() {
        tracing::info!("Starting replication with {} peers", config.peers.len());
        let peer_client = Arc::new(HttpPeerClientWithEndpoints::new(&config.peers));
        let replicator = PullReplicator::new(
            config.node_id,
            event_store,
            entity_store,
            checkpoint_store,
            peer_client,
            config.peers.clone(),
        );
        let interval = Duration::from_secs(config.pull_interval_secs);
        tokio::spawn(async move {
            replicator.run(interval).await;
        });
    }

    // Build router
    let app = routes::create_router(state)
        .nest_service("/static", ServeDir::new("crates/fylge-server/static"))
        .into_make_service_with_connect_info::<std::net::SocketAddr>();

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server running at http://{}", config.listen_addr);

    axum::serve(listener, app).await.expect("Server error");
}
