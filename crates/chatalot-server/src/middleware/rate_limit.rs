use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::Instant;

use axum::extract::{ConnectInfo, Request};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tokio::sync::Mutex;

/// Simple token-bucket rate limiter per IP address.
pub struct RateLimiter {
    buckets: Mutex<HashMap<IpAddr, Bucket>>,
    max_tokens: u32,
    refill_rate: f64, // tokens per second
    last_eviction: Mutex<Instant>,
}

struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

/// Evict stale buckets every 5 minutes
const EVICTION_INTERVAL_SECS: u64 = 300;
/// Remove buckets idle for more than 10 minutes
const BUCKET_TTL_SECS: u64 = 600;

impl RateLimiter {
    pub fn new(max_requests_per_second: u32, burst: u32) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            max_tokens: burst,
            refill_rate: max_requests_per_second as f64,
            last_eviction: Mutex::new(Instant::now()),
        }
    }

    async fn check(&self, ip: IpAddr) -> bool {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();

        // Periodically evict stale buckets to prevent unbounded growth
        let mut last_eviction = self.last_eviction.lock().await;
        if now.duration_since(*last_eviction).as_secs() >= EVICTION_INTERVAL_SECS {
            buckets.retain(|_, b| now.duration_since(b.last_refill).as_secs() < BUCKET_TTL_SECS);
            *last_eviction = now;
        }
        drop(last_eviction);

        let bucket = buckets.entry(ip).or_insert(Bucket {
            tokens: self.max_tokens as f64,
            last_refill: now,
        });

        // Refill tokens based on elapsed time
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_rate).min(self.max_tokens as f64);
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Check if an IP is a trusted reverse proxy (loopback or Docker bridge).
fn is_trusted_proxy(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.octets()[..2] == [172, 17] // Docker default bridge
                || v4.octets()[..2] == [172, 18] // Docker additional networks
                || v4.octets()[..3] == [172, 19, 0] // Docker compose
        }
        IpAddr::V6(v6) => v6.is_loopback(),
    }
}

/// Extract the real client IP. Trusts proxy headers only from trusted sources.
fn extract_client_ip(request: &Request) -> IpAddr {
    let conn_ip = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip());

    // Only trust proxy headers when the direct connection is from a trusted proxy
    if let Some(peer) = conn_ip {
        if is_trusted_proxy(peer)
            && let Some(ip) = request
                .headers()
                .get("cf-connecting-ip")
                .or_else(|| request.headers().get("x-forwarded-for"))
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .and_then(|s| s.trim().parse::<IpAddr>().ok())
        {
            return ip;
        }
        return peer;
    }

    IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
}

/// Rate limiting middleware.
pub async fn rate_limit_middleware(request: Request, next: Next) -> Response {
    let ip = extract_client_ip(&request);

    // Use a lazily initialized static rate limiter
    static LIMITER: std::sync::LazyLock<RateLimiter> =
        std::sync::LazyLock::new(|| RateLimiter::new(20, 50));

    if LIMITER.check(ip).await {
        next.run(request).await
    } else {
        let body = json!({
            "error": {
                "code": "rate_limited",
                "message": "too many requests, please slow down"
            }
        });
        (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, "1")], axum::Json(body))
            .into_response()
    }
}

/// Stricter rate limiter for auth endpoints (login/register).
pub async fn auth_rate_limit_middleware(request: Request, next: Next) -> Response {
    let ip = extract_client_ip(&request);

    // 5 auth attempts per second, burst of 10
    static AUTH_LIMITER: std::sync::LazyLock<RateLimiter> =
        std::sync::LazyLock::new(|| RateLimiter::new(5, 10));

    if AUTH_LIMITER.check(ip).await {
        next.run(request).await
    } else {
        let body = json!({
            "error": {
                "code": "rate_limited",
                "message": "too many authentication attempts"
            }
        });
        (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, "5")], axum::Json(body))
            .into_response()
    }
}
