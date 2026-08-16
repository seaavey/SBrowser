use serde::{Deserialize, Serialize};

use crate::{
    lightpanda::{DumpFormat, WaitUntil},
    search::models::SearchEngine,
};

#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    pub q: String,
    pub engine: Option<SearchEngine>,
    pub limit: Option<usize>,
    pub scrape: Option<bool>,
    pub scrape_limit: Option<usize>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequestBody {
    pub query: String,
    pub engine: Option<SearchEngine>,
    pub limit: Option<usize>,
    pub scrape_content: Option<bool>,
    pub scrape_limit: Option<usize>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct FetchQueryParams {
    pub url: String,
    pub format: Option<DumpFormat>,
    pub wait_ms: Option<u64>,
    pub wait_selector: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct FetchRequestBody {
    pub url: String,
    pub format: Option<DumpFormat>,
    pub wait_until: Option<WaitUntil>,
    pub wait_ms: Option<u64>,
    pub wait_selector: Option<String>,
    pub timeout_ms: Option<u64>,
    pub with_base: Option<bool>,
    pub with_frames: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct EngineInfo {
    pub id: String,
    pub name: String,
    pub default: bool,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub lightpanda_path: String,
    pub lightpanda_status: String,
    pub redis_status: String,
    pub uptime_secs: u64,
}
