use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

/// Add security headers to all responses.
pub async fn security_headers(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Cache control: immutable hashed assets get long cache, HTML gets no-cache
    if path.starts_with("/_app/immutable/") {
        headers.insert(
            "Cache-Control",
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    } else if !path.starts_with("/api") {
        // HTML pages and other static assets: always revalidate
        headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
    }

    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    // Allow framing only from Tauri desktop shell (tauri://localhost).
    // X-Frame-Options doesn't support custom schemes, so we omit it and
    // rely on the CSP frame-ancestors directive (supported by all modern browsers).
    headers.remove("X-Frame-Options");
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("camera=(self), microphone=(self), geolocation=()"),
    );
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval' blob:; \
             style-src 'self' 'unsafe-inline'; \
             connect-src 'self' wss: https://cdn.jsdelivr.net; \
             img-src 'self' data: blob: https://media0.giphy.com https://media1.giphy.com https://media2.giphy.com https://media3.giphy.com https://media4.giphy.com; \
             media-src 'self' blob:; \
             worker-src 'self' blob:; \
             frame-ancestors 'self' tauri://localhost; \
             base-uri 'self'; \
             form-action 'self'; \
             object-src 'none'",
        ),
    );

    response
}
