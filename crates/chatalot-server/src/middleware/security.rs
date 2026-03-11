use axum::body::Body;
use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use http_body_util::BodyExt;
use rand::RngCore;

/// Static CSP used for non-HTML responses (API, assets, etc.).
const CSP_NO_INLINE: &str = "\
    default-src 'self'; \
    script-src 'self' 'wasm-unsafe-eval' blob:; \
    style-src 'self' 'unsafe-inline'; \
    connect-src 'self' wss://chatalot.seglamater.app wss://chatalot.qlab wss://localhost:* https://cdn.jsdelivr.net; \
    img-src 'self' data: blob: https://media0.giphy.com https://media1.giphy.com https://media2.giphy.com https://media3.giphy.com https://media4.giphy.com; \
    media-src 'self' blob:; \
    worker-src 'self' blob:; \
    frame-ancestors 'self' tauri://localhost; \
    base-uri 'self'; \
    form-action 'self'; \
    object-src 'none'";

/// Generate a cryptographically random 128-bit nonce, base64-encoded.
fn generate_nonce() -> String {
    let mut buf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut buf);
    data_encoding::BASE64.encode(&buf)
}

/// Build the CSP string with a per-request nonce for inline scripts.
fn csp_with_nonce(nonce: &str) -> String {
    format!(
        "default-src 'self'; \
         script-src 'self' 'wasm-unsafe-eval' blob: 'nonce-{nonce}'; \
         style-src 'self' 'unsafe-inline'; \
         connect-src 'self' wss://chatalot.seglamater.app wss://chatalot.qlab wss://localhost:* https://cdn.jsdelivr.net; \
         img-src 'self' data: blob: https://media0.giphy.com https://media1.giphy.com https://media2.giphy.com https://media3.giphy.com https://media4.giphy.com; \
         media-src 'self' blob:; \
         worker-src 'self' blob:; \
         frame-ancestors 'self' tauri://localhost; \
         base-uri 'self'; \
         form-action 'self'; \
         object-src 'none'"
    )
}

/// Inject `nonce="..."` into every `<script` tag in the HTML.
///
/// Handles `<script>`, `<script `, and is case-insensitive for the tag name.
fn inject_nonce(html: &str, nonce: &str) -> String {
    let attr = format!(r#" nonce="{nonce}""#);
    let mut result = String::with_capacity(html.len() + attr.len() * 8);
    let lower = html.to_ascii_lowercase();
    let tag = "<script";
    let mut pos = 0;

    while let Some(idx) = lower[pos..].find(tag) {
        let abs = pos + idx;
        // Copy everything up to and including "<script"
        result.push_str(&html[pos..abs + tag.len()]);
        // Insert nonce attribute
        result.push_str(&attr);
        pos = abs + tag.len();
    }
    // Copy the remainder
    result.push_str(&html[pos..]);
    result
}

/// Add security headers to all responses.
///
/// For HTML responses, a per-request nonce is generated and injected into all
/// `<script>` tags so they pass the Content-Security-Policy without needing
/// `'unsafe-inline'`.
pub async fn security_headers(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;

    // --- Common headers (applied to every response) ---
    let headers = response.headers_mut();

    // Cache control: immutable hashed assets get long cache, HTML gets no-cache
    if path.starts_with("/_app/immutable/") {
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    } else if !path.starts_with("/api") {
        headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
    }

    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.remove("X-Frame-Options");
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("camera=(self), microphone=(self), geolocation=()"),
    );

    // --- CSP: static for non-HTML, nonce-injected for HTML ---
    let is_html = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.contains("text/html"));

    if !is_html {
        response.headers_mut().insert(
            "Content-Security-Policy",
            HeaderValue::from_static(CSP_NO_INLINE),
        );
        return response;
    }

    // HTML response: generate nonce, rewrite body, set dynamic CSP.
    let nonce = generate_nonce();

    // Set the CSP header with the nonce before consuming the body
    let csp = csp_with_nonce(&nonce);
    if let Ok(val) = HeaderValue::from_str(&csp) {
        response.headers_mut().insert("Content-Security-Policy", val);
    }

    // Collect the response body, inject nonces, and rebuild
    let (parts, body) = response.into_parts();
    match body.collect().await {
        Ok(collected) => {
            let bytes = collected.to_bytes();
            // Only attempt nonce injection if the body is valid UTF-8
            match std::str::from_utf8(&bytes) {
                Ok(html) => {
                    let patched = inject_nonce(html, &nonce);
                    Response::from_parts(parts, Body::from(patched))
                }
                Err(_) => {
                    // Not valid UTF-8 — return as-is (shouldn't happen for HTML)
                    Response::from_parts(parts, Body::from(bytes))
                }
            }
        }
        Err(_) => {
            // Body collection failed — return a minimal error response
            Response::from_parts(parts, Body::from("internal error"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonce_injected_into_script_tags() {
        let html = r#"<html><head><script>console.log("hi")</script></head><body><script type="module" src="/app.js"></script></body></html>"#;
        let result = inject_nonce(html, "abc123");
        assert!(result.contains(r#"<script nonce="abc123">"#));
        assert!(result.contains(r#"<script nonce="abc123" type="module""#));
        assert!(!result.contains("<script>"));
        assert!(!result.contains(r#"<script type"#));
    }

    #[test]
    fn nonce_handles_mixed_case() {
        let html = r#"<Script>test</Script>"#;
        let result = inject_nonce(html, "xyz");
        assert!(result.contains(r#"<Script nonce="xyz">"#));
    }

    #[test]
    fn nonce_not_injected_when_no_scripts() {
        let html = "<html><body><p>Hello</p></body></html>";
        let result = inject_nonce(html, "abc");
        assert_eq!(result, html);
    }

    #[test]
    fn generated_nonce_is_unique() {
        let a = generate_nonce();
        let b = generate_nonce();
        assert_ne!(a, b);
        // Base64 of 16 bytes = 24 chars (with padding)
        assert_eq!(a.len(), 24);
    }
}
