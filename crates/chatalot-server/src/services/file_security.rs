/// Validate file content against a whitelist of known-safe file types using magic bytes.
///
/// Returns `Ok(detected_content_type)` for allowed files, or `Err(reason)` for rejected files.
/// Files are checked against binary signatures first, then unknown binary data is rejected.
/// Valid UTF-8 text (code, markdown, JSON, etc.) is always allowed.
pub fn validate_file_type(data: &[u8]) -> Result<&'static str, &'static str> {
    if data.is_empty() {
        return Err("empty file");
    }

    // Check for explicitly dangerous formats first
    if let Some(reason) = check_dangerous(data) {
        return Err(reason);
    }

    // Check against known-safe binary signatures
    if let Some(content_type) = check_whitelist(data) {
        return Ok(content_type);
    }

    // Allow valid UTF-8 text (code, markdown, JSON, config files, etc.)
    // Check a reasonable prefix to avoid scanning huge binary blobs
    let check_len = data.len().min(8192);
    if std::str::from_utf8(&data[..check_len]).is_ok() && !data[..check_len].contains(&0) {
        return Ok("text/plain");
    }

    // Unknown binary format — reject
    Err("unrecognized binary file type")
}

/// Check for explicitly dangerous file signatures.
fn check_dangerous(data: &[u8]) -> Option<&'static str> {
    if data.len() < 2 {
        return None;
    }

    // PE executable (Windows .exe, .dll)
    if data[..2] == *b"MZ" {
        return Some("Windows executable (PE) not allowed");
    }

    // ELF binary (Linux executables)
    if data.len() >= 4 && data[..4] == *b"\x7fELF" {
        return Some("Linux executable (ELF) not allowed");
    }

    // Mach-O binary (macOS executables)
    if data.len() >= 4 {
        let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        if matches!(magic, 0xFEEDFACE | 0xFEEDFACF | 0xCEFAEDFE | 0xCFFAEDFE | 0xCAFEBABE) {
            // 0xCAFEBABE is also Java class / Mach-O fat binary
            return Some("macOS/Java executable not allowed");
        }
    }

    // Shell scripts
    if data.len() >= 2 && data[..2] == *b"#!" {
        return Some("shell script not allowed");
    }

    // Windows batch files
    if data.len() >= 10 {
        let lower: Vec<u8> = data[..10.min(data.len())].iter().map(|b| b.to_ascii_lowercase()).collect();
        if lower.starts_with(b"@echo off") || lower.starts_with(b"@echo on") {
            return Some("batch script not allowed");
        }
    }

    None
}

