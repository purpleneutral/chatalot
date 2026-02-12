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
