-- Marker log table (append-only)
CREATE TABLE IF NOT EXISTS marker_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    globe_id TEXT NOT NULL DEFAULT 'default',
    uuid TEXT NOT NULL,
    operation TEXT NOT NULL CHECK (operation IN ('insert', 'update', 'delete')),
    ts TEXT NOT NULL DEFAULT (datetime('now')),
    lat REAL,
    lon REAL,
    icon_id TEXT,
    label TEXT
);

-- Index for efficient paging/sync queries
CREATE INDEX IF NOT EXISTS idx_marker_log_globe_id ON marker_log (globe_id, id);

-- Index for finding latest state per uuid
CREATE INDEX IF NOT EXISTS idx_marker_log_uuid ON marker_log (globe_id, uuid, id DESC);
