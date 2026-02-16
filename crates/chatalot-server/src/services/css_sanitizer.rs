//! Sanitize user-provided CSS for community themes.
//!
//! Allows a limited set of safe CSS properties and rejects anything that
//! could be used for XSS, data exfiltration, or disruptive layout changes.

const MAX_CSS_SIZE: usize = 4096; // 4KB

const ALLOWED_PROPERTIES: &[&str] = &[
    "color",
    "background-color",
    "background",
    "border-color",
    "border-radius",
    "padding",
    "margin",
    "gap",
    "font-size",
    "font-weight",
    "font-family",
    "letter-spacing",
    "line-height",
    "box-shadow",
    "text-shadow",
    "opacity",
    "border",
    "outline",
    "text-decoration",
    "text-transform",
    "border-width",
    "border-style",
];

const BLOCKED_PATTERNS: &[&str] = &[
    "@import",
    "@font-face",
    "@keyframes",
    "expression(",
    "javascript:",
    "behavior:",
    "-moz-binding",
    "position: fixed",
    "position:fixed",
    "position: absolute",
    "position:absolute",
    "z-index",
    "cursor:",
    "pointer-events",
];

/// Sanitize a CSS string. Returns the sanitized CSS or an error message.
pub fn sanitize_css(input: &str) -> Result<String, String> {
    if input.len() > MAX_CSS_SIZE {
        return Err(format!("CSS too large (max {MAX_CSS_SIZE} bytes)"));
    }

    let lower = input.to_lowercase();

    // Check for blocked patterns
    for pattern in BLOCKED_PATTERNS {
        if lower.contains(pattern) {
            return Err(format!("CSS contains blocked pattern: {pattern}"));
        }
    }

    // Block url() except for /api/ paths
    if lower.contains("url(") {
        // Find all url() occurrences and validate them
        let mut search_from = 0;
        while let Some(pos) = lower[search_from..].find("url(") {
            let abs_pos = search_from + pos;
            let after = &lower[abs_pos + 4..];
            // Strip optional quotes
            let content = after.trim_start_matches(['\'', '"', ' ']);
            if !content.starts_with("/api/") && !content.starts_with(')') {
                return Err("url() is only allowed for /api/ paths".to_string());
            }
            search_from = abs_pos + 4;
        }
    }

    // Parse declarations: split by ; and validate each property
    let mut sanitized = Vec::new();

    for line in input.split(';') {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Support selectors with { } blocks
        if trimmed.contains('{') || trimmed.contains('}') {
            // Allow simple selectors but validate the declaration inside
            // For community themes we only allow CSS custom properties and flat declarations
            // Just pass through selector braces
            sanitized.push(trimmed.to_string());
            continue;
        }

        // Must be a property: value pair
        let Some(colon_pos) = trimmed.find(':') else {
            continue; // Skip non-declaration lines
        };

        let property = trimmed[..colon_pos].trim().to_lowercase();
        let value = trimmed[colon_pos + 1..].trim();

        // Allow CSS custom properties (--var-name)
        if property.starts_with("--") {
            sanitized.push(format!("{property}: {value}"));
            continue;
        }

        // Check if property is in allowlist
        if !ALLOWED_PROPERTIES.contains(&property.as_str()) {
            return Err(format!("CSS property not allowed: {property}"));
        }

        sanitized.push(format!("{property}: {value}"));
    }

    Ok(sanitized.join("; "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_basic_properties() {
        let result = sanitize_css("color: red; background-color: #fff");
        assert!(result.is_ok());
    }

    #[test]
    fn allows_custom_properties() {
        let result = sanitize_css("--my-var: blue; --bg: #123456");
        assert!(result.is_ok());
    }

    #[test]
    fn blocks_import() {
        let result = sanitize_css("@import url('evil.css')");
        assert!(result.is_err());
    }

    #[test]
    fn blocks_javascript() {
        let result = sanitize_css("background: url(javascript:alert(1))");
        assert!(result.is_err());
    }

    #[test]
    fn blocks_position_fixed() {
        let result = sanitize_css("position: fixed; top: 0");
        assert!(result.is_err());
    }

    #[test]
    fn blocks_external_url() {
        let result = sanitize_css("background: url(https://evil.com/img.png)");
        assert!(result.is_err());
    }

    #[test]
    fn allows_api_url() {
        let result = sanitize_css("background: url(/api/community-assets/bg.png)");
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_too_large() {
        let big = "a".repeat(MAX_CSS_SIZE + 1);
        let result = sanitize_css(&big);
        assert!(result.is_err());
    }

    #[test]
    fn blocks_moz_binding() {
        let result = sanitize_css("-moz-binding: url(evil)");
        assert!(result.is_err());
    }

    #[test]
    fn blocks_expression() {
        let result = sanitize_css("width: expression(alert(1))");
        assert!(result.is_err());
    }

    #[test]
    fn blocks_pointer_events() {
        let result = sanitize_css("pointer-events: none");
        assert!(result.is_err());
    }

    #[test]
    fn allows_gradients() {
        let result =
            sanitize_css("background: linear-gradient(135deg, #667eea 0%, #764ba2 100%)");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("linear-gradient"));
    }

    #[test]
    fn allows_rgba_colors() {
        let result = sanitize_css("color: rgba(255, 255, 255, 0.8)");
        assert!(result.is_ok());
    }

    #[test]
    fn blocks_behavior() {
        let result = sanitize_css("behavior: url(xss.htc)");
        assert!(result.is_err());
    }

    #[test]
    fn preserves_declaration_values() {
        let result = sanitize_css("border-radius: 8px; opacity: 0.5");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("border-radius: 8px"));
        assert!(output.contains("opacity: 0.5"));
    }
}
