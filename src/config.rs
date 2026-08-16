use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub lightpanda_path: PathBuf,
    pub default_timeout_ms: u64,
    pub http_proxy: Option<String>,
    pub max_concurrent_fetches: usize,

    // Redis & Anti-DDoS / Rate Limit / Cache configurations
    pub redis_url: Option<String>,
    pub rate_limit_enabled: bool,
    pub rate_limit_requests: u64,
    pub rate_limit_window_secs: u64,
    pub cache_enabled: bool,
    pub cache_search_ttl_secs: u64,
    pub cache_fetch_ttl_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        let lightpanda_path = detect_lightpanda_path();
        let redis_url = std::env::var("REDIS_URL")
            .or_else(|_| std::env::var("SBROWSER_REDIS_URL"))
            .ok()
            .filter(|s| !s.trim().is_empty());

        Self {
            host: std::env::var("SBROWSER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("SBROWSER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            lightpanda_path,
            default_timeout_ms: std::env::var("SBROWSER_TIMEOUT_MS")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(15000),
            http_proxy: std::env::var("HTTP_PROXY").or_else(|_| std::env::var("http_proxy")).ok(),
            max_concurrent_fetches: std::env::var("SBROWSER_MAX_CONCURRENT")
                .ok()
                .and_then(|m| m.parse().ok())
                .unwrap_or(8),

            redis_url,
            rate_limit_enabled: std::env::var("RATE_LIMIT_ENABLED")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            rate_limit_requests: std::env::var("RATE_LIMIT_REQUESTS")
                .or_else(|_| std::env::var("SBROWSER_RATE_LIMIT_REQUESTS"))
                .ok()
                .and_then(|r| r.parse().ok())
                .unwrap_or(60),
            rate_limit_window_secs: std::env::var("RATE_LIMIT_WINDOW_SECS")
                .or_else(|_| std::env::var("SBROWSER_RATE_LIMIT_WINDOW_SECS"))
                .ok()
                .and_then(|w| w.parse().ok())
                .unwrap_or(60),
            cache_enabled: std::env::var("CACHE_ENABLED")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            cache_search_ttl_secs: std::env::var("CACHE_SEARCH_TTL_SECS")
                .or_else(|_| std::env::var("SBROWSER_CACHE_SEARCH_TTL_SECS"))
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(600), // 10 minutes
            cache_fetch_ttl_secs: std::env::var("CACHE_FETCH_TTL_SECS")
                .or_else(|_| std::env::var("SBROWSER_CACHE_FETCH_TTL_SECS"))
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(3600), // 1 hour
        }
    }
}

pub fn detect_lightpanda_path() -> PathBuf {
    // 1. Check env var
    if let Ok(env_path) = std::env::var("LIGHTPANDA_PATH") {
        let path = PathBuf::from(env_path);
        if path.exists() {
            return path;
        }
    }

    // 2. Check ./bin/lightpanda relative to current directory
    let local_bin = Path::new("./bin/lightpanda");
    if local_bin.exists() {
        if let Ok(canon) = local_bin.canonicalize() {
            return canon;
        }
        return local_bin.to_path_buf();
    }

    // 3. Check cargo workspace root /bin/lightpanda
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bin/lightpanda");
    if manifest_dir.exists() {
        return manifest_dir;
    }

    // 4. Fallback to system PATH "lightpanda"
    PathBuf::from("lightpanda")
}
