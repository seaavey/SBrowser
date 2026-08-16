use std::{sync::Arc, time::Instant};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use tracing::warn;

use crate::{
    api::models::{
        EngineInfo, FetchQueryParams, FetchRequestBody, HealthResponse, SearchQueryParams,
        SearchRequestBody,
    },
    config::Config,
    error::AppError,
    lightpanda::{DumpFormat, FetchOptions},
    redis::RedisService,
    search::{
        models::{SearchEngine, SearchOptions, SearchResponse},
        SearchService,
    },
};

pub struct AppState {
    pub search_service: SearchService,
    pub config: Config,
    pub redis_service: Option<RedisService>,
    pub start_time: Instant,
}

pub type SharedState = Arc<AppState>;

/// Rate Limiting & Anti-DDoS Middleware via Redis
pub async fn rate_limit_middleware(
    State(state): State<SharedState>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if !state.config.rate_limit_enabled {
        return next.run(request).await;
    }

    if let Some(ref redis) = state.redis_service {
        let ip = extract_client_ip(&request);
        let result = redis
            .check_rate_limit(
                &ip,
                state.config.rate_limit_requests,
                state.config.rate_limit_window_secs,
            )
            .await;

        if !result.allowed {
            warn!(
                ip = %ip,
                limit = result.limit,
                reset_secs = result.reset_secs,
                "Rate limit exceeded (HTTP 429)"
            );

            let body = serde_json::json!({
                "error": {
                    "message": "Rate limit exceeded. Too many requests, please slow down.",
                    "status": 429,
                    "retry_after_secs": result.reset_secs
                }
            });

            let mut response = (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                Json(body),
            ).into_response();

            let headers = response.headers_mut();
            headers.insert("X-RateLimit-Limit", axum::http::HeaderValue::from(result.limit));
            headers.insert("X-RateLimit-Remaining", axum::http::HeaderValue::from(result.remaining));
            headers.insert("X-RateLimit-Reset", axum::http::HeaderValue::from(result.reset_secs));
            headers.insert("Retry-After", axum::http::HeaderValue::from(result.reset_secs));

            return response;
        }

        let mut response = next.run(request).await;
        let headers = response.headers_mut();
        headers.insert("X-RateLimit-Limit", axum::http::HeaderValue::from(result.limit));
        headers.insert("X-RateLimit-Remaining", axum::http::HeaderValue::from(result.remaining));
        headers.insert("X-RateLimit-Reset", axum::http::HeaderValue::from(result.reset_secs));
        response
    } else {
        next.run(request).await
    }
}

fn extract_client_ip(req: &axum::extract::Request) -> String {
    // 1. X-Forwarded-For
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                let trimmed = first_ip.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    // 2. X-Real-IP
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            let trimmed = ip_str.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    // 3. CF-Connecting-IP
    if let Some(cf_ip) = req.headers().get("cf-connecting-ip") {
        if let Ok(ip_str) = cf_ip.to_str() {
            let trimmed = ip_str.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

pub async fn handle_health(State(state): State<SharedState>) -> Result<Json<HealthResponse>, AppError> {
    let client = state.search_service.client();
    let status_str = match client.check_installed().await {
        Ok(v) => v,
        Err(e) => format!("Error: {}", e),
    };

    let redis_status = match &state.redis_service {
        Some(redis) => match redis.ping().await {
            Ok(_) => format!("connected ({})", redis.url()),
            Err(e) => format!("error: {}", e),
        },
        None => "disabled (REDIS_URL not configured)".to_string(),
    };

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        lightpanda_path: client.binary_path().to_string_lossy().to_string(),
        lightpanda_status: status_str,
        redis_status,
        uptime_secs: state.start_time.elapsed().as_secs(),
    }))
}

pub async fn handle_engines() -> Json<Vec<EngineInfo>> {
    Json(vec![
        EngineInfo {
            id: "brave".to_string(),
            name: "Brave Search".to_string(),
            default: true,
            description: "Independent index web search powered by Brave".to_string(),
        },
    ])
}

pub async fn handle_search_get(
    State(state): State<SharedState>,
    Query(params): Query<SearchQueryParams>,
) -> Result<Json<SearchResponse>, AppError> {
    let options = SearchOptions {
        query: params.q,
        engine: params.engine.unwrap_or(SearchEngine::Brave),
        limit: params.limit.unwrap_or(10),
        scrape_content: params.scrape.unwrap_or(false),
        scrape_limit: params.scrape_limit.unwrap_or(3),
        timeout_ms: params.timeout_ms,
    };

    let cache_key = format!(
        "{}:{}:{}:{}",
        options.query.trim().to_lowercase(),
        options.limit,
        options.scrape_content,
        options.scrape_limit
    );

    // 1. Check Redis Cache
    if state.config.cache_enabled {
        if let Some(ref redis) = state.redis_service {
            if let Some(cached) = redis.get_cached_search(&cache_key).await {
                return Ok(Json(cached));
            }
        }
    }

    // 2. Execute Search
    let response = state.search_service.search(options).await?;

    // 3. Save to Redis Cache
    if state.config.cache_enabled {
        if let Some(ref redis) = state.redis_service {
            redis
                .set_cached_search(
                    &cache_key,
                    &response,
                    state.config.cache_search_ttl_secs,
                )
                .await;
        }
    }

    Ok(Json(response))
}

