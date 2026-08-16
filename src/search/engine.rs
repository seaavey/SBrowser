use std::time::Instant;
use tracing::{info, warn};
use url::form_urlencoded;

use crate::{
    error::AppError,
    lightpanda::{DumpFormat, FetchOptions, LightpandaClient},
    search::{
        models::{SearchOptions, SearchResponse},
        parser::SearchParser,
    },
};


#[derive(Clone)]
pub struct SearchService {
    client: LightpandaClient,
}

impl SearchService {
    pub fn new(client: LightpandaClient) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &LightpandaClient {
        &self.client
    }

    pub fn build_search_url(&self, query: &str) -> String {
        let encoded_query: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
        format!("https://search.brave.com/search?q={}", encoded_query)
    }

    pub async fn search(&self, options: SearchOptions) -> Result<SearchResponse, AppError> {
        let start_time = Instant::now();
        let query = options.query.trim();

        if query.is_empty() {
            return Err(AppError::BadRequest("Search query cannot be empty".to_string()));
        }

        let search_url = self.build_search_url(query);

        info!(
            query = %query,
            engine = %options.engine,
            url = %search_url,
            "Executing web search with Lightpanda"
        );

        let fetch_opts = FetchOptions {
            format: DumpFormat::Html,
            timeout_ms: options.timeout_ms,
            wait_ms: Some(3000),
            ..Default::default()
        };

        let fetch_result = self.client.fetch(&search_url, &fetch_opts).await?;

        let mut results = SearchParser::parse(&fetch_result.content, options.limit);

        // If deep scraping is enabled, enrich top results with markdown content
        if options.scrape_content && !results.is_empty() {
            let scrape_count = options.scrape_limit.min(results.len());
            info!(
                count = scrape_count,
                "Enriching search results with Lightpanda markdown extraction"
            );

            let fetch_tasks: Vec<_> = results
                .iter()
                .take(scrape_count)
                .map(|item| {
                    let client = self.client.clone();
                    let url = item.url.clone();
                    let timeout_ms = options.timeout_ms;
                    async move {
                        let opts = FetchOptions {
                            format: DumpFormat::Markdown,
                            timeout_ms,
                            wait_ms: Some(3000),
                            ..Default::default()
                        };
                        match client.fetch(&url, &opts).await {
                            Ok(res) => (url, Some(res.content)),
                            Err(e) => {
                                warn!(url = %url, error = %e, "Failed to scrape result page content");
                                (url, None)
                            }
                        }
                    }
                })
                .collect();

            let enriched_results = futures::future::join_all(fetch_tasks).await;

            for (idx, (_url, content)) in enriched_results.into_iter().enumerate() {
                if let Some(item) = results.get_mut(idx) {
                    item.content = content;
                }
            }
        }

        let took_ms = start_time.elapsed().as_millis();
        let total_results = results.len();

        info!(
            query = %query,
            total_results = total_results,
            took_ms = took_ms,
            "Search completed"
        );

        Ok(SearchResponse {
            query: query.to_string(),
            engine: options.engine,
            total_results,
            took_ms,
            results,
        })
    }
}
