use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use tracing::{info, warn};

use crate::lightpanda::FetchResult;
use crate::search::models::SearchResponse;

#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub limit: u64,
    pub remaining: u64,
    pub reset_secs: u64,
}

#[derive(Clone)]
pub struct RedisService {
    connection_manager: ConnectionManager,
    redis_url: String,
}

impl RedisService {
    /// Attempts to connect to Redis and returns a RedisService instance
    pub async fn connect(url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(url)?;
        let connection_manager = ConnectionManager::new(client).await?;
        info!(url = %url, "Successfully connected to Redis");
        Ok(Self {
            connection_manager,
            redis_url: url.to_string(),
        })
    }

    pub fn url(&self) -> &str {
        &self.redis_url
    }

    /// Check health/ping
    pub async fn ping(&self) -> Result<(), redis::RedisError> {
        let mut conn = self.connection_manager.clone();
        let _: () = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(())
    }

    /// Anti-DDoS / Rate Limiter using atomic INCR + EXPIRE
    pub async fn check_rate_limit(&self, ip: &str, limit: u64, window_secs: u64) -> RateLimitResult {
        let key = format!("sbrowser:ratelimit:{}", ip);
        let mut conn = self.connection_manager.clone();

        // Use Lua script for atomic sliding/fixed window rate limiting
        let script = redis::Script::new(r#"
            local current = redis.call('INCR', KEYS[1])
            if current == 1 then
                redis.call('EXPIRE', KEYS[1], ARGV[1])
            end
            local ttl = redis.call('TTL', KEYS[1])
            return {current, ttl}
        "#);

        match script
            .key(&key)
            .arg(window_secs)
            .invoke_async::<Vec<i64>>(&mut conn)
            .await
        {
            Ok(values) if values.len() >= 2 => {
                let current = values[0].max(0) as u64;
                let ttl = values[1].max(1) as u64;
                let allowed = current <= limit;
                let remaining = if allowed { limit.saturating_sub(current) } else { 0 };

                RateLimitResult {
                    allowed,
                    limit,
                    remaining,
                    reset_secs: ttl,
                }
            }
            Ok(_) => {
                // Fallback allow if script returns unexpected shape
                RateLimitResult {
                    allowed: true,
                    limit,
                    remaining: limit,
                    reset_secs: window_secs,
                }
            }
            Err(e) => {
                warn!(error = %e, ip = %ip, "Redis rate limiting check failed, allowing request gracefully");
                RateLimitResult {
                    allowed: true,
                    limit,
                    remaining: limit,
                    reset_secs: window_secs,
                }
            }
        }
    }

    /// Retrieve cached search results from Redis
    pub async fn get_cached_search(&self, key: &str) -> Option<SearchResponse> {
        let mut conn = self.connection_manager.clone();
        let full_key = format!("sbrowser:cache:search:{}", key);

        match conn.get::<_, Option<String>>(&full_key).await {
            Ok(Some(json_str)) => {
                match serde_json::from_str::<SearchResponse>(&json_str) {
                    Ok(res) => {
                        info!(key = %full_key, "Search cache HIT from Redis");
                        Some(res)
                    }
                    Err(e) => {
                        warn!(key = %full_key, error = %e, "Corrupted cached search JSON in Redis");
                        None
                    }
                }
            }
            Ok(None) => None,
            Err(e) => {
                warn!(key = %full_key, error = %e, "Failed to read search cache from Redis");
                None
            }
        }
    }

    /// Store search results into Redis with TTL
    pub async fn set_cached_search(&self, key: &str, data: &SearchResponse, ttl_secs: u64) {
        let mut conn = self.connection_manager.clone();
        let full_key = format!("sbrowser:cache:search:{}", key);

        if let Ok(json_str) = serde_json::to_string(data) {
            let res: Result<(), _> = conn.set_ex(&full_key, json_str, ttl_secs).await;
            if let Err(e) = res {
                warn!(key = %full_key, error = %e, "Failed to store search cache in Redis");
            }
        }
    }

    /// Retrieve cached fetch page results from Redis
    pub async fn get_cached_fetch(&self, key: &str) -> Option<FetchResult> {
        let mut conn = self.connection_manager.clone();
        let full_key = format!("sbrowser:cache:fetch:{}", key);

        match conn.get::<_, Option<String>>(&full_key).await {
            Ok(Some(json_str)) => {
                match serde_json::from_str::<FetchResult>(&json_str) {
                    Ok(res) => {
                        info!(key = %full_key, "Fetch page cache HIT from Redis");
                        Some(res)
                    }
                    Err(e) => {
                        warn!(key = %full_key, error = %e, "Corrupted cached fetch JSON in Redis");
                        None
                    }
                }
            }
            Ok(None) => None,
            Err(e) => {
                warn!(key = %full_key, error = %e, "Failed to read fetch cache from Redis");
                None
            }
        }
    }

    /// Store fetch page results into Redis with TTL
    pub async fn set_cached_fetch(&self, key: &str, data: &FetchResult, ttl_secs: u64) {
        let mut conn = self.connection_manager.clone();
        let full_key = format!("sbrowser:cache:fetch:{}", key);

        if let Ok(json_str) = serde_json::to_string(data) {
            let res: Result<(), _> = conn.set_ex(&full_key, json_str, ttl_secs).await;
            if let Err(e) = res {
                warn!(key = %full_key, error = %e, "Failed to store fetch cache in Redis");
            }
        }
    }
}