pub async fn handle_search_post(
    State(state): State<SharedState>,
    Json(body): Json<SearchRequestBody>,
) -> Result<Json<SearchResponse>, AppError> {
    let options = SearchOptions {
        query: body.query,
        engine: body.engine.unwrap_or(SearchEngine::Brave),
        limit: body.limit.unwrap_or(10),
        scrape_content: body.scrape_content.unwrap_or(false),
        scrape_limit: body.scrape_limit.unwrap_or(3),
        timeout_ms: body.timeout_ms,
    };

    let cache_key = format!(
        "{}:{}:{}:{}",
        options.query.trim().to_lowercase(),
        options.limit,
        options.scrape_content,
        options.scrape_limit
    );

    // 1. Check Redis Cache
    if state.config.cache_enabled {
        if let Some(ref redis) = state.redis_service {
            if let Some(cached) = redis.get_cached_search(&cache_key).await {
                return Ok(Json(cached));
            }
        }
    }

    // 2. Execute Search
    let response = state.search_service.search(options).await?;

    // 3. Save to Redis Cache
    if state.config.cache_enabled {
        if let Some(ref redis) = state.redis_service {
            redis
                .set_cached_search(
                    &cache_key,
                    &response,
                    state.config.cache_search_ttl_secs,
                )
                .await;
        }
    }

    Ok(Json(response))
}

pub async fn handle_fetch_get(
    State(state): State<SharedState>,
    Query(params): Query<FetchQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    let client = state.search_service.client();
    let format = params.format.unwrap_or(DumpFormat::Markdown);
    let options = FetchOptions {
        format,
        wait_ms: params.wait_ms.or(Some(3000)),
        wait_selector: params.wait_selector,
        timeout_ms: params.timeout_ms,
        ..Default::default()
    };

    let cache_key = format!(
        "{}:{}:{:?}:{:?}",
        params.url.trim(),
        format,
        options.wait_ms,
        options.wait_selector
    );

    // 1. Check Redis Cache
    if state.config.cache_enabled {
        if let Some(ref redis) = state.redis_service {
            if let Some(cached) = redis.get_cached_fetch(&cache_key).await {
                return Ok(Json(cached));
            }
        }
    }

    // 2. Execute Fetch
    let result = client.fetch(&params.url, &options).await?;

    // 3. Save to Redis Cache
    if state.config.cache_enabled {
        if let Some(ref redis) = state.redis_service {
            redis
                .set_cached_fetch(
                    &cache_key,
                    &result,
                    state.config.cache_fetch_ttl_secs,
                )
                .await;
        }
    }

    Ok(Json(result))
}

pub async fn handle_fetch_post(
    State(state): State<SharedState>,
    Json(body): Json<FetchRequestBody>,
) -> Result<impl IntoResponse, AppError> {
    let client = state.search_service.client();
    let format = body.format.unwrap_or(DumpFormat::Markdown);
    let options = FetchOptions {
        format,
        wait_until: body.wait_until,
        wait_ms: body.wait_ms.or(Some(3000)),
        wait_selector: body.wait_selector,
        timeout_ms: body.timeout_ms,
        with_base: body.with_base.unwrap_or(false),
        with_frames: body.with_frames.unwrap_or(false),
        insecure_disable_tls: false,
    };

    let cache_key = format!(
        "{}:{}:{:?}:{:?}",
        body.url.trim(),
        format,
        options.wait_ms,
        options.wait_selector
    );

    // 1. Check Redis Cache
    if state.config.cache_enabled {
        if let Some(ref redis) = state.redis_service {
            if let Some(cached) = redis.get_cached_fetch(&cache_key).await {
                return Ok(Json(cached));
            }
        }
    }

    // 2. Execute Fetch
    let result = client.fetch(&body.url, &options).await?;

    // 3. Save to Redis Cache
    if state.config.cache_enabled {
        if let Some(ref redis) = state.redis_service {
            redis
                .set_cached_fetch(
                    &cache_key,
                    &result,
                    state.config.cache_fetch_ttl_secs,
                )
                .await;
        }
    }

    Ok(Json(result))
}
