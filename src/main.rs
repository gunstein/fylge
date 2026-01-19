use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use fylge::{create_router, init_pool, load_icons, run_migrations, AppState, Config};

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
            eprintln!("Optional: DATABASE_URL (default: sqlite://fylge.db)");
            eprintln!("Optional: LISTEN_ADDR (default: 0.0.0.0:3000)");
            std::process::exit(1);
        }
    };

    tracing::info!("Starting Fylge server");
    tracing::info!("Listen address: {}", config.listen_addr);
    tracing::info!("Database: {}", config.database_url);

    // Connect to database
    let pool = match init_pool(&config.database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Database connection error: {}", e);
            std::process::exit(1);
        }
    };

    // Run migrations
    if let Err(e) = run_migrations(&pool).await {
        eprintln!("Migration error: {}", e);
        std::process::exit(1);
    }
    tracing::info!("Database migrations completed");

    // Load icons
    let icons = load_icons();
    tracing::info!("Loaded {} icons", icons.len());

    // Create app state
    let state = AppState::new(pool, icons);

    // Build router
    let app = create_router(state).nest_service("/static", ServeDir::new("static"));

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.listen_addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server running at http://{}", config.listen_addr);

    axum::serve(listener, app).await.expect("Server error");
}
