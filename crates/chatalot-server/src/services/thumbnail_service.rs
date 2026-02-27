use std::io::Cursor;
use std::path::Path;

use image::imageops::FilterType;
use image::ImageFormat;
use tokio::io::AsyncWriteExt;

const THUMB_MAX_DIM: u32 = 300;
const THUMB_JPEG_QUALITY: u8 = 80;

/// Generate a JPEG thumbnail for a raster image.
/// Returns `Ok(true)` if a thumbnail was generated, `Ok(false)` if the content
/// type is not a supported image format, or an error on I/O failure.
pub async fn generate_thumbnail(
    data: &[u8],
    content_type: &str,
    thumb_path: &Path,
) -> Result<bool, String> {
    let format = match content_type {
        "image/png" => ImageFormat::Png,
        "image/jpeg" => ImageFormat::Jpeg,
        "image/gif" => ImageFormat::Gif,
        "image/webp" => ImageFormat::WebP,
        "image/bmp" => ImageFormat::Bmp,
        _ => return Ok(false),
    };

    // Decode the image (blocking — run in spawn_blocking to avoid blocking the runtime)
    let data = data.to_vec();
    let thumb_bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, String> {
        let img = image::load_from_memory_with_format(&data, format)
            .map_err(|e| format!("decode image: {e}"))?;

        let thumb = img.resize(THUMB_MAX_DIM, THUMB_MAX_DIM, FilterType::Lanczos3);

        let mut buf = Cursor::new(Vec::new());
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, THUMB_JPEG_QUALITY);
        thumb
            .write_with_encoder(encoder)
            .map_err(|e| format!("encode thumbnail: {e}"))?;

        Ok(buf.into_inner())
    })
    .await
    .map_err(|e| format!("spawn_blocking: {e}"))??;

    let mut f = tokio::fs::File::create(thumb_path)
        .await
        .map_err(|e| format!("create thumbnail file: {e}"))?;
    f.write_all(&thumb_bytes)
        .await
        .map_err(|e| format!("write thumbnail: {e}"))?;

    Ok(true)
}

/// Strip EXIF/metadata from an image by decoding and re-encoding.
/// Returns the clean image bytes, or `None` if the format is not supported.
/// This naturally drops all EXIF, GPS, camera info, and other metadata.
pub async fn strip_exif(data: &[u8], content_type: &str) -> Option<Vec<u8>> {
    let format = match content_type {
        "image/jpeg" => ImageFormat::Jpeg,
        "image/png" => ImageFormat::Png,
        "image/webp" => ImageFormat::WebP,
        _ => return None,
    };

    let data = data.to_vec();
    let out_format = format;
    tokio::task::spawn_blocking(move || -> Option<Vec<u8>> {
        let img = image::load_from_memory_with_format(&data, out_format).ok()?;
        let mut buf = Cursor::new(Vec::new());
        match out_format {
            ImageFormat::Jpeg => {
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 95);
                img.write_with_encoder(encoder).ok()?;
            }
            ImageFormat::Png => {
                img.write_to(&mut buf, ImageFormat::Png).ok()?;
            }
            ImageFormat::WebP => {
                // Re-encode as PNG (lossless) since the image crate's WebP encoder
                // may not be available. The file stays functional and metadata-free.
                img.write_to(&mut buf, ImageFormat::Png).ok()?;
            }
            _ => return None,
        }
        Some(buf.into_inner())
    })
    .await
    .ok()
    .flatten()
}
