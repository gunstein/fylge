use axum::{Router, extract::State, response::Html, routing::get};

use fylge_core::{EntityStore, Icon};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(index))
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let icons = &*state.icons;
    let entities = state.entity_store.get_all().unwrap_or_default();

    Html(render_index(icons, entities.len()))
}

fn render_index(icons: &[Icon], marker_count: usize) -> String {
    let icon_buttons: String = icons
        .iter()
        .map(|icon| {
            format!(
                r#"<button type="button" class="icon-btn" data-icon-id="{}" title="{}">
                    <img src="/static/icons/{}" alt="{}" width="32" height="32">
                </button>"#,
                icon.id, icon.name, icon.filename, icon.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Fylge</title>
    <link rel="stylesheet" href="/static/style.css">
    <script src="/static/htmx.min.js"></script>
    <script src="https://unpkg.com/globe.gl"></script>
</head>
<body>
    <div id="app">
        <aside id="sidebar">
            <h1>Fylge</h1>

            <section id="icon-palette">
                <h2>Select Icon</h2>
                <div class="icon-grid">
                    {icon_buttons}
                </div>
            </section>

            <section id="instructions">
                <p>1. Select an icon above</p>
                <p>2. Click on the globe to place a marker</p>
            </section>

            <section id="stats">
                <p>Markers: <span id="marker-count">{marker_count}</span></p>
            </section>

            <section id="messages"></section>
        </aside>

        <main id="globe-container"></main>
    </div>

    <script src="/static/app.js"></script>
</body>
</html>"##
    )
}
