use sqlx::PgPool;
use uuid::Uuid;

use crate::models::file::FileRecord;

/// Store file metadata after upload.
pub async fn create_file(
    pool: &PgPool,
    id: Uuid,
    uploader_id: Uuid,
    encrypted_name: &str,
    size_bytes: i64,
    content_type: Option<&str>,
    storage_path: &str,
    checksum: &str,
    channel_id: Option<Uuid>,
) -> Result<FileRecord, sqlx::Error> {
    sqlx::query_as::<_, FileRecord>(
        r#"
        INSERT INTO files (id, uploader_id, encrypted_name, size_bytes, content_type, storage_path, checksum, channel_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(uploader_id)
    .bind(encrypted_name)
    .bind(size_bytes)
    .bind(content_type)
    .bind(storage_path)
    .bind(checksum)
    .bind(channel_id)
    .fetch_one(pool)
    .await
}

/// Fetch file metadata by ID.
pub async fn get_file(pool: &PgPool, id: Uuid) -> Result<Option<FileRecord>, sqlx::Error> {
    sqlx::query_as::<_, FileRecord>("SELECT * FROM files WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// List files uploaded by a user.
pub async fn list_user_files(
    pool: &PgPool,
    uploader_id: Uuid,
    limit: i64,
) -> Result<Vec<FileRecord>, sqlx::Error> {
    sqlx::query_as::<_, FileRecord>(
        "SELECT * FROM files WHERE uploader_id = $1 ORDER BY created_at DESC LIMIT $2",
    )
    .bind(uploader_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Delete file metadata (the actual file must be removed separately).
pub async fn delete_file(
    pool: &PgPool,
    id: Uuid,
    uploader_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM files WHERE id = $1 AND uploader_id = $2")
        .bind(id)
        .bind(uploader_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Delete file metadata by ID without ownership check (admin).
pub async fn delete_file_admin(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM files WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List all files with pagination and sorting (admin browser).
pub async fn list_all_files(
    pool: &PgPool,
    user_id: Option<Uuid>,
    sort_by: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<FileRecord>, sqlx::Error> {
    let order = if sort_by == "size" {
        "size_bytes DESC"
    } else {
        "created_at DESC"
    };

    if let Some(uid) = user_id {
        let q = format!(
            "SELECT * FROM files WHERE uploader_id = $1 ORDER BY {} LIMIT $2 OFFSET $3",
            order
        );
        sqlx::query_as::<_, FileRecord>(&q)
            .bind(uid)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    } else {
        let q = format!("SELECT * FROM files ORDER BY {} LIMIT $1 OFFSET $2", order);
        sqlx::query_as::<_, FileRecord>(&q)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }
}

/// Count files with optional user filter.
pub async fn count_files(pool: &PgPool, user_id: Option<Uuid>) -> Result<i64, sqlx::Error> {
    let row: (i64,) = if let Some(uid) = user_id {
        sqlx::query_as("SELECT COUNT(*) FROM files WHERE uploader_id = $1")
            .bind(uid)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_as("SELECT COUNT(*) FROM files")
            .fetch_one(pool)
            .await?
    };
    Ok(row.0)
}

/// Per-user storage statistics.
#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct UserStorageStat {
    pub uploader_id: Uuid,
    pub file_count: i64,
    pub total_bytes: i64,
}

/// Get per-user storage breakdown.
pub async fn storage_stats(pool: &PgPool) -> Result<Vec<UserStorageStat>, sqlx::Error> {
    sqlx::query_as::<_, UserStorageStat>(
        r#"
        SELECT uploader_id, COUNT(*) as file_count, COALESCE(SUM(size_bytes), 0) as total_bytes
        FROM files
        GROUP BY uploader_id
        ORDER BY total_bytes DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

/// List all files uploaded by a user (no limit, for purge operations).
pub async fn list_all_user_files(
    pool: &PgPool,
    uploader_id: Uuid,
) -> Result<Vec<FileRecord>, sqlx::Error> {
    sqlx::query_as::<_, FileRecord>(
        "SELECT * FROM files WHERE uploader_id = $1",
    )
    .bind(uploader_id)
    .fetch_all(pool)
    .await
}

/// List all files in a channel (for purge operations).
pub async fn list_channel_files(
    pool: &PgPool,
    channel_id: Uuid,
) -> Result<Vec<FileRecord>, sqlx::Error> {
    sqlx::query_as::<_, FileRecord>(
        "SELECT * FROM files WHERE channel_id = $1",
    )
    .bind(channel_id)
    .fetch_all(pool)
    .await
}

/// Hard-delete all file records for a user.
pub async fn delete_all_user_files(
    pool: &PgPool,
    uploader_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM files WHERE uploader_id = $1")
        .bind(uploader_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Hard-delete all file records for a channel.
pub async fn delete_channel_files(
    pool: &PgPool,
    channel_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM files WHERE channel_id = $1")
        .bind(channel_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Quarantine a file (admin).
pub async fn quarantine_file(
    pool: &PgPool,
    id: Uuid,
    quarantined_by: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE files SET quarantined_at = NOW(), quarantined_by = $2 WHERE id = $1 AND quarantined_at IS NULL",
    )
    .bind(id)
    .bind(quarantined_by)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Unquarantine a file (admin).
pub async fn unquarantine_file(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE files SET quarantined_at = NULL, quarantined_by = NULL WHERE id = $1 AND quarantined_at IS NOT NULL",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// List all file IDs and storage paths (for orphan cleanup background task).
pub async fn list_all_file_paths(pool: &PgPool) -> Result<Vec<(Uuid, String)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String)>("SELECT id, storage_path FROM files")
        .fetch_all(pool)
        .await
}
