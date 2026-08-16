# 🌐 SBrowser — Web Search & Scraping REST API with Lightpanda

[![CI](https://github.com/seaavey/SBrowser/actions/workflows/ci.yml/badge.svg)](https://github.com/seaavey/SBrowser/actions/workflows/ci.yml)
[![Docker](https://github.com/seaavey/SBrowser/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/seaavey/SBrowser/actions/workflows/docker-publish.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**SBrowser** is a high-performance web search and scraping REST API built in **Rust** ([Axum](https://github.com/tokio-rs/axum) + [Tokio](https://tokio.rs)).

It is powered by the Zig-based headless browser **[Lightpanda](https://github.com/lightpanda-io/browser)** for ultra-fast JavaScript rendering and clean Markdown extraction, equipped with a **Redis Anti-DDoS Shield (Rate Limiter & Caching)**.

---

## ✨ Key Features

- 🔍 **Brave Web Search**: Fast, independent, tracker-free web search powered by Brave Search.
- ⚡ **Lightpanda Headless Browser Integration**: Ultra-fast JavaScript page rendering with direct conversion to clean Markdown or rendered HTML.
- 🧠 **AI & LLM / RAG Ready**: **Deep Markdown Content Scrape** automatically extracts and renders top search result pages into clean Markdown for prompt context in LLM / AI agents.
- 🛡️ **Redis Anti-DDoS & Rate Limiting**: Automatic IP-based request limiting (*HTTP 429 Too Many Requests*) to prevent brute-force attacks and flood/DDoS abuse.
- ⚡ **Redis Result Caching**: Intelligent sub-millisecond (< 1ms) response caching for search queries and page rendering to conserve server CPU resources.
- 🐳 **Docker & Docker Compose Ready**: One-command instant deployment (`docker compose up -d`).
- 🚀 **Pure REST API**: Lightweight, blazing fast, zero frontend overhead (*API only*).

---

## 🚀 Quick Start

### Option 1: Using Docker Compose (Recommended)

Run a single command in the project root directory:

```bash
docker compose up -d --build
```

This will automatically:
1. Compile the SBrowser Rust release binary inside a container.
2. Download the latest Lightpanda headless browser binary.
3. Start both SBrowser and Redis services.

View container logs:
```bash
docker compose logs -f
```

Stop containers:
```bash
docker compose down
```

---

### Option 2: Running Locally (Native Cargo)

#### 1. Prerequisites & Lightpanda Binary
The `lightpanda` binary is already placed in the `bin/` folder. If you need to re-download the latest version:
```bash
curl -fsSL https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-x86_64-linux -o bin/lightpanda
chmod +x bin/lightpanda
```

#### 2. Run the API Server
```bash
cargo run
```

The server starts by default on `http://0.0.0.0:3000`.

---

## 📡 API Endpoint Documentation

### 1. Web Search (`GET /api/v1/search`)

Performs a web search using Brave Search and returns structured search results.

#### Query Parameters:
| Parameter | Type | Default | Required? | Description |
|---|---|---|---|---|
| `q` | `string` | - | **Yes** | Search keyword query. |
| `limit` | `number` | `10` | No | Maximum number of search results to return. |
| `scrape` | `boolean` | `false` | No | If `true`, renders and extracts full markdown content for top pages. |
| `scrape_limit` | `number` | `3` | No | Maximum number of top result pages to scrape content from. |

#### cURL Example:
```bash
curl -G "http://localhost:3000/api/v1/search" \
  --data-urlencode "q=rust tokio tutorial" \
  -d "limit=3" \
  -d "scrape=true"
```

#### Response Example:
```json
{
  "query": "rust tokio tutorial",
  "engine": "brave",
  "total_results": 3,
  "took_ms": 420,
  "results": [
    {
      "rank": 1,
      "title": "Tutorial | Tokio - An asynchronous Rust runtime",
      "url": "https://tokio.rs/tokio/tutorial",
      "snippet": "A comprehensive guide to asynchronous programming in Rust using Tokio...",
      "content": "# Tutorial\n\nWelcome to the Tokio tutorial! In this guide..."
    }
  ]
}
```

---

### 2. Web Search via JSON (`POST /api/v1/search`)

Performs a search using a JSON body payload.

```bash
curl -X POST "http://localhost:3000/api/v1/search" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "model context protocol zig",
    "limit": 5,
    "scrape_content": true,
    "scrape_limit": 2
  }'
```

---

### 3. Page Render & Scrape (`GET /api/v1/fetch`)

Renders any URL with the Lightpanda headless browser and returns clean Markdown or HTML.

#### Query Parameters:
| Parameter | Type | Default | Required? | Description |
|---|---|---|---|---|
| `url` | `string` | - | **Yes** | Target web page URL to render. |
| `format` | `string` | `markdown` | No | Output format: `markdown` or `html`. |
| `wait_ms` | `number` | `3000` | No | JavaScript rendering wait time in milliseconds. |

#### cURL Example:
```bash
curl -G "http://localhost:3000/api/v1/fetch" \
  --data-urlencode "url=https://news.ycombinator.com" \
  -d "format=markdown"
```

#### Response Example:
```json
{
  "url": "https://news.ycombinator.com",
  "format": "markdown",
  "content": "# Hacker News\n\n| 1. | [Show HN: ...] | ...",
  "length": 19111,
  "took_ms": 1784
}
```

---

### 4. Server Status & Health Check (`GET /health` or `GET /`)

Checks the health of the server, Lightpanda binary readiness, Redis connection status, and server uptime.

```bash
curl "http://localhost:3000/health"
```

#### Response Example:
```json
{
  "status": "ok",
  "version": "0.1.0",
  "lightpanda_path": "/usr/local/bin/lightpanda",
  "lightpanda_status": "1.0.0-nightly.8662+be7b87f3",
  "redis_status": "connected (redis://redis:6379)",
  "uptime_secs": 120
}
```

---

## 🛡️ Anti-DDoS & Rate Limiting Headers

When Redis is enabled, every API response includes rate limiting quota headers:
- `X-RateLimit-Limit`: Maximum allowed requests per window period.
- `X-RateLimit-Remaining`: Remaining request quota for the client IP.
- `X-RateLimit-Reset`: Number of seconds remaining until the quota resets.

If requests exceed the limit, the server responds with **HTTP 429 Too Many Requests**:
```json
{
  "error": {
    "message": "Rate limit exceeded. Too many requests, please slow down.",
    "status": 429,
    "retry_after_secs": 45
  }
}
```

---

## ⚙️ Environment Variables Configuration

All server configurations can be customized via `.env` or environment variables:

| Variable | Default | Description |
|---|---|---|
| `SBROWSER_HOST` | `0.0.0.0` | Host IP address binding. |
| `SBROWSER_PORT` | `3000` | Server listening port. |
| `LIGHTPANDA_PATH` | `./bin/lightpanda` | Path to the Lightpanda binary executable. |
| `SBROWSER_TIMEOUT_MS` | `15000` | Headless browser rendering timeout (ms). |
| `SBROWSER_MAX_CONCURRENT` | `8` | Maximum concurrent browser processes. |
| `HTTP_PROXY` | - | Optional HTTP proxy (`http://proxy:port`). |
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis server connection URL (optional). |
| `RATE_LIMIT_ENABLED` | `true` | Enable or disable rate limiting protection. |
| `RATE_LIMIT_REQUESTS` | `60` | Request limit per client IP per window. |
| `RATE_LIMIT_WINDOW_SECS` | `60` | Rate limit window duration (seconds). |
| `CACHE_ENABLED` | `true` | Enable or disable Redis result caching. |
| `CACHE_SEARCH_TTL_SECS` | `600` | Search result cache TTL (10 minutes). |
| `CACHE_FETCH_TTL_SECS` | `3600` | Scraped page cache TTL (1 hour). |

---

## 🔄 CI/CD Pipeline (GitHub Actions)

This repository includes automated workflows under `.github/workflows/`:
1. **CI (`ci.yml`)**: Runs `cargo check`, `cargo test`, and builds the Docker image on every push and pull request to `main`.
2. **CD (`docker-publish.yml`)**: Automatically builds and publishes multi-tag Docker images to **GitHub Container Registry (GHCR)** when a new release tag (`v*`) is pushed.

---

## 📄 License

Distributed under the MIT License. See [LICENSE](LICENSE) for details.
