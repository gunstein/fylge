use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::models::{LogEntry, Marker};
use uuid::Uuid;

/// Initialize database connection pool.
pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

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

/// Insert a new marker (operation = insert).
pub async fn insert_marker(
    pool: &SqlitePool,
    globe_id: &str,
    uuid: Uuid,
    lat: f64,
    lon: f64,
    icon_id: &str,
    label: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let uuid_str = uuid.to_string();
    let result = sqlx::query(
        r#"
        INSERT INTO marker_log (globe_id, uuid, operation, lat, lon, icon_id, label)
        VALUES (?, ?, 'insert', ?, ?, ?, ?)
        "#,
    )
    .bind(globe_id)
    .bind(&uuid_str)
    .bind(lat)
    .bind(lon)
    .bind(icon_id)
    .bind(label)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Update an existing marker (operation = update).
pub async fn update_marker(
    pool: &SqlitePool,
    globe_id: &str,
    uuid: Uuid,
    lat: Option<f64>,
    lon: Option<f64>,
    icon_id: Option<&str>,
    label: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let uuid_str = uuid.to_string();
    let result = sqlx::query(
        r#"
        INSERT INTO marker_log (globe_id, uuid, operation, lat, lon, icon_id, label)
        VALUES (?, ?, 'update', ?, ?, ?, ?)
        "#,
    )
    .bind(globe_id)
    .bind(&uuid_str)
    .bind(lat)
    .bind(lon)
    .bind(icon_id)
    .bind(label)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Delete a marker (operation = delete).
pub async fn delete_marker(
    pool: &SqlitePool,
    globe_id: &str,
    uuid: Uuid,
) -> Result<i64, sqlx::Error> {
    let uuid_str = uuid.to_string();
    let result = sqlx::query(
        r#"
        INSERT INTO marker_log (globe_id, uuid, operation)
        VALUES (?, ?, 'delete')
        "#,
    )
    .bind(globe_id)
    .bind(&uuid_str)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Get log entries after a given id.
pub async fn get_log(
    pool: &SqlitePool,
    globe_id: &str,
    after_id: i64,
    limit: i64,
) -> Result<Vec<LogEntry>, sqlx::Error> {
    let entries = sqlx::query_as::<_, LogEntry>(
        r#"
        SELECT id, globe_id, uuid, operation, ts, lat, lon, icon_id, label
        FROM marker_log
        WHERE globe_id = ? AND id > ?
        ORDER BY id
        LIMIT ?
        "#,
    )
    .bind(globe_id)
    .bind(after_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(entries)
}

/// Get current state of all markers (latest non-deleted state per uuid).
/// Materializes state by applying all log entries in order.
pub async fn get_markers(pool: &SqlitePool, globe_id: &str) -> Result<Vec<Marker>, sqlx::Error> {
    // To handle partial updates, we need to materialize state by applying all events.
    // This query uses window functions to carry forward non-null values.
    let markers = sqlx::query_as::<_, Marker>(
        r#"
        WITH ordered_log AS (
            SELECT
                uuid,
                operation,
                lat,
                lon,
                icon_id,
                label,
                ts,
                ROW_NUMBER() OVER (PARTITION BY uuid ORDER BY id DESC) as rn
            FROM marker_log
            WHERE globe_id = ?
        ),
        latest AS (
            SELECT uuid, operation, ts
            FROM ordered_log
            WHERE rn = 1
        ),
        materialized AS (
            SELECT
                l.uuid,
                (SELECT ol.lat FROM ordered_log ol WHERE ol.uuid = l.uuid AND ol.lat IS NOT NULL ORDER BY ol.rn LIMIT 1) as lat,
                (SELECT ol.lon FROM ordered_log ol WHERE ol.uuid = l.uuid AND ol.lon IS NOT NULL ORDER BY ol.rn LIMIT 1) as lon,
                (SELECT ol.icon_id FROM ordered_log ol WHERE ol.uuid = l.uuid AND ol.icon_id IS NOT NULL ORDER BY ol.rn LIMIT 1) as icon_id,
                (SELECT ol.label FROM ordered_log ol WHERE ol.uuid = l.uuid AND ol.label IS NOT NULL ORDER BY ol.rn LIMIT 1) as label,
                l.ts as updated_at
            FROM latest l
            WHERE l.operation != 'delete'
        )
        SELECT uuid, lat, lon, icon_id, label, updated_at
        FROM materialized
        WHERE lat IS NOT NULL AND lon IS NOT NULL AND icon_id IS NOT NULL
        "#,
    )
    .bind(globe_id)
    .fetch_all(pool)
    .await?;

    Ok(markers)
}

/// Check if a marker exists (has non-deleted state).
pub async fn marker_exists(
    pool: &SqlitePool,
    globe_id: &str,
    uuid: Uuid,
) -> Result<bool, sqlx::Error> {
    let uuid_str = uuid.to_string();
    let row: (i32,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM (
            SELECT m.operation
            FROM marker_log m
            INNER JOIN (
                SELECT MAX(id) as max_id
                FROM marker_log
                WHERE globe_id = ? AND uuid = ?
            ) latest ON m.id = latest.max_id
            WHERE m.operation != 'delete'
        )
        "#,
    )
    .bind(globe_id)
    .bind(&uuid_str)
    .fetch_one(pool)
    .await?;

    Ok(row.0 > 0)
}
