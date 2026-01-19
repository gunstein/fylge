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

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test database with in-memory SQLite.
    async fn setup_test_db() -> SqlitePool {
        let pool = init_pool("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_insert_marker() {
        let pool = setup_test_db().await;

        let (marker, created) = insert_marker(
            &pool,
            "550e8400-e29b-41d4-a716-446655440000",
            59.91,
            10.75,
            "marker",
            Some("Oslo"),
        )
        .await
        .unwrap();

        assert!(created);
        assert_eq!(marker.uuid, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(marker.lat, 59.91);
        assert_eq!(marker.lon, 10.75);
        assert_eq!(marker.icon_id, "marker");
        assert_eq!(marker.label, Some("Oslo".to_string()));
        assert_eq!(marker.id, 1);
    }

    #[tokio::test]
    async fn test_insert_marker_without_label() {
        let pool = setup_test_db().await;

        let (marker, created) = insert_marker(
            &pool,
            "550e8400-e29b-41d4-a716-446655440000",
            59.91,
            10.75,
            "marker",
            None,
        )
        .await
        .unwrap();

        assert!(created);
        assert_eq!(marker.label, None);
    }

    #[tokio::test]
    async fn test_insert_marker_idempotent() {
        let pool = setup_test_db().await;

        // First insert
        let (marker1, created1) = insert_marker(
            &pool,
            "550e8400-e29b-41d4-a716-446655440000",
            59.91,
            10.75,
            "marker",
            Some("Oslo"),
        )
        .await
        .unwrap();

        assert!(created1);

        // Second insert with same UUID
        let (marker2, created2) = insert_marker(
            &pool,
            "550e8400-e29b-41d4-a716-446655440000",
            60.0, // Different lat
            11.0, // Different lon
            "ship",
            Some("Bergen"),
        )
        .await
        .unwrap();

        assert!(!created2); // Should not be created
        assert_eq!(marker2.id, marker1.id); // Same marker
        assert_eq!(marker2.lat, 59.91); // Original values preserved
        assert_eq!(marker2.icon_id, "marker");
    }

    #[tokio::test]
    async fn test_get_markers_last_24h_empty() {
        let pool = setup_test_db().await;

        let (markers, max_id) = get_markers_last_24h(&pool).await.unwrap();

        assert!(markers.is_empty());
        assert_eq!(max_id, 0);
    }

    #[tokio::test]
    async fn test_get_markers_last_24h() {
        let pool = setup_test_db().await;

        // Insert two markers
        insert_marker(&pool, "uuid-1", 59.91, 10.75, "marker", Some("Oslo"))
            .await
            .unwrap();
        insert_marker(&pool, "uuid-2", 60.39, 5.32, "ship", Some("Bergen"))
            .await
            .unwrap();

        let (markers, max_id) = get_markers_last_24h(&pool).await.unwrap();

        assert_eq!(markers.len(), 2);
        assert_eq!(max_id, 2);
        assert_eq!(markers[0].uuid, "uuid-1");
        assert_eq!(markers[1].uuid, "uuid-2");
    }

    #[tokio::test]
    async fn test_get_log_after_empty() {
        let pool = setup_test_db().await;

        let (entries, max_id, has_more) = get_log_after(&pool, 0, 100).await.unwrap();

        assert!(entries.is_empty());
        assert_eq!(max_id, 0);
        assert!(!has_more);
    }

    #[tokio::test]
    async fn test_get_log_after() {
        let pool = setup_test_db().await;

        // Insert three markers
        insert_marker(&pool, "uuid-1", 59.91, 10.75, "marker", None)
            .await
            .unwrap();
        insert_marker(&pool, "uuid-2", 60.39, 5.32, "ship", None)
            .await
            .unwrap();
        insert_marker(&pool, "uuid-3", 63.43, 10.39, "plane", None)
            .await
            .unwrap();

        // Get all entries after id 0
        let (entries, max_id, has_more) = get_log_after(&pool, 0, 100).await.unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(max_id, 3);
        assert!(!has_more);

        // Get entries after id 1
        let (entries, max_id, has_more) = get_log_after(&pool, 1, 100).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].uuid, "uuid-2");
        assert_eq!(entries[1].uuid, "uuid-3");
        assert_eq!(max_id, 3);
        assert!(!has_more);

        // Get entries after id 3 (none)
        let (entries, max_id, has_more) = get_log_after(&pool, 3, 100).await.unwrap();
        assert!(entries.is_empty());
        assert_eq!(max_id, 3);
        assert!(!has_more);
    }

    #[tokio::test]
    async fn test_get_log_after_with_limit() {
        let pool = setup_test_db().await;

        // Insert five markers
        for i in 1..=5 {
            insert_marker(
                &pool,
                &format!("uuid-{}", i),
                59.0 + i as f64,
                10.0,
                "marker",
                None,
            )
            .await
            .unwrap();
        }

        // Get with limit 2
        let (entries, max_id, has_more) = get_log_after(&pool, 0, 2).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(max_id, 2);
        assert!(has_more);

        // Get next page
        let (entries, max_id, has_more) = get_log_after(&pool, 2, 2).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(max_id, 4);
        assert!(has_more);

        // Get last page
        let (entries, max_id, has_more) = get_log_after(&pool, 4, 2).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(max_id, 5);
        assert!(!has_more);
    }

    #[tokio::test]
    async fn test_get_server_time() {
        let pool = setup_test_db().await;

        let time = get_server_time(&pool).await.unwrap();

        // Should be in ISO format
        assert!(time.contains("T"));
        assert!(time.ends_with("Z"));
        // Should be parseable
        assert!(time.len() > 20);
    }

    #[tokio::test]
    async fn test_get_markers_at() {
        let pool = setup_test_db().await;

        // Insert a marker
        insert_marker(&pool, "uuid-1", 59.91, 10.75, "marker", Some("Oslo"))
            .await
            .unwrap();

        // Get current server time
        let now = get_server_time(&pool).await.unwrap();

        // Get markers at current time (should include the marker)
        let markers = get_markers_at(&pool, &now).await.unwrap();
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].uuid, "uuid-1");
    }
}
