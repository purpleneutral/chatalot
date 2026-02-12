use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use chatalot_common::api_types::{FileMetadataResponse, FileUploadResponse};
use chatalot_db::repos::{channel_repo, file_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/files/upload", post(upload_file))
        .route("/files/{file_id}", get(download_file))
        .route("/files/{file_id}/meta", get(get_file_meta))
}

/// Upload an encrypted file.
/// The client encrypts the file before upload — the server only stores the ciphertext blob.
async fn upload_file(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<FileUploadResponse>, AppError> {
    let max_size = state.config.max_file_size_mb * 1024 * 1024;

    let mut file_data: Option<Vec<u8>> = None;
    let mut encrypted_name = String::from("unnamed");
    let mut content_type: Option<String> = None;
    let mut channel_id: Option<Uuid> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                content_type = field.content_type().map(|s| s.to_string());
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("read error: {e}")))?;
                if bytes.len() as u64 > max_size {
                    return Err(AppError::Validation(format!(
                        "file too large (max {} MB)",
                        state.config.max_file_size_mb
                    )));
                }
                file_data = Some(bytes.to_vec());
            }
            "name" => {
                encrypted_name = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("read name: {e}")))?;
            }
            "channel_id" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("read channel_id: {e}")))?;
                channel_id = Uuid::parse_str(&text).ok();
            }
            _ => {}
        }
    }

    let data = file_data.ok_or_else(|| AppError::Validation("no file field".to_string()))?;

    // Verify channel membership if a channel_id was provided
    if let Some(cid) = channel_id
        && !channel_repo::is_member(&state.db, cid, claims.sub).await?
    {
        return Err(AppError::Forbidden);
    }

    let size_bytes = data.len() as i64;
    let checksum = hex::encode(Sha256::digest(&data));

    // Store the file on disk
    let file_id = Uuid::now_v7();
    let storage_dir = std::path::Path::new(&state.config.file_storage_path);
    // Use first 2 chars of ID as subdirectory for sharding
    let id_str = file_id.to_string();
    let shard_dir = storage_dir.join(&id_str[..2]);
    tokio::fs::create_dir_all(&shard_dir)
        .await
        .map_err(|e| AppError::Internal(format!("create dir: {e}")))?;

    let file_path = shard_dir.join(&id_str);
    let mut f = tokio::fs::File::create(&file_path)
        .await
        .map_err(|e| AppError::Internal(format!("create file: {e}")))?;
    f.write_all(&data)
        .await
        .map_err(|e| AppError::Internal(format!("write file: {e}")))?;
    f.flush()
        .await
        .map_err(|e| AppError::Internal(format!("flush file: {e}")))?;

    // Record metadata in DB
    let record = file_repo::create_file(
        &state.db,
        file_id,
        claims.sub,
        &encrypted_name,
        size_bytes,
        content_type.as_deref(),
        &file_path.to_string_lossy(),
        &checksum,
        channel_id,
    )
    .await?;

    Ok(Json(FileUploadResponse {
        id: record.id,
        size_bytes: record.size_bytes,
        created_at: record.created_at.to_rfc3339(),
    }))
}

/// Download an encrypted file blob.
async fn download_file(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(file_id): Path<Uuid>,
) -> Result<([(header::HeaderName, String); 2], Body), AppError> {
    let record = file_repo::get_file(&state.db, file_id)
        .await?
        .ok_or_else(|| AppError::NotFound("file not found".to_string()))?;

    // Verify the requesting user has access (member of the file's channel)
    if let Some(cid) = record.channel_id
        && !channel_repo::is_member(&state.db, cid, claims.sub).await?
    {
        return Err(AppError::Forbidden);
    }

    let file = tokio::fs::File::open(&record.storage_path)
        .await
        .map_err(|e| AppError::Internal(format!("open file: {e}")))?;

    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let content_type = record
        .content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());

    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CONTENT_DISPOSITION,
                format!(
                    "attachment; filename=\"{}\"",
                    record.encrypted_name.replace('"', "'").replace('\n', "_")
                ),
            ),
        ],
        body,
    ))
}

/// Get file metadata without downloading the blob.
async fn get_file_meta(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(file_id): Path<Uuid>,
) -> Result<Json<FileMetadataResponse>, AppError> {
    let record = file_repo::get_file(&state.db, file_id)
        .await?
        .ok_or_else(|| AppError::NotFound("file not found".to_string()))?;

    // Verify the requesting user has access
    if let Some(cid) = record.channel_id
        && !channel_repo::is_member(&state.db, cid, claims.sub).await?
    {
        return Err(AppError::Forbidden);
    }

    Ok(Json(FileMetadataResponse {
        id: record.id,
        uploader_id: record.uploader_id,
        encrypted_name: record.encrypted_name,
        size_bytes: record.size_bytes,
        content_type: record.content_type,
        checksum: record.checksum,
        created_at: record.created_at.to_rfc3339(),
    }))
}
