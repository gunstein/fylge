# Fylge

A globe marker application with a simple append-only log architecture. Place markers on a 3D globe - they remain visible for 24 hours.

## Architecture

- **SQLite database** with WAL mode for concurrent access
- **Append-only `marker_log` table** - only inserts, no updates or deletes
- **24-hour TTL** - markers automatically expire after 24 hours
- **Idempotent creates** - frontend generates UUID, duplicate inserts are no-ops
- **Database constraints** - CHECK constraints enforce data validity at DB level

## Requirements

- Rust 1.78+
- Node.js 18+ (for frontend development only)

## Quick Start

```bash
# Run the server (creates fylge.db automatically)
cargo run

# Open in browser
open http://localhost:3000
```

If the frontend hasn't been built, you'll see instructions for building it.

## Configuration

Environment variables (all optional):

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite://fylge.db` | SQLite database path |
| `LISTEN_ADDR` | `0.0.0.0:3000` | Server listen address |

## API

All timestamps are in **epoch milliseconds** (ms since Unix epoch).

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
    "ts_epoch_ms": 1705665600000,
    "lat": 59.91,
    "lon": 10.75,
    "icon_id": "marker",
    "label": "Oslo"
  }
}
```

If the same UUID is sent again, returns `"status": "exists"` with the existing marker.

**Validation:**
- `uuid` must be valid UUID format
- `lat` must be between -90 and 90
- `lon` must be between -180 and 180
- `icon_id` must be non-empty, max 64 chars, and must exist in available icons
- `label` is optional, max 256 chars
- Unknown fields are rejected

**Error response:**
```json
{
  "error": "Invalid latitude: 91 (must be between -90 and 90)",
  "field": "lat"
}
```

### Get Markers (Last 24 Hours)

```bash
GET /api/markers
```

Response:
```json
{
  "window_hours": 24,
  "server_time_ms": 1705665600000,
  "max_id": 42,
  "markers": [...]
}
```

### Get Markers at Specific Time

```bash
GET /api/markers_at?at=1705665600000
```

The `at` parameter is epoch milliseconds. Returns markers visible at that point in time (24h window ending at `at`).

### Get Log (for Polling)

```bash
GET /api/log?after_id=0&limit=100
```

Response:
```json
{
  "after_id": 0,
  "limit": 100,
  "server_time_ms": 1705665600000,
  "max_id": 42,
  "has_more": false,
  "entries": [...]
}
```

**Validation:**
- `limit` must be between 1 and 1000

### Get Icons

```bash
GET /api/icons
```

## Data Model

```sql
CREATE TABLE marker_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE,
    ts_epoch_ms INTEGER NOT NULL,
    lat REAL NOT NULL CHECK(lat BETWEEN -90 AND 90),
    lon REAL NOT NULL CHECK(lon BETWEEN -180 AND 180),
    icon_id TEXT NOT NULL CHECK(length(icon_id) BETWEEN 1 AND 64),
    label TEXT CHECK(label IS NULL OR length(label) <= 256)
);
```

## Frontend Development

The backend serves a fallback page with instructions if the frontend hasn't been built.

For development with hot reload:

```bash
# Terminal 1: backend
cargo run

# Terminal 2: frontend dev server
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

**Note:** The `icon_id` in marker creation requests is validated against this list.

## Testing

```bash
cargo test
```

Tests cover:
- Model validation (UUID, coordinates, icon_id, label)
- Database operations (CRUD, idempotency, pagination)
- 24-hour TTL filtering
- Database CHECK constraints
- API endpoints (health, icons, markers, log)
- Error handling
