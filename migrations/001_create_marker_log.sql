-- Marker log table (append-only, no updates/deletes)
CREATE TABLE IF NOT EXISTS marker_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL,
    ts TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    lat REAL NOT NULL,
    lon REAL NOT NULL,
    icon_id TEXT NOT NULL,
    label TEXT
);

-- Idempotency: same UUID cannot be inserted twice
CREATE UNIQUE INDEX IF NOT EXISTS ux_marker_log_uuid ON marker_log(uuid);

-- Efficient fetching of last 24 hours
CREATE INDEX IF NOT EXISTS ix_marker_log_ts ON marker_log(ts);

-- Efficient paging/sync with after_id
CREATE INDEX IF NOT EXISTS ix_marker_log_id ON marker_log(id);
