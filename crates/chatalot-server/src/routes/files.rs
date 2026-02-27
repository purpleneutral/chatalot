use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use dashmap::DashMap;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use chatalot_common::api_types::{FileMetadataResponse, FileUploadResponse};
use chatalot_db::repos::{blocked_hash_repo, channel_repo, file_repo, user_repo};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;
use crate::services::{file_security, thumbnail_service};

/// Per-user upload rate limiter: max 10 uploads per 60 seconds.
static UPLOAD_RATE: std::sync::LazyLock<DashMap<Uuid, (Instant, u32)>> =
    std::sync::LazyLock::new(DashMap::new);
const UPLOAD_RATE_WINDOW: u64 = 60;
const UPLOAD_RATE_MAX: u32 = 10;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/files/upload", post(upload_file))
        .route(
            "/files/{file_id}",
            get(download_file).delete(delete_own_file),
        )
        .route("/files/{file_id}/meta", get(get_file_meta))
        .route("/files/{file_id}/thumb", get(download_thumbnail))
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
                if encrypted_name.len() > 512 {
                    return Err(AppError::Validation("file name too long (max 512 bytes)".to_string()));
                }
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

    // Rate limiting: max 10 uploads per minute per user
    {
        let mut entry = UPLOAD_RATE.entry(claims.sub).or_insert((Instant::now(), 0));
        let (window_start, count) = entry.value_mut();
        if window_start.elapsed().as_secs() >= UPLOAD_RATE_WINDOW {
            *window_start = Instant::now();
            *count = 1;
        } else if *count >= UPLOAD_RATE_MAX {
            return Err(AppError::Validation(
                "upload rate limit exceeded (max 10 per minute)".to_string(),
            ));
        } else {
            *count += 1;
        }
    }

    // Verify channel membership if a channel_id was provided
    if let Some(cid) = channel_id
        && !channel_repo::is_member(&state.db, cid, claims.sub).await?
    {
        return Err(AppError::Forbidden);
    }

    let size_bytes = data.len() as i64;

    // Check per-user upload quota
    let quota_bytes = state.config.upload_quota_mb as i64 * 1024 * 1024;
    if quota_bytes > 0 {
        let used = user_repo::get_upload_bytes_used(&state.db, claims.sub).await?;
        if used + size_bytes > quota_bytes {
            let remaining_mb = ((quota_bytes - used) as f64 / 1024.0 / 1024.0).max(0.0);
            return Err(AppError::Validation(format!(
                "upload quota exceeded ({} MB limit, {:.1} MB remaining)",
                state.config.upload_quota_mb, remaining_mb
            )));
        }
    }

    let checksum = hex::encode(Sha256::digest(&data));

    // Security: check if this file hash is blocked
    if blocked_hash_repo::is_hash_blocked(&state.db, &checksum).await? {
        return Err(AppError::Validation(
            "this file has been blocked by an administrator".to_string(),
        ));
    }

    // Security: validate file type via magic bytes and use detected type
    let detected_type = match file_security::validate_file_type(&data) {
        Ok(detected) => Some(detected.to_string()),
        Err(reason) => {
            return Err(AppError::Validation(format!(
                "file type not allowed: {reason}"
            )));
        }
    };

    // Prefer server-detected content type over client-claimed MIME
    let content_type = detected_type.or(content_type);

    // Store the file on disk
    let file_id = Uuid::now_v7();
    let storage_dir = std::path::Path::new(&state.config.file_storage_path);
    // Use first 2 chars of ID as subdirectory for sharding
    let id_str = file_id.to_string();
    let shard_dir = storage_dir.join(&id_str[..2]);
    tokio::fs::create_dir_all(&shard_dir)
        .await
        .map_err(|e| AppError::Internal(format!("create dir: {e}")))?;

    // Security: strip EXIF metadata and sanitize SVGs before storing
    let mut data = data;
    let mut exif_stripped = false;

    if let Some(ref ct) = content_type {
        // Strip EXIF from images (removes GPS coordinates, camera info, etc.)
        if matches!(ct.as_str(), "image/jpeg" | "image/png" | "image/webp") {
            if let Some(clean) = thumbnail_service::strip_exif(&data, ct).await {
                data = clean;
                exif_stripped = true;
            }
        }

        // Sanitize SVG files (strip scripts, event handlers, etc.)
        if ct == "image/svg+xml" {
            match file_security::sanitize_svg(&data) {
                Ok(clean) => data = clean,
                Err(reason) => {
                    return Err(AppError::Validation(format!("invalid SVG: {reason}")));
                }
            }
        }
    }

    // Update size after potential re-encoding
    let size_bytes = data.len() as i64;

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

    // Generate thumbnail for image files
    let mut thumbnail_path: Option<String> = None;
    if let Some(ref ct) = content_type {
        let thumb_file = shard_dir.join(format!("{id_str}_thumb"));
        match thumbnail_service::generate_thumbnail(&data, ct, &thumb_file).await {
            Ok(true) => {
                thumbnail_path = Some(thumb_file.to_string_lossy().to_string());
            }
            Ok(false) => {} // not an image type
            Err(e) => {
                tracing::warn!("thumbnail generation failed for {id_str}: {e}");
                // Non-fatal — upload still succeeds without thumbnail
            }
        }
    }

    // Record metadata in DB (clean up orphaned file on failure)
    let record = match file_repo::create_file(
        &state.db,
        file_id,
        claims.sub,
        &encrypted_name,
        size_bytes,
        content_type.as_deref(),
        &file_path.to_string_lossy(),
        &checksum,
        channel_id,
        thumbnail_path.as_deref(),
        exif_stripped,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            let _ = tokio::fs::remove_file(&file_path).await;
            if let Some(ref tp) = thumbnail_path {
                let _ = tokio::fs::remove_file(tp).await;
            }
            return Err(e.into());
        }
    };

    // Track quota usage
    user_repo::increment_upload_bytes(&state.db, claims.sub, size_bytes).await?;

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
) -> Result<([(header::HeaderName, String); 3], Body), AppError> {
    let record = file_repo::get_file(&state.db, file_id)
        .await?
        .ok_or_else(|| AppError::NotFound("file not found".to_string()))?;

    // Block download of quarantined files (unless admin)
    if record.quarantined_at.is_some() && !claims.is_admin && !claims.is_owner {
        return Err(AppError::Forbidden);
    }

    // Verify the requesting user has access
    if let Some(cid) = record.channel_id {
        if !channel_repo::is_member(&state.db, cid, claims.sub).await? {
            return Err(AppError::Forbidden);
        }
    } else if record.uploader_id != claims.sub && !claims.is_admin && !claims.is_owner {
        // Files without a channel are only accessible to the uploader (or admins)
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
            (header::CACHE_CONTROL, "private, no-store".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!(
                    "attachment; filename=\"{}\"",
                    record.encrypted_name.replace('"', "'").replace(['\n', '\r'], "_")
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
    if let Some(cid) = record.channel_id {
        if !channel_repo::is_member(&state.db, cid, claims.sub).await? {
            return Err(AppError::Forbidden);
        }
    } else if record.uploader_id != claims.sub && !claims.is_admin && !claims.is_owner {
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

/// Download the thumbnail for a file.
async fn download_thumbnail(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(file_id): Path<Uuid>,
) -> Result<([(header::HeaderName, String); 3], Body), AppError> {
    let record = file_repo::get_file(&state.db, file_id)
        .await?
        .ok_or_else(|| AppError::NotFound("file not found".to_string()))?;

    let thumb_path = record
        .thumbnail_path
        .as_ref()
        .ok_or_else(|| AppError::NotFound("no thumbnail available".to_string()))?;

    // Same access control as file download
    if record.quarantined_at.is_some() && !claims.is_admin && !claims.is_owner {
        return Err(AppError::Forbidden);
    }
    if let Some(cid) = record.channel_id {
        if !channel_repo::is_member(&state.db, cid, claims.sub).await? {
            return Err(AppError::Forbidden);
        }
    } else if record.uploader_id != claims.sub && !claims.is_admin && !claims.is_owner {
        return Err(AppError::Forbidden);
    }

    let file = tokio::fs::File::open(thumb_path)
        .await
        .map_err(|e| AppError::Internal(format!("open thumbnail: {e}")))?;

    let stream = tokio_util::io::ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // Thumbnails are always JPEG (except sanitized SVGs)
    let ct = if record.content_type.as_deref() == Some("image/svg+xml") {
        "image/svg+xml".to_string()
    } else {
        "image/jpeg".to_string()
    };

    Ok((
        [
            (header::CONTENT_TYPE, ct),
            (header::CACHE_CONTROL, "private, max-age=3600".to_string()),
            (header::CONTENT_DISPOSITION, "inline".to_string()),
        ],
        body,
    ))
}

/// Delete a file. Owner can delete own files; admins can delete any file.
async fn delete_own_file(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<AccessClaims>,
    Path(file_id): Path<Uuid>,
) -> Result<(), AppError> {
    let record = file_repo::get_file(&state.db, file_id)
        .await?
        .ok_or_else(|| AppError::NotFound("file not found".to_string()))?;

    let is_admin = claims.is_admin || claims.is_owner;

    // Check ownership or admin
    if record.uploader_id != claims.sub && !is_admin {
        return Err(AppError::Forbidden);
    }

    // Delete from disk
    if let Err(e) = tokio::fs::remove_file(&record.storage_path).await {
        tracing::warn!(
            "Failed to delete file from disk {}: {e}",
            record.storage_path
        );
    }
    // Delete thumbnail if present
    if let Some(ref tp) = record.thumbnail_path {
        let _ = tokio::fs::remove_file(tp).await;
    }

    // Delete from DB
    if is_admin {
        file_repo::delete_file_admin(&state.db, file_id).await?;
    } else {
        file_repo::delete_file(&state.db, file_id, claims.sub).await?;
    }

    // Return quota to uploader
    user_repo::decrement_upload_bytes(&state.db, record.uploader_id, record.size_bytes).await?;

    // Audit log
    chatalot_db::repos::user_repo::insert_audit_log(
        &state.db,
        Uuid::now_v7(),
        Some(claims.sub),
        "file_deleted",
        None,
        None,
        Some(serde_json::json!({
            "file_id": file_id,
            "uploader_id": record.uploader_id,
        })),
    )
    .await?;

    Ok(())
}
