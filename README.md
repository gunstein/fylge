# Fylge

A globe marker application with a simple append-only log architecture. Place markers on a 3D globe - they remain visible for 24 hours.

## Architecture

- **SQLite database** with WAL mode for concurrent access
- **Append-only `marker_log` table** - only inserts, no updates or deletes
- **24-hour TTL** - markers automatically expire after 24 hours
- **Idempotent creates** - frontend generates UUID, duplicate inserts are no-ops

## Requirements

- Rust 1.70+
- Node.js 18+ (for frontend development only)

## Quick Start

```bash
# Run the server (creates fylge.db automatically)
cargo run

# Open in browser
open http://localhost:3000
```

## Configuration

Environment variables (all optional):

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite://fylge.db` | SQLite database path |
| `LISTEN_ADDR` | `0.0.0.0:3000` | Server listen address |

## API

### Create Marker

```bash
POST /markers
Content-Type: application/json

{
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "lat": 59.91,
  "lon": 10.75,
  "icon_id": "marker",
  "label": "Oslo"
}
```

Response:
```json
{
  "status": "created",
  "marker": {
    "id": 1,
    "uuid": "550e8400-e29b-41d4-a716-446655440000",
    "ts": "2024-01-19T12:00:00.000Z",
    "lat": 59.91,
    "lon": 10.75,
    "icon_id": "marker",
    "label": "Oslo"
  }
}
```

If the same UUID is sent again, returns `"status": "exists"` with the existing marker.

### Get Markers (Last 24 Hours)

```bash
GET /api/markers
```

Response:
```json
{
  "window_hours": 24,
  "server_time": "2024-01-19T12:00:00.000Z",
  "max_id": 42,
  "markers": [...]
}
```

### Get Markers at Specific Time

```bash
GET /api/markers_at?at=2024-01-19T10:00:00.000Z
```

Returns markers visible at that point in time (24h window ending at `at`).

### Get Log (for Polling)

```bash
GET /api/log?after_id=0&limit=100
```

Response:
```json
{
  "after_id": 0,
  "limit": 100,
  "server_time": "2024-01-19T12:00:00.000Z",
  "max_id": 42,
  "has_more": false,
  "entries": [...]
}
```

### Get Icons

```bash
GET /api/icons
```

## Data Model

```sql
CREATE TABLE marker_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE,
    ts TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    lat REAL NOT NULL,
    lon REAL NOT NULL,
    icon_id TEXT NOT NULL,
    label TEXT
);
```

## Frontend Development

The backend serves a standalone HTML file at `/` that works without a build step.

For development with hot reload:

```bash
cd frontend
npm install
npm run dev
```

This starts Vite on port 5173 with proxy to the backend.

To build for production:

```bash
cd frontend
npm run build
```

Output goes to `static/dist/`.

## Icons

Icons are configured in `static/icons/icons.json`:

```json
[
  { "id": "marker", "name": "Marker", "url": "/static/icons/marker.svg" },
  { "id": "ship", "name": "Ship", "url": "/static/icons/ship.svg" }
]
```

Add new icons by placing SVG files in `static/icons/` and updating the JSON file.
