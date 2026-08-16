use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DumpFormat {
    #[default]
    Markdown,
    Html,
}

impl std::str::FromStr for DumpFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(DumpFormat::Markdown),
            "html" | "htm" => Ok(DumpFormat::Html),
            _ => Err(format!("Invalid format: {}", s)),
        }
    }
}


impl std::fmt::Display for DumpFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DumpFormat::Markdown => write!(f, "markdown"),
            DumpFormat::Html => write!(f, "html"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WaitUntil {
    Load,
    DomContentLoaded,
    NetworkAlmostIdle,
    NetworkIdle,
    Done,
}

impl std::fmt::Display for WaitUntil {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaitUntil::Load => write!(f, "load"),
            WaitUntil::DomContentLoaded => write!(f, "domcontentloaded"),
            WaitUntil::NetworkAlmostIdle => write!(f, "networkalmostidle"),
            WaitUntil::NetworkIdle => write!(f, "networkidle"),
            WaitUntil::Done => write!(f, "done"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOptions {
    #[serde(default)]
    pub format: DumpFormat,
    pub wait_until: Option<WaitUntil>,
    pub wait_ms: Option<u64>,
    pub wait_selector: Option<String>,
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub with_base: bool,
    #[serde(default)]
    pub with_frames: bool,
    #[serde(default)]
    pub insecure_disable_tls: bool,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            format: DumpFormat::Markdown,
            wait_until: Some(WaitUntil::Load),
            wait_ms: Some(5000),
            wait_selector: None,
            timeout_ms: Some(15000),
            with_base: false,
            with_frames: false,
            insecure_disable_tls: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResult {
    pub url: String,
    pub format: DumpFormat,
    pub content: String,
    pub length: usize,
    pub took_ms: u128,
}
