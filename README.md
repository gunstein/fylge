# Fylge

A distributed globe marker application with eventual consistency. Place markers on a 3D globe and sync them across multiple nodes.

## Architecture

- **Frontend**: Server-driven with htmx + globe.gl for 3D visualization
- **Backend**: Rust + Axum + redb
- **Replication**: CouchDB-inspired append-only event log with Hybrid Logical Clocks

### Crate Structure

```
fylge/
├── crates/
│   ├── fylge-core/          # Domain models, traits, validation
│   ├── fylge-db/            # redb storage implementation
│   ├── fylge-replication/   # Pull-based sync, conflict resolution
│   └── fylge-server/        # Axum HTTP server
```

## Quick Start

```bash
# Run the server
FYLGE_NODE_ID=1 cargo run -p fylge-server

# Open in browser
open http://localhost:3000
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `FYLGE_NODE_ID` | (required) | Unique node ID (u64) |
| `FYLGE_LISTEN_ADDR` | `0.0.0.0:3000` | Listen address |
| `FYLGE_DB_PATH` | `./fylge.redb` | Database file path |
| `FYLGE_PEERS` | (empty) | Comma-separated peer URLs |
| `FYLGE_PULL_INTERVAL_SECS` | `5` | Sync interval |

## Multi-Node Setup

```bash
# Terminal 1 - Node 1
FYLGE_NODE_ID=1 FYLGE_LISTEN_ADDR=0.0.0.0:3001 FYLGE_DB_PATH=./node1.redb cargo run -p fylge-server

# Terminal 2 - Node 2
FYLGE_NODE_ID=2 FYLGE_LISTEN_ADDR=0.0.0.0:3002 FYLGE_DB_PATH=./node2.redb FYLGE_PEERS=http://localhost:3001 cargo run -p fylge-server
```

## Usage

1. Select an icon from the palette on the left
2. Click anywhere on the globe to place a marker
3. Markers sync automatically between nodes

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Main page with globe |
| GET | `/api/markers` | Get all markers (JSON) |
| GET | `/api/icons` | Get available icons (JSON) |
| POST | `/markers` | Create marker |
| DELETE | `/markers/{id}` | Delete marker |
| GET | `/health` | Health check |

## Testing

```bash
cargo test
```

## Design Principles

- **Eventual consistency**: Split-brain is allowed; nodes converge when reconnected
- **Append-only**: All writes are events; no updates or deletes at the storage level
- **Last-write-wins**: Conflicts resolved deterministically using Hybrid Logical Clocks
- **Simple**: Minimal dependencies, easy to understand and modify
