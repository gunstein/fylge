use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::models::Marker;

/// Initialize database connection pool with recommended pragmas.
pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5))
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

    SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await
}

/// Run database migrations.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(include_str!("../migrations/001_create_marker_log.sql"))
        .execute(pool)
        .await?;
    Ok(())
}

/// Insert a new marker. Returns the marker if created, or existing marker if uuid already exists.
/// Returns (marker, created) where created is true if this was a new insert.
pub async fn insert_marker(
    pool: &SqlitePool,
    uuid: &str,
    lat: f64,
    lon: f64,
    icon_id: &str,
    label: Option<&str>,
) -> Result<(Marker, bool), sqlx::Error> {
    // Try to insert
    let result = sqlx::query(
        r#"
        INSERT INTO marker_log (uuid, lat, lon, icon_id, label)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(uuid) DO NOTHING
        "#,
    )
    .bind(uuid)
    .bind(lat)
    .bind(lon)
    .bind(icon_id)
    .bind(label)
    .execute(pool)
    .await?;

    let created = result.rows_affected() > 0;

    // Fetch the marker (either just created or existing)
    let marker = sqlx::query_as::<_, Marker>(
        "SELECT id, uuid, ts, lat, lon, icon_id, label FROM marker_log WHERE uuid = ?",
    )
    .bind(uuid)
    .fetch_one(pool)
    .await?;

    Ok((marker, created))
}

/// Get markers from the last 24 hours.
pub async fn get_markers_last_24h(pool: &SqlitePool) -> Result<(Vec<Marker>, i64), sqlx::Error> {
    let markers = sqlx::query_as::<_, Marker>(
        r#"
        SELECT id, uuid, ts, lat, lon, icon_id, label
        FROM marker_log
        WHERE ts >= datetime('now', '-24 hours')
        ORDER BY ts ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    // Get max_id
    let max_id: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) FROM marker_log")
        .fetch_one(pool)
        .await?;

    Ok((markers, max_id))
}

/// Get markers visible at a specific point in time (24h window ending at that time).
pub async fn get_markers_at(pool: &SqlitePool, at: &str) -> Result<Vec<Marker>, sqlx::Error> {
    let markers = sqlx::query_as::<_, Marker>(
        r#"
        SELECT id, uuid, ts, lat, lon, icon_id, label
        FROM marker_log
        WHERE ts <= ?
          AND ts >= datetime(?, '-24 hours')
        ORDER BY ts ASC
        "#,
    )
    .bind(at)
    .bind(at)
    .fetch_all(pool)
    .await?;

    Ok(markers)
}

/// Get log entries after a given id (for polling/sync).
pub async fn get_log_after(
    pool: &SqlitePool,
    after_id: i64,
    limit: i64,
) -> Result<(Vec<Marker>, i64, bool), sqlx::Error> {
    let entries = sqlx::query_as::<_, Marker>(
        r#"
        SELECT id, uuid, ts, lat, lon, icon_id, label
        FROM marker_log
        WHERE id > ?
        ORDER BY id ASC
        LIMIT ?
        "#,
    )
    .bind(after_id)
    .bind(limit + 1) // Fetch one extra to check if there's more
    .fetch_all(pool)
    .await?;

    let has_more = entries.len() > limit as usize;
    let entries: Vec<Marker> = entries.into_iter().take(limit as usize).collect();

    let max_id = entries.last().map(|m| m.id).unwrap_or(after_id);

    Ok((entries, max_id, has_more))
}

/// Get current server time in ISO format.
pub async fn get_server_time(pool: &SqlitePool) -> Result<String, sqlx::Error> {
    let time: String = sqlx::query_scalar("SELECT strftime('%Y-%m-%dT%H:%M:%fZ', 'now')")
        .fetch_one(pool)
        .await?;
    Ok(time)
}
