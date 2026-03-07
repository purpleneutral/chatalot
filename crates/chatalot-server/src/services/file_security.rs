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
        // Detect SVG specifically (text-based image format)
        if is_svg(&data[..check_len]) {
            return Ok("image/svg+xml");
        }
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
        if matches!(
            magic,
            0xFEEDFACE | 0xFEEDFACF | 0xCEFAEDFE | 0xCFFAEDFE | 0xCAFEBABE
        ) {
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
        let lower: Vec<u8> = data[..10.min(data.len())]
            .iter()
            .map(|b| b.to_ascii_lowercase())
            .collect();
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
    if starts(data, b"ID3")
        || starts(data, b"\xFF\xFB")
        || starts(data, b"\xFF\xF3")
        || starts(data, b"\xFF\xF2")
    {
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

/// Check if UTF-8 text looks like an SVG file.
fn is_svg(text: &[u8]) -> bool {
    let s = match std::str::from_utf8(text) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let trimmed = s.trim_start();
    // SVG files start with <svg or <?xml (then contain <svg)
    trimmed.starts_with("<svg")
        || (trimmed.starts_with("<?xml") && trimmed.contains("<svg"))
}

/// Allowed SVG element names (lowercase). Everything else is stripped.
const ALLOWED_SVG_ELEMENTS: &[&str] = &[
    "svg", "g", "defs", "symbol", "use", "title", "desc",
    "circle", "ellipse", "line", "path", "polygon", "polyline", "rect",
    "text", "tspan", "textpath",
    "clippath", "mask", "pattern", "marker",
    "lineargradient", "radialgradient", "stop",
    "filter", "fegaussianblur", "feoffset", "feblend", "fecolormatrix",
    "fecomponenttransfer", "fecomposite", "feconvolvematrix", "fediffuselighting",
    "fedisplacementmap", "feflood", "feimage", "femerge", "femergenode",
    "femorphology", "fespecularlighting", "fetile", "feturbulence",
    "image",
];

/// Allowed SVG attribute prefixes/names (lowercase). Everything else is stripped.
fn is_allowed_attribute(name: &str) -> bool {
    let lower = name.to_lowercase();
    // Block all event handlers
    if lower.starts_with("on") {
        return false;
    }
    matches!(
        lower.as_str(),
        "id" | "class" | "style" | "transform" | "d" | "fill" | "stroke"
        | "stroke-width" | "stroke-linecap" | "stroke-linejoin" | "stroke-dasharray"
        | "stroke-dashoffset" | "stroke-opacity" | "fill-opacity" | "opacity"
        | "cx" | "cy" | "r" | "rx" | "ry" | "x" | "y" | "x1" | "y1" | "x2" | "y2"
        | "width" | "height" | "viewbox" | "preserveaspectratio" | "xmlns"
        | "points" | "dx" | "dy" | "text-anchor" | "dominant-baseline"
        | "font-size" | "font-family" | "font-weight" | "font-style"
        | "letter-spacing" | "text-decoration"
        | "offset" | "stop-color" | "stop-opacity" | "gradientunits" | "gradienttransform"
        | "patternunits" | "patterntransform" | "patterncontentunits"
        | "clip-path" | "clip-rule" | "mask" | "filter" | "flood-color" | "flood-opacity"
        | "color-interpolation-filters" | "lighting-color"
        | "markerwidth" | "markerheight" | "orient" | "markerunits" | "refx" | "refy"
        | "result" | "in" | "in2" | "stddeviation" | "values" | "type" | "mode"
        | "k1" | "k2" | "k3" | "k4" | "operator" | "basefrequency" | "numoctaves"
        | "seed" | "stitchtiles" | "surfacescale" | "specularconstant" | "specularexponent"
        | "role" | "aria-label" | "aria-hidden" | "focusable" | "tabindex"
        | "visibility" | "display" | "overflow" | "color"
        | "vector-effect" | "shape-rendering" | "image-rendering"
    ) || lower.starts_with("data-")
}

/// Check if an href/xlink:href value is safe (only fragment references allowed).
fn is_safe_href(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.is_empty() || trimmed.starts_with('#')
}

/// Sanitize SVG content using an XML parser with element/attribute allowlists.
/// Returns the sanitized SVG bytes, or an error if the input is invalid.
pub fn sanitize_svg(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    use quick_xml::events::{BytesEnd, BytesStart, Event};
    use quick_xml::{Reader, Writer};
    use std::io::Cursor;

    let text = std::str::from_utf8(data).map_err(|_| "SVG is not valid UTF-8")?;

    let mut reader = Reader::from_str(text);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut skip_depth: usize = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                if skip_depth > 0 {
                    skip_depth += 1;
                    continue;
                }
                if !ALLOWED_SVG_ELEMENTS.contains(&tag_name.as_str()) {
                    skip_depth = 1;
                    continue;
                }
                // Filter attributes
                let mut clean = BytesStart::new(tag_name.clone());
                for attr in e.attributes().flatten() {
                    let attr_name = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
                    if !is_allowed_attribute(&attr_name) {
                        continue;
                    }
                    let value = String::from_utf8_lossy(&attr.value);
                    // Block javascript: in any attribute value
                    if value.trim().to_lowercase().starts_with("javascript:") {
                        continue;
                    }
                    // Restrict href to fragment-only references
                    if (attr_name == "href" || attr_name == "xlink:href")
                        && !is_safe_href(&value)
                    {
                        continue;
                    }
                    clean.push_attribute((attr_name.as_str(), value.as_ref()));
                }
                writer.write_event(Event::Start(clean)).map_err(|_| "SVG write error")?;
            }
            Ok(Event::End(ref e)) => {
                if skip_depth > 0 {
                    skip_depth -= 1;
                    continue;
                }
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                if ALLOWED_SVG_ELEMENTS.contains(&tag_name.as_str()) {
                    writer
                        .write_event(Event::End(BytesEnd::new(tag_name)))
                        .map_err(|_| "SVG write error")?;
                }
            }
            Ok(Event::Empty(ref e)) => {
                if skip_depth > 0 {
                    continue;
                }
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                if !ALLOWED_SVG_ELEMENTS.contains(&tag_name.as_str()) {
                    continue;
                }
                let mut clean = BytesStart::new(tag_name.clone());
                for attr in e.attributes().flatten() {
                    let attr_name = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
                    if !is_allowed_attribute(&attr_name) {
                        continue;
                    }
                    let value = String::from_utf8_lossy(&attr.value);
                    if value.trim().to_lowercase().starts_with("javascript:") {
                        continue;
                    }
                    if (attr_name == "href" || attr_name == "xlink:href")
                        && !is_safe_href(&value)
                    {
                        continue;
                    }
                    clean.push_attribute((attr_name.as_str(), value.as_ref()));
                }
                writer
                    .write_event(Event::Empty(clean))
                    .map_err(|_| "SVG write error")?;
            }
            Ok(Event::Text(ref e)) => {
                if skip_depth == 0 {
                    writer.write_event(Event::Text(e.clone())).map_err(|_| "SVG write error")?;
                }
            }
            Ok(Event::Decl(ref e)) => {
                writer.write_event(Event::Decl(e.clone())).map_err(|_| "SVG write error")?;
            }
            Ok(Event::Comment(_)) | Ok(Event::CData(_)) | Ok(Event::PI(_)) | Ok(Event::DocType(_)) => {
                // Strip comments, CDATA, processing instructions, and doctypes
            }
            Err(_) => return Err("SVG XML parsing error"),
        }
    }

    Ok(writer.into_inner().into_inner())
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
        assert!(
            validate_file_type(&data)
                .unwrap_err()
                .contains("unrecognized")
        );
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

    #[test]
    fn test_svg_detected() {
        let data = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><rect/></svg>";
        assert_eq!(validate_file_type(data), Ok("image/svg+xml"));
    }

    #[test]
    fn test_svg_with_xml_decl() {
        let data = b"<?xml version=\"1.0\"?><svg><circle/></svg>";
        assert_eq!(validate_file_type(data), Ok("image/svg+xml"));
    }

    #[test]
    fn test_sanitize_svg_removes_script() {
        let svg = b"<svg><script>alert('xss')</script><rect/></svg>";
        let clean = sanitize_svg(svg).unwrap();
        let text = std::str::from_utf8(&clean).unwrap();
        assert!(!text.contains("<script"));
        assert!(text.contains("<rect"));
    }

    #[test]
    fn test_sanitize_svg_removes_on_events() {
        let svg = b"<svg><rect onclick=\"alert(1)\" fill=\"red\"/></svg>";
        let clean = sanitize_svg(svg).unwrap();
        let text = std::str::from_utf8(&clean).unwrap();
        assert!(!text.contains("onclick"));
        assert!(text.contains("fill=\"red\""));
    }

    #[test]
    fn test_sanitize_svg_removes_javascript_href() {
        let svg = b"<svg><a href=\"javascript:alert(1)\"><text>click</text></a></svg>";
        let clean = sanitize_svg(svg).unwrap();
        let text = std::str::from_utf8(&clean).unwrap();
        assert!(!text.contains("javascript:"));
    }

    #[test]
    fn test_sanitize_svg_removes_foreign_object() {
        let svg = b"<svg><foreignObject><div>html</div></foreignObject><rect/></svg>";
        let clean = sanitize_svg(svg).unwrap();
        let text = std::str::from_utf8(&clean).unwrap();
        assert!(!text.contains("<foreignObject"));
        assert!(text.contains("<rect"));
    }
}
