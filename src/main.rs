pub mod api;
pub mod config;
pub mod error;
pub mod lightpanda;
pub mod redis;
pub mod search;

use std::net::SocketAddr;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    lightpanda::LightpandaClient,
    redis::RedisService,
    search::SearchService,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sbrowser=info,tower_http=info,axum=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::default();
    run_server(config).await
}

async fn run_server(config: Config) -> anyhow::Result<()> {
    let client = LightpandaClient::new(
        config.lightpanda_path.clone(),
        config.default_timeout_ms,
        config.http_proxy.clone(),
        config.max_concurrent_fetches,
    );

    // Verify Lightpanda availability
    match client.check_installed().await {
        Ok(info_str) => {
            info!("Lightpanda detected at {:?}: {}", config.lightpanda_path, info_str);
        }
        Err(e) => {
            warn!(
                "Warning: Lightpanda check failed at {:?}: {}. Search/fetch calls may fail until Lightpanda is configured properly.",
                config.lightpanda_path, e
            );
        }
    }

    // Connect to Redis if configured
    let redis_service = if let Some(ref url) = config.redis_url {
        match RedisService::connect(url).await {
            Ok(service) => {
                info!("Redis connected at: {}", url);
                Some(service)
            }
            Err(e) => {
                warn!(
                    url = %url,
                    error = %e,
                    "Failed to connect to Redis. Running without rate limiting & caching."
                );
                None
            }
        }
    } else {
        info!("Redis disabled (set REDIS_URL to enable Rate Limiting and Caching)");
        None
    };

    let search_service = SearchService::new(client);
    let app = api::create_router(search_service, config.clone(), redis_service.clone());

    let addr_str = format!("{}:{}", config.host, config.port);
    let addr: SocketAddr = addr_str.parse()?;

    let redis_status_msg = match &redis_service {
        Some(r) => format!("Enabled ({}) [Rate Limit: {} req/{}s, Caching: {}]", r.url(), config.rate_limit_requests, config.rate_limit_window_secs, config.cache_enabled),
        None => "Disabled (Set REDIS_URL to enable anti-DDoS rate limiting & caching)".to_string(),
    };

    info!("=======================================================");
    info!("  SBrowser Search & Scraping REST API");
    info!("  Running on:   http://{}", addr);
    info!("  Redis Shield: {}", redis_status_msg);
    info!("  Search API:   GET /api/v1/search?q=...");
    info!("  Fetch API:    GET /api/v1/fetch?url=...");
    info!("  Health:       GET /health");
    info!("=======================================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
