use std::sync::LazyLock;
use std::time::{Duration, Instant};

use axum::extract::Query;
use axum::routing::get;
use axum::{Extension, Json, Router};
use dashmap::DashMap;
use std::sync::Arc;

use chatalot_common::api_types::{GifResult, GifSearchQuery, GifSearchResponse};

use crate::app_state::AppState;
use crate::error::AppError;
use crate::middleware::auth::AccessClaims;

struct CacheEntry {
    data: GifSearchResponse,
    created: Instant,
}

static GIF_CACHE: LazyLock<DashMap<String, CacheEntry>> = LazyLock::new(DashMap::new);
const CACHE_TTL: Duration = Duration::from_secs(300);
const MAX_CACHE_SIZE: usize = 200;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/gifs/search", get(search_gifs))
        .route("/gifs/trending", get(trending_gifs))
}

fn get_tenor_key() -> Option<String> {
    std::env::var("TENOR_API_KEY").ok().filter(|k| !k.is_empty())
}

async fn search_gifs(
    Extension(_claims): Extension<AccessClaims>,
    Query(query): Query<GifSearchQuery>,
) -> Result<Json<GifSearchResponse>, AppError> {
    let api_key = get_tenor_key()
        .ok_or_else(|| AppError::Validation("GIF search is not configured on this server".into()))?;

    let q = query.q.unwrap_or_default();
    if q.is_empty() {
        return Ok(Json(GifSearchResponse { results: vec![], next: None }));
    }
    let limit = query.limit.unwrap_or(20).min(50);
    let cache_key = format!("search:{}:{}:{}", q, limit, query.pos.as_deref().unwrap_or(""));

    if let Some(entry) = GIF_CACHE.get(&cache_key) {
        if entry.created.elapsed() < CACHE_TTL {
            return Ok(Json(GifSearchResponse {
                results: entry.data.results.clone(),
                next: entry.data.next.clone(),
            }));
        }
        drop(entry);
        GIF_CACHE.remove(&cache_key);
    }

    let mut params: Vec<(&str, String)> = vec![
        ("q", q),
        ("key", api_key),
        ("client_key", "chatalot".into()),
        ("limit", limit.to_string()),
        ("media_filter", "gif,tinygif".into()),
    ];
    if let Some(ref pos) = query.pos {
        params.push(("pos", pos.clone()));
    }

    let response = fetch_tenor("https://tenor.googleapis.com/v2/search", &params).await?;
    cache_gif_response(&cache_key, &response);
    Ok(Json(response))
}

async fn trending_gifs(
    Extension(_claims): Extension<AccessClaims>,
    Query(query): Query<GifSearchQuery>,
) -> Result<Json<GifSearchResponse>, AppError> {
    let api_key = get_tenor_key()
        .ok_or_else(|| AppError::Validation("GIF search is not configured on this server".into()))?;

    let limit = query.limit.unwrap_or(20).min(50);
    let cache_key = format!("trending:{}:{}", limit, query.pos.as_deref().unwrap_or(""));

    if let Some(entry) = GIF_CACHE.get(&cache_key) {
        if entry.created.elapsed() < CACHE_TTL {
            return Ok(Json(GifSearchResponse {
                results: entry.data.results.clone(),
                next: entry.data.next.clone(),
            }));
        }
        drop(entry);
        GIF_CACHE.remove(&cache_key);
    }

    let mut params: Vec<(&str, String)> = vec![
        ("key", api_key),
        ("client_key", "chatalot".into()),
        ("limit", limit.to_string()),
        ("media_filter", "gif,tinygif".into()),
    ];
    if let Some(ref pos) = query.pos {
        params.push(("pos", pos.clone()));
    }

    let response = fetch_tenor("https://tenor.googleapis.com/v2/featured", &params).await?;
    cache_gif_response(&cache_key, &response);
    Ok(Json(response))
}

async fn fetch_tenor(base_url: &str, params: &[(&str, String)]) -> Result<GifSearchResponse, AppError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| AppError::Internal(format!("HTTP client error: {e}")))?;

    let resp = client.get(base_url).query(params).send().await.map_err(|e| {
        tracing::warn!("Tenor API request failed: {e}");
        AppError::Internal("Failed to fetch GIFs".into())
    })?;

    if !resp.status().is_success() {
        let status = resp.status();
        tracing::warn!("Tenor API returned status {status}");
        return Err(AppError::Internal(format!("Tenor API error: {status}")));
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| {
        AppError::Internal(format!("Failed to parse Tenor response: {e}"))
    })?;

    let results = body["results"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|item| {
            let id = item["id"].as_str()?.to_string();
            let title = item["title"].as_str().unwrap_or("").to_string();
            let gif = &item["media_formats"]["gif"];
            let tinygif = &item["media_formats"]["tinygif"];
            let url = gif["url"].as_str()?.to_string();
            let preview_url = tinygif["url"].as_str().unwrap_or(&url).to_string();
            let dims = gif["dims"].as_array()?;
            let width = dims.first()?.as_u64()? as u32;
            let height = dims.get(1)?.as_u64()? as u32;
            Some(GifResult { id, title, preview_url, url, width, height })
        })
        .collect();

    let next = body["next"].as_str().map(|s| s.to_string()).filter(|s| !s.is_empty());

    Ok(GifSearchResponse { results, next })
}

fn cache_gif_response(key: &str, response: &GifSearchResponse) {
    if GIF_CACHE.len() >= MAX_CACHE_SIZE {
        let to_remove: Vec<String> = GIF_CACHE
            .iter()
            .filter(|e| e.created.elapsed() > CACHE_TTL)
            .map(|e| e.key().clone())
            .take(MAX_CACHE_SIZE / 4)
            .collect();
        for k in to_remove {
            GIF_CACHE.remove(&k);
        }
    }
    GIF_CACHE.insert(
        key.to_string(),
        CacheEntry {
            data: GifSearchResponse {
                results: response.results.clone(),
                next: response.next.clone(),
            },
            created: Instant::now(),
        },
    );
}
