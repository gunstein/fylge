use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::models::{LogEntry, Marker};
use uuid::Uuid;

/// Initialize database connection pool.
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

/// Run database migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(include_str!("../migrations/001_create_marker_log.sql"))
        .execute(pool)
        .await?;
    Ok(())
}

/// Insert a new marker (operation = insert).
pub async fn insert_marker(
    pool: &PgPool,
    globe_id: &str,
    uuid: Uuid,
    lat: f64,
    lon: f64,
    icon_id: &str,
    label: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO marker_log (globe_id, uuid, operation, lat, lon, icon_id, label)
        VALUES ($1, $2, 'insert', $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(globe_id)
    .bind(uuid)
    .bind(lat)
    .bind(lon)
    .bind(icon_id)
    .bind(label)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Update an existing marker (operation = update).
pub async fn update_marker(
    pool: &PgPool,
    globe_id: &str,
    uuid: Uuid,
    lat: Option<f64>,
    lon: Option<f64>,
    icon_id: Option<&str>,
    label: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO marker_log (globe_id, uuid, operation, lat, lon, icon_id, label)
        VALUES ($1, $2, 'update', $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(globe_id)
    .bind(uuid)
    .bind(lat)
    .bind(lon)
    .bind(icon_id)
    .bind(label)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Delete a marker (operation = delete).
pub async fn delete_marker(pool: &PgPool, globe_id: &str, uuid: Uuid) -> Result<i64, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO marker_log (globe_id, uuid, operation)
        VALUES ($1, $2, 'delete')
        RETURNING id
        "#,
    )
    .bind(globe_id)
    .bind(uuid)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Get log entries after a given id.
pub async fn get_log(
    pool: &PgPool,
    globe_id: &str,
    after_id: i64,
    limit: i64,
) -> Result<Vec<LogEntry>, sqlx::Error> {
    let entries = sqlx::query_as::<_, LogEntry>(
        r#"
        SELECT id, globe_id, uuid, operation, ts, lat, lon, icon_id, label
        FROM marker_log
        WHERE globe_id = $1 AND id > $2
        ORDER BY id
        LIMIT $3
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
pub async fn get_markers(pool: &PgPool, globe_id: &str) -> Result<Vec<Marker>, sqlx::Error> {
    // Use DISTINCT ON to get latest entry per uuid, then filter out deletes
    let markers = sqlx::query_as::<_, Marker>(
        r#"
        SELECT uuid, lat, lon, icon_id, label, ts as updated_at
        FROM (
            SELECT DISTINCT ON (uuid) uuid, operation, lat, lon, icon_id, label, ts
            FROM marker_log
            WHERE globe_id = $1
            ORDER BY uuid, id DESC
        ) latest
        WHERE operation != 'delete'
          AND lat IS NOT NULL
          AND lon IS NOT NULL
          AND icon_id IS NOT NULL
        "#,
    )
    .bind(globe_id)
    .fetch_all(pool)
    .await?;

    Ok(markers)
}

/// Check if a marker exists (has non-deleted state).
pub async fn marker_exists(pool: &PgPool, globe_id: &str, uuid: Uuid) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM (
                SELECT DISTINCT ON (uuid) operation
                FROM marker_log
                WHERE globe_id = $1 AND uuid = $2
                ORDER BY uuid, id DESC
            ) latest
            WHERE operation != 'delete'
        )
        "#,
    )
    .bind(globe_id)
    .bind(uuid)
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

/// Get the latest state of a single marker.
pub async fn get_marker(
    pool: &PgPool,
    globe_id: &str,
    uuid: Uuid,
) -> Result<Option<Marker>, sqlx::Error> {
    let marker = sqlx::query_as::<_, Marker>(
        r#"
        SELECT uuid, lat, lon, icon_id, label, ts as updated_at
        FROM (
            SELECT DISTINCT ON (uuid) uuid, operation, lat, lon, icon_id, label, ts
            FROM marker_log
            WHERE globe_id = $1 AND uuid = $2
            ORDER BY uuid, id DESC
        ) latest
        WHERE operation != 'delete'
          AND lat IS NOT NULL
          AND lon IS NOT NULL
          AND icon_id IS NOT NULL
        "#,
    )
    .bind(globe_id)
    .bind(uuid)
    .fetch_optional(pool)
    .await?;

    Ok(marker)
}
