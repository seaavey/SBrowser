use std::{
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
use tokio::{
    process::Command,
    sync::Semaphore,
    time::{timeout, Duration},
};
use tracing::{debug, error, info, warn};

use super::models::{DumpFormat, FetchOptions, FetchResult};
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct LightpandaClient {
    binary_path: PathBuf,
    default_timeout_ms: u64,
    proxy: Option<String>,
    semaphore: Arc<Semaphore>,
}

impl LightpandaClient {
    pub fn new(
        binary_path: PathBuf,
        default_timeout_ms: u64,
        proxy: Option<String>,
        max_concurrent: usize,
    ) -> Self {
        Self {
            binary_path,
            default_timeout_ms,
            proxy,
            semaphore: Arc::new(Semaphore::new(max_concurrent.max(1))),
        }
    }

    pub fn binary_path(&self) -> &PathBuf {
        &self.binary_path
    }

    pub async fn check_installed(&self) -> Result<String, AppError> {
        let output = Command::new(&self.binary_path)
            .arg("version")
            .output()
            .await
            .map_err(|e| {
                AppError::Lightpanda(format!(
                    "Failed to execute Lightpanda binary at '{:?}': {}",
                    self.binary_path, e
                ))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() && stdout.is_empty() {
            // Check help as fallback
            let help_output = Command::new(&self.binary_path)
                .arg("help")
                .output()
                .await
                .map_err(|e| AppError::Lightpanda(format!("Execution check error: {}", e)))?;

            if help_output.status.success() {
                return Ok("Lightpanda (installed and ready)".to_string());
            }

            return Err(AppError::Lightpanda(format!(
                "Lightpanda binary check failed (exit code {}): {}",
                output.status, stderr
            )));
        }

        if stdout.is_empty() {
            Ok("Lightpanda ready".to_string())
        } else {
            Ok(stdout)
        }
    }

    pub async fn fetch(&self, url: &str, options: &FetchOptions) -> Result<FetchResult, AppError> {
        let start_time = Instant::now();
        let timeout_duration = Duration::from_millis(
            options
                .timeout_ms
                .unwrap_or(self.default_timeout_ms)
                .max(1000),
        );

        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Semaphore error: {}", e)))?;

        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("fetch");

        // Dump format
        match options.format {
            DumpFormat::Markdown => {
                cmd.args(["--dump", "markdown"]);
            }
            DumpFormat::Html => {
                cmd.args(["--dump", "html"]);
            }
        }

        // Wait conditions
        if let Some(ref wait_until) = options.wait_until {
            cmd.args(["--wait-until", &wait_until.to_string()]);
        }
        if let Some(wait_ms) = options.wait_ms {
            cmd.args(["--wait-ms", &wait_ms.to_string()]);
        }
        if let Some(ref selector) = options.wait_selector {
            cmd.args(["--wait-selector", selector]);
        }

        if options.with_base {
            cmd.arg("--with-base");
        }
        if options.with_frames {
            cmd.arg("--with-frames");
        }
        if options.insecure_disable_tls {
            cmd.arg("--insecure-disable-tls-host-verification");
        }

        // Proxy
        if let Some(ref proxy) = self.proxy {
            cmd.args(["--http-proxy", proxy]);
        }

        // HTTP timeout
        let http_timeout = timeout_duration.as_millis().to_string();
        cmd.args(["--http-timeout", &http_timeout]);

        // Target URL
        cmd.arg(url);

        debug!(
            url = %url,
            format = %options.format,
            "Executing Lightpanda fetch"
        );

        let output_result = timeout(timeout_duration + Duration::from_secs(2), cmd.output()).await;

        let output = match output_result {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => {
                error!(url = %url, error = %e, "Lightpanda command execution failed");
                return Err(AppError::Lightpanda(format!(
                    "Lightpanda failed to run: {}",
                    e
                )));
            }
            Err(_) => {
                warn!(url = %url, "Lightpanda fetch timed out");
                return Err(AppError::Timeout(format!(
                    "Fetching URL '{}' timed out after {}ms",
                    url,
                    timeout_duration.as_millis()
                )));
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            // Check if there was navigation error
            return Err(AppError::Lightpanda(format!(
                "Lightpanda fetch error (code {:?}): {}",
                output.status.code(),
                if stderr.is_empty() { stdout } else { stderr }
            )));
        }

        // Lightpanda outputs "# Navigation failed" in stdout when navigation fails
        if stdout.starts_with("# Navigation failed") || stdout.contains("err=OperationTimedout") {
            return Err(AppError::Lightpanda(format!(
                "Navigation failed for URL '{}': {}",
                url,
                stdout.trim()
            )));
        }

        let took_ms = start_time.elapsed().as_millis();
        let length = stdout.len();

        info!(
            url = %url,
            took_ms = took_ms,
            bytes = length,
            "Lightpanda fetch completed successfully"
        );

        Ok(FetchResult {
            url: url.to_string(),
            format: options.format,
            content: stdout,
            length,
            took_ms,
        })
    }
}
