pub mod api;
pub mod markers;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};

use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Index page
        .route("/", get(index))
        // Marker creation (append-only, no update/delete)
        .route("/markers", post(markers::create_marker))
        // API endpoints
        .route("/api/markers", get(api::get_markers))
        .route("/api/markers_at", get(api::get_markers_at))
        .route("/api/log", get(api::get_log))
        .route("/api/icons", get(api::get_icons))
        // Health check
        .route("/health", get(health))
        .with_state(state)
}

async fn index() -> Response {
    // Try to serve the built frontend
    match std::fs::read_to_string("static/dist/index.html") {
        Ok(content) => Html(content).into_response(),
        Err(_) => {
            // Fallback: instructions for building frontend
            (StatusCode::OK, Html(FALLBACK_HTML)).into_response()
        }
    }
}

async fn health() -> &'static str {
    "OK"
}

const FALLBACK_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Fylge - Setup Required</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            background: #1a1a2e;
            color: #eee;
        }
        h1 { color: #4fc3f7; }
        code {
            background: #0f0f23;
            padding: 2px 6px;
            border-radius: 4px;
        }
        pre {
            background: #0f0f23;
            padding: 16px;
            border-radius: 8px;
            overflow-x: auto;
        }
        a { color: #4fc3f7; }
    </style>
</head>
<body>
    <h1>Fylge</h1>
    <p>The frontend has not been built yet. To build it:</p>
    <pre>cd frontend
npm install
npm run build</pre>
    <p>Then refresh this page.</p>
    <hr>
    <p>Alternatively, for development with hot reload:</p>
    <pre># Terminal 1: backend
cargo run

# Terminal 2: frontend dev server
cd frontend
npm run dev</pre>
    <p>Then open <a href="http://localhost:5173">http://localhost:5173</a></p>
    <hr>
    <p>API endpoints are available:</p>
    <ul>
        <li><a href="/api/markers">/api/markers</a> - Get markers (last 24h)</li>
        <li><a href="/api/icons">/api/icons</a> - Get available icons</li>
        <li><a href="/api/log">/api/log</a> - Get log entries</li>
        <li><a href="/health">/health</a> - Health check</li>
    </ul>
</body>
</html>"#;
