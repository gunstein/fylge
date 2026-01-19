-- Marker log table (append-only, no updates/deletes)
CREATE TABLE IF NOT EXISTS marker_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL,
    ts_epoch_ms INTEGER NOT NULL,  -- milliseconds since Unix epoch
    lat REAL NOT NULL CHECK(lat BETWEEN -90 AND 90),
    lon REAL NOT NULL CHECK(lon BETWEEN -180 AND 180),
    icon_id TEXT NOT NULL CHECK(length(icon_id) BETWEEN 1 AND 64),
    label TEXT CHECK(label IS NULL OR length(label) <= 256)
);

-- Idempotency: same UUID cannot be inserted twice
CREATE UNIQUE INDEX IF NOT EXISTS ux_marker_log_uuid ON marker_log(uuid);

-- Efficient fetching of last 24 hours (numeric comparison)
CREATE INDEX IF NOT EXISTS ix_marker_log_ts ON marker_log(ts_epoch_ms);

-- Efficient paging/sync with after_id
CREATE INDEX IF NOT EXISTS ix_marker_log_id ON marker_log(id);
