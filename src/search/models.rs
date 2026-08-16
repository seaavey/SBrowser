use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SearchEngine {
    #[default]
    Brave,
}

impl std::fmt::Display for SearchEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchEngine::Brave => write!(f, "brave"),
        }
    }
}

impl std::str::FromStr for SearchEngine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "brave" => Ok(SearchEngine::Brave),
            _ => Err(format!("Unknown search engine: '{}'. Supported engine: brave", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub rank: usize,
    pub title: String,
    pub url: String,
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub query: String,
    #[serde(default)]
    pub engine: SearchEngine,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub scrape_content: bool,
    #[serde(default = "default_scrape_limit")]
    pub scrape_limit: usize,
    pub timeout_ms: Option<u64>,
}

fn default_limit() -> usize {
    10
}

fn default_scrape_limit() -> usize {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub query: String,
    pub engine: SearchEngine,
    pub total_results: usize,
    pub took_ms: u128,
    pub results: Vec<SearchResultItem>,
}
