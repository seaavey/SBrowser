use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

use super::models::SearchResultItem;

pub struct SearchParser;

impl SearchParser {
    pub fn parse(html_content: &str, limit: usize) -> Vec<SearchResultItem> {
        let document = Html::parse_document(html_content);
        let mut results = Self::parse_brave(&document);

        // If specific engine parsing failed, try generic fallback parser
        if results.is_empty() {
            results = Self::parse_generic(&document);
        }

        // Limit results and assign sequential ranks
        results
            .into_iter()
            .take(limit)
            .enumerate()
            .map(|(idx, mut item)| {
                item.rank = idx + 1;
                item
            })
            .collect()
    }

    fn parse_brave(document: &Html) -> Vec<SearchResultItem> {
        let mut items = Vec::new();
        let mut seen = HashSet::new();
        let snippet_container_sel = match Selector::parse("div.result-wrapper, div.snippet, div[data-type=\"web\"]") {
            Ok(s) => s,
            Err(_) => return items,
        };

        let title_sel = Selector::parse("span.title, a.title, a.heading, .title, .result-header, h3").ok();
        let link_sel = Selector::parse("a[href]").ok();
        let desc_sel = Selector::parse("div.description, p.snippet-description, div.snippet-content, .snippet-description, .description, div[class*=\"description\"]").ok();

        for element in document.select(&snippet_container_sel) {
            let title = if let Some(ref t_sel) = title_sel {
                element
                    .select(t_sel)
                    .next()
                    .map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            };

            let raw_url = if let Some(ref l_sel) = link_sel {
                element
                    .select(l_sel)
                    .find_map(|a| a.value().attr("href").map(|h| h.to_string()))
                    .unwrap_or_default()
            } else {
                String::new()
            };

            let clean_url = Self::clean_url(&raw_url);
            if title.is_empty() || clean_url.is_empty() || !clean_url.starts_with("http") || !seen.insert(clean_url.clone()) {
                continue;
            }

            let snippet = if let Some(ref d_sel) = desc_sel {
                element
                    .select(d_sel)
                    .next()
                    .map(|d| d.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_default()
            } else {
                String::new()
            };

            items.push(SearchResultItem {
                rank: 0,
                title: clean_text(&title),
                url: clean_url,
                snippet: clean_text(&snippet),
                content: None,
            });
        }

        items
    }

    fn parse_generic(document: &Html) -> Vec<SearchResultItem> {
        let mut items = Vec::new();
        let mut seen = HashSet::new();
        if let Ok(link_sel) = Selector::parse("h2 a, h3 a, a:has(h2), a:has(h3)") {
            for element in document.select(&link_sel) {
                let title = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                let raw_url = element.value().attr("href").unwrap_or_default().to_string();
                let clean_url = Self::clean_url(&raw_url);

                if title.len() > 3 && clean_url.starts_with("http") && seen.insert(clean_url.clone()) {
                    items.push(SearchResultItem {
                        rank: 0,
                        title: clean_text(&title),
                        url: clean_url,
                        snippet: String::new(),
                        content: None,
                    });
                }
            }
        }
        items
    }

    pub fn clean_url(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        if trimmed.starts_with("//") {
            return format!("https:{}", trimmed);
        }

        if trimmed.starts_with("/url?") || trimmed.contains("search.brave.com/url?") {
            let full_url = if trimmed.starts_with('/') {
                format!("https://search.brave.com{}", trimmed)
            } else {
                trimmed.to_string()
            };

            if let Ok(parsed) = Url::parse(&full_url) {
                for (k, v) in parsed.query_pairs() {
                    if k == "url" || k == "q" {
                        return v.to_string();
                    }
                }
            }
        }

        trimmed.to_string()
    }
}

fn clean_text(input: &str) -> String {
    let mut cleaned = input
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    while cleaned.contains("  ") {
        cleaned = cleaned.replace("  ", " ");
    }
    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_url() {
        let raw = "https://www.rust-lang.org/";
        let cleaned = SearchParser::clean_url(raw);
        assert_eq!(cleaned, "https://www.rust-lang.org/");
    }

    #[test]
    fn test_parse_brave_html() {
        let html = r#"
        <div class="snippet" data-type="web">
            <span class="title">Anthropic | AI research and products</span>
            <a href="https://www.anthropic.com">Link</a>
            <p class="snippet-description">Anthropic is an AI safety and research company.</p>
        </div>
        <div class="snippet" data-type="web">
            <span class="title">Rust Programming Language</span>
            <a href="https://www.rust-lang.org">Link</a>
            <p class="snippet-description">Empowering everyone to build reliable and efficient software.</p>
        </div>
        "#;

        let results = SearchParser::parse(html, 5);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].rank, 1);
        assert_eq!(results[0].title, "Anthropic | AI research and products");
        assert_eq!(results[0].url, "https://www.anthropic.com");
        assert_eq!(
            results[0].snippet,
            "Anthropic is an AI safety and research company."
        );
        assert_eq!(results[1].rank, 2);
        assert_eq!(results[1].title, "Rust Programming Language");
        assert_eq!(results[1].url, "https://www.rust-lang.org");
    }

    #[test]
    fn test_parse_generic_fallback() {
        let html = r#"
        <html>
            <body>
                <main>
                    <div>
                        <h3><a href="https://news.ycombinator.com">Hacker News</a></h3>
                    </div>
                </main>
            </body>
        </html>
        "#;

        let results = SearchParser::parse(html, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Hacker News");
        assert_eq!(results[0].url, "https://news.ycombinator.com");
    }
}