/// Check data against a whitelist of known-safe binary file signatures.
fn check_whitelist(data: &[u8]) -> Option<&'static str> {
    // Images
    if starts(data, b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if starts(data, b"\xFF\xD8\xFF") {
        return Some("image/jpeg");
    }
    if starts(data, b"GIF87a") || starts(data, b"GIF89a") {
        return Some("image/gif");
    }
    if starts(data, b"RIFF") && data.len() >= 12 && &data[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if starts(data, b"BM") && data.len() >= 6 {
        return Some("image/bmp");
    }

    // Audio
    if starts(data, b"ID3") || starts(data, b"\xFF\xFB") || starts(data, b"\xFF\xF3") || starts(data, b"\xFF\xF2") {
        return Some("audio/mpeg");
    }
    if starts(data, b"OggS") {
        return Some("audio/ogg");
    }
    if starts(data, b"fLaC") {
        return Some("audio/flac");
    }
    if starts(data, b"RIFF") && data.len() >= 12 && &data[8..12] == b"WAVE" {
        return Some("audio/wav");
    }

    // Video
    if data.len() >= 8 && &data[4..8] == b"ftyp" {
        return Some("video/mp4");
    }
    if starts(data, b"\x1A\x45\xDF\xA3") {
        return Some("video/webm"); // also MKV/matroska
    }

    // Documents
    if starts(data, b"%PDF") {
        return Some("application/pdf");
    }

    // Archives / compound documents
    if starts(data, b"PK\x03\x04") {
        return Some("application/zip"); // also docx, xlsx, pptx, odt, jar
    }
    if starts(data, b"\x1F\x8B") {
        return Some("application/gzip");
    }
    if starts(data, b"BZh") {
        return Some("application/x-bzip2");
    }
    if starts(data, b"\xFD7zXZ\x00") {
        return Some("application/x-xz");
    }
    if starts(data, b"7z\xBC\xAF\x27\x1C") {
        return Some("application/x-7z-compressed");
    }
    if starts(data, b"Rar!\x1A\x07") {
        return Some("application/x-rar-compressed");
    }

    // WASM module
    if starts(data, b"\x00asm") {
        return Some("application/wasm");
    }

    None
}

fn starts(data: &[u8], prefix: &[u8]) -> bool {
    data.len() >= prefix.len() && &data[..prefix.len()] == prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_accepted() {
        let data = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        assert_eq!(validate_file_type(data), Ok("image/png"));
    }

    #[test]
    fn test_jpeg_accepted() {
        let data = b"\xFF\xD8\xFF\xE0\x00\x10JFIF";
        assert_eq!(validate_file_type(data), Ok("image/jpeg"));
    }

    #[test]
    fn test_gif_accepted() {
        let data = b"GIF89a\x01\x00\x01\x00";
        assert_eq!(validate_file_type(data), Ok("image/gif"));
    }

    #[test]
    fn test_webp_accepted() {
        let mut data = vec![0u8; 12];
        data[..4].copy_from_slice(b"RIFF");
        data[8..12].copy_from_slice(b"WEBP");
        assert_eq!(validate_file_type(&data), Ok("image/webp"));
    }

    #[test]
    fn test_pdf_accepted() {
        let data = b"%PDF-1.7\n";
        assert_eq!(validate_file_type(data), Ok("application/pdf"));
    }

    #[test]
    fn test_zip_accepted() {
        let data = b"PK\x03\x04\x14\x00\x00\x00";
        assert_eq!(validate_file_type(data), Ok("application/zip"));
    }

    #[test]
    fn test_mp4_accepted() {
        let data = b"\x00\x00\x00\x18ftypmp42";
        assert_eq!(validate_file_type(data), Ok("video/mp4"));
    }

    #[test]
    fn test_mp3_id3_accepted() {
        let data = b"ID3\x03\x00\x00\x00\x00";
        assert_eq!(validate_file_type(data), Ok("audio/mpeg"));
    }

    #[test]
    fn test_gzip_accepted() {
        let data = b"\x1F\x8B\x08\x00\x00\x00\x00\x00";
        assert_eq!(validate_file_type(data), Ok("application/gzip"));
    }

    #[test]
    fn test_utf8_text_accepted() {
        let data = b"fn main() {\n    println!(\"hello\");\n}\n";
        assert_eq!(validate_file_type(data), Ok("text/plain"));
    }

    #[test]
    fn test_json_accepted() {
        let data = b"{\"key\": \"value\", \"num\": 42}";
        assert_eq!(validate_file_type(data), Ok("text/plain"));
    }

    #[test]
    fn test_exe_rejected() {
        let data = b"MZ\x90\x00\x03\x00\x00\x00";
        assert!(validate_file_type(data).is_err());
        assert!(validate_file_type(data).unwrap_err().contains("PE"));
    }

    #[test]
    fn test_elf_rejected() {
        let data = b"\x7fELF\x02\x01\x01\x00";
        assert!(validate_file_type(data).is_err());
        assert!(validate_file_type(data).unwrap_err().contains("ELF"));
    }

    #[test]
    fn test_shell_script_rejected() {
        let data = b"#!/bin/bash\necho hello\n";
        assert!(validate_file_type(data).is_err());
        assert!(validate_file_type(data).unwrap_err().contains("shell"));
    }

    #[test]
    fn test_batch_rejected() {
        let data = b"@echo off\ndir\n";
        assert!(validate_file_type(data).is_err());
        assert!(validate_file_type(data).unwrap_err().contains("batch"));
    }

    #[test]
    fn test_empty_rejected() {
        assert_eq!(validate_file_type(b""), Err("empty file"));
    }

    #[test]
    fn test_unknown_binary_rejected() {
        let data = [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01, 0x02, 0x03];
        assert!(validate_file_type(&data).is_err());
        assert!(validate_file_type(&data).unwrap_err().contains("unrecognized"));
    }

    #[test]
    fn test_wav_accepted() {
        let mut data = vec![0u8; 12];
        data[..4].copy_from_slice(b"RIFF");
        data[8..12].copy_from_slice(b"WAVE");
        assert_eq!(validate_file_type(&data), Ok("audio/wav"));
    }

    #[test]
    fn test_flac_accepted() {
        let data = b"fLaC\x00\x00\x00\x22";
        assert_eq!(validate_file_type(data), Ok("audio/flac"));
    }

    #[test]
    fn test_webm_accepted() {
        let data = b"\x1A\x45\xDF\xA3\x93\x42\x86";
        assert_eq!(validate_file_type(data), Ok("video/webm"));
    }

    #[test]
    fn test_7z_accepted() {
        let data = b"7z\xBC\xAF\x27\x1C\x00\x04";
        assert_eq!(validate_file_type(data), Ok("application/x-7z-compressed"));
    }
}
