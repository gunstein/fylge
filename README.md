# Fylge

A distributed globe marker application with a simple append-only log architecture.

## Architecture

- **Single Postgres database** as the source of truth
- **Append-only `marker_log` table** - inserts, updates, and deletes are all log entries
- **Stateless app servers** - can run multiple instances behind a load balancer
- **HA via Postgres replication** - no custom sync logic needed

## Requirements

- Rust 1.70+
- PostgreSQL 14+

## Setup

1. Create a PostgreSQL database:
```bash
createdb fylge
```

2. Set environment variables:
```bash
export DATABASE_URL=postgres://user:pass@localhost/fylge
export LISTEN_ADDR=0.0.0.0:3000  # optional, default shown
```

3. Run the server:
```bash
cargo run
```

The server will automatically run migrations on startup.

## API

### Write Operations

**Create marker:**
```bash
POST /markers
Content-Type: application/json

{
  "lat": 59.9,
  "lon": 10.7,
  "icon_id": "ship",
  "label": "Oslo"
}
```

**Update marker:**
```bash
PUT /markers/{uuid}
Content-Type: application/json

{
  "lat": 60.0,
  "lon": 11.0
}
```

**Delete marker:**
```bash
DELETE /markers/{uuid}
```

### Read Operations

**Get current markers (computed state):**
```bash
GET /api/markers?globe_id=default
```

**Get log entries (for sync/replay):**
```bash
GET /api/log?globe_id=default&after_id=0&limit=100
```

**Get available icons:**
```bash
GET /api/icons
```

## Data Model

All changes are stored in a single append-only log table:

```sql
CREATE TABLE marker_log (
    id BIGSERIAL PRIMARY KEY,       -- monotonic cursor for sync
    globe_id TEXT NOT NULL,         -- partition key
    uuid UUID NOT NULL,             -- marker identity
    operation TEXT NOT NULL,        -- 'insert', 'update', 'delete'
    ts TIMESTAMPTZ DEFAULT now(),   -- timestamp
    lat DOUBLE PRECISION,           -- coordinates (null for delete)
    lon DOUBLE PRECISION,
    icon_id TEXT,
    label TEXT
);
```

**Key properties:**
- Nothing is ever deleted from the log
- Current state = latest non-deleted entry per UUID
- Sync = `SELECT * FROM marker_log WHERE id > last_seen_id`

## Frontend

Open http://localhost:3000 to see the globe interface.

- Select an icon from the palette
- Click on the globe to place a marker
- Click "Delete" to remove markers
