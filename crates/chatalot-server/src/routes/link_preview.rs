use std::sync::LazyLock;
use std::time::{Duration, Instant};

use axum::extract::Query;
use axum::routing::get;
use axum::{Extension, Json, Router};
use dashmap::DashMap;
use std::sync::Arc;

use chatalot_common::api_types::{LinkPreviewQuery, LinkPreviewResponse};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

struct CacheEntry {
    data: LinkPreviewResponse,
    created: Instant,
}

static PREVIEW_CACHE: LazyLock<DashMap<String, CacheEntry>> = LazyLock::new(DashMap::new);
const CACHE_TTL: Duration = Duration::from_secs(3600);
const MAX_CACHE_SIZE: usize = 1000;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/link-preview", get(get_link_preview))
}

async fn get_link_preview(
    Extension(_claims): Extension<AccessClaims>,
    Query(query): Query<LinkPreviewQuery>,
) -> Result<Json<LinkPreviewResponse>, AppError> {
    let url = query.url.trim().to_string();

    // Validate URL scheme
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(AppError::Validation("Only http/https URLs are allowed".into()));
    }

    // SSRF protection: block private/internal IPs
    if let Ok(parsed) = url::Url::parse(&url)
        && let Some(host) = parsed.host_str()
        && is_private_host(host)
    {
        return Err(AppError::Validation("Cannot preview internal URLs".into()));
    }

    // Check cache
    if let Some(entry) = PREVIEW_CACHE.get(&url) {
        if entry.created.elapsed() < CACHE_TTL {
            return Ok(Json(entry.data.clone()));
        }
        drop(entry);
        PREVIEW_CACHE.remove(&url);
    }

    // Fetch the URL
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::limited(3))
        .user_agent("ChatalotBot/1.0 (link preview)")
        .build()
        .map_err(|e| AppError::Internal(format!("HTTP client error: {e}")))?;

    let response = client.get(&url).send().await.map_err(|e| {
        tracing::debug!("Link preview fetch failed for {url}: {e}");
        AppError::Validation("Could not fetch URL".into())
    })?;

    // Only process HTML responses
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !content_type.contains("text/html") {
        let preview = LinkPreviewResponse {
            url: url.clone(),
            title: None,
            description: None,
            image: None,
            site_name: None,
        };
        cache_preview(&url, &preview);
        return Ok(Json(preview));
    }

    // Limit response body to 512KB
    let body = response
        .text()
        .await
        .map_err(|_| AppError::Validation("Could not read response".into()))?;
    let body = if body.len() > 512_000 {
        &body[..512_000]
    } else {
        &body
    };

    let preview = parse_og_metadata(body, &url);
    cache_preview(&url, &preview);

    Ok(Json(preview))
}

fn cache_preview(url: &str, preview: &LinkPreviewResponse) {
    if PREVIEW_CACHE.len() >= MAX_CACHE_SIZE {
        // Evict expired entries
        let to_remove: Vec<String> = PREVIEW_CACHE
            .iter()
            .filter(|e| e.created.elapsed() > CACHE_TTL)
            .map(|e| e.key().clone())
            .take(MAX_CACHE_SIZE / 10)
            .collect();
        for key in to_remove {
            PREVIEW_CACHE.remove(&key);
        }
    }
    PREVIEW_CACHE.insert(
        url.to_string(),
        CacheEntry {
            data: preview.clone(),
            created: Instant::now(),
        },
    );
}

fn parse_og_metadata(html: &str, url: &str) -> LinkPreviewResponse {
    let mut title = None;
    let mut description = None;
    let mut image = None;
    let mut site_name = None;

    // Match <meta property="og:..." content="..."> (property before content)
    let meta_re = regex::Regex::new(
        r#"<meta\s+[^>]*?(?:property|name)=["']([^"']+)["'][^>]*?content=["']([^"']*)["']"#,
    )
    .unwrap();

    // Match <meta content="..." property="og:..."> (content before property)
    let meta_re2 = regex::Regex::new(
        r#"<meta\s+[^>]*?content=["']([^"']*)["'][^>]*?(?:property|name)=["']([^"']+)["']"#,
    )
    .unwrap();

    for cap in meta_re.captures_iter(html) {
        let prop = cap[1].to_lowercase();
        let content = html_escape::decode_html_entities(&cap[2]).to_string();
        match prop.as_str() {
            "og:title" => title = Some(content),
            "og:description" => description = Some(content),
            "description" if description.is_none() => description = Some(content),
            "og:image" => image = Some(content),
            "og:site_name" => site_name = Some(content),
            _ => {}
        }
    }

    for cap in meta_re2.captures_iter(html) {
        let content = html_escape::decode_html_entities(&cap[1]).to_string();
        let prop = cap[2].to_lowercase();
        match prop.as_str() {
            "og:title" if title.is_none() => title = Some(content),
            "og:description" if description.is_none() => description = Some(content),
            "description" if description.is_none() => description = Some(content),
            "og:image" if image.is_none() => image = Some(content),
            "og:site_name" if site_name.is_none() => site_name = Some(content),
            _ => {}
        }
    }

    // Fallback: extract <title> tag
    if title.is_none() {
        let title_re = regex::Regex::new(r"<title[^>]*>([^<]+)</title>").unwrap();
        if let Some(cap) = title_re.captures(html) {
            title = Some(html_escape::decode_html_entities(&cap[1]).to_string());
        }
    }

    // Make relative image URLs absolute
    if let Some(ref img) = image {
        if img.starts_with("//") {
            image = Some(format!("https:{img}"));
        } else if img.starts_with('/')
            && let Ok(parsed) = url::Url::parse(url)
        {
            image = Some(format!(
                "{}://{}{}",
                parsed.scheme(),
                parsed.host_str().unwrap_or(""),
                img
            ));
        }
    }

    LinkPreviewResponse {
        url: url.to_string(),
        title: title.map(|t| truncate_str(t, 200)),
        description: description.map(|d| truncate_str(d, 500)),
        image,
        site_name: site_name.map(|s| truncate_str(s, 100)),
    }
}

fn truncate_str(s: String, max: usize) -> String {
    if s.len() <= max {
        s
    } else {
        match s.char_indices().nth(max) {
            Some((idx, _)) => format!("{}...", &s[..idx]),
            None => s,
        }
    }
}

fn is_private_host(host: &str) -> bool {
    if host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "0.0.0.0" {
        return true;
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        match ip {
            std::net::IpAddr::V4(v4) => {
                return v4.is_private()
                    || v4.is_loopback()
                    || v4.is_link_local()
                    || v4.octets()[0] == 0;
            }
            std::net::IpAddr::V6(v6) => {
                return v6.is_loopback();
            }
        }
    }
    if host.ends_with(".local") || host.ends_with(".internal") {
        return true;
    }
    false
}
