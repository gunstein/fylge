use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod models;
mod routes;
mod state;

use config::Config;
use state::AppState;

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
            eprintln!("Required: DATABASE_URL=postgres://...");
            eprintln!("Optional: LISTEN_ADDR (default: 0.0.0.0:3000)");
            std::process::exit(1);
        }
    };

    tracing::info!("Starting Fylge server");
    tracing::info!("Listen address: {}", config.listen_addr);

    // Connect to database
    let pool = match db::init_pool(&config.database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Database connection error: {}", e);
            std::process::exit(1);
        }
    };

    // Run migrations
    if let Err(e) = db::run_migrations(&pool).await {
        eprintln!("Migration error: {}", e);
        std::process::exit(1);
    }
    tracing::info!("Database migrations completed");

    // Create app state
    let state = AppState::new(pool);

    // Build router
    let app = routes::create_router(state)
        .nest_service("/static", ServeDir::new("static"));

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server running at http://{}", config.listen_addr);

    axum::serve(listener, app).await.expect("Server error");
}
