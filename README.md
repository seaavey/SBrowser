# 🌐 SBrowser — Web Search & Scraping API with Lightpanda

**SBrowser** adalah layanan REST API web search & scraping berperforma tinggi yang dibangun dengan **Rust** (Axum + Tokio), ditenagai oleh headless browser **[Lightpanda](https://github.com/lightpanda-io/browser)** (browser AI-native berbasis Zig), serta dilindungi oleh **Redis Anti-DDoS Rate Limiting & Caching Shield**.

---

## ✨ Fitur Utama

- 🔍 **Brave Web Search**: Pencarian web independen berkecepatan tinggi melalui **Brave Search**.
- ⚡ **Lightpanda Headless Browser Integration**: Rendering JavaScript ultra-cepat dan konversi langsung ke Markdown bersih atau HTML yang telah di-render.
- 🧠 **AI & LLM / RAG Ready**: Fitur **Deep Markdown Content Scrape** otomatis mengambil dan merender konten halaman web teratas ke format Markdown untuk langsung dijadikan konteks prompt LLM / agent.
- 🛡️ **Redis Anti-DDoS & Rate Limiting**: Perlindungan IP-based rate limiting otomatis (HTTP 429 Too Many Requests) untuk mencegah flood/serangan DDoS.
- ⚡ **Redis Result Caching**: Caching instan hasil pencarian dan render halaman untuk respon sub-milidetik dan hemat CPU.
- 🐳 **Docker & Docker Compose Ready**: Siap deploy sekali perintah dengan `docker compose up -d`.
- ⚡ **Fast & Lightweight REST API**: Server API murni tanpa frontend overhead dengan endpoint `/api/v1/search`, `/api/v1/fetch`, `/api/v1/engines`, dan `/health`.

---

## 🚀 Quick Start

### Opsi 1: Menjalankan dengan Docker Compose (Direkomendasikan)

Tinggal jalankan satu perintah, SBrowser dan Redis langsung aktif:

```bash
docker compose up -d --build
```

Cek status layanan:
```bash
docker compose ps
docker compose logs -f
```

---

### Opsi 2: Menjalankan Lokal (Native Cargo)

#### 1. Prasyarat & Binary Lightpanda
Binary `lightpanda` sudah terpasang di folder `bin/lightpanda`. Jika ingin mengunduh ulang:
```bash
curl -fsSL https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-x86_64-linux -o bin/lightpanda
chmod +x bin/lightpanda
```

#### 2. Jalankan Server API
```bash
cargo run
```

---

## 📡 API Endpoints

### 1. `GET /api/v1/search`

Melakukan pencarian web menggunakan Brave Search.

#### Query Parameters:
- `q` (_string, required_): Kata kunci pencarian.
- `limit` (_number, optional_): Jumlah hasil (default `10`).
- `scrape` (_boolean, optional_): Jika `true`, Lightpanda akan merender isi halaman teratas ke format Markdown (default `false`).
- `scrape_limit` (_number, optional_): Batas jumlah halaman yang di-scrape kontennya (default `3`).

#### Contoh cURL:
```bash
curl -G "http://localhost:3000/api/v1/search" \
  --data-urlencode "q=rust tokio tutorial" \
  -d "limit=5" \
  -d "scrape=true"
```

#### Contoh Response:
```json
{
  "query": "rust tokio tutorial",
  "engine": "brave",
  "total_results": 5,
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

### 2. `POST /api/v1/search`

Melakukan pencarian dengan payload JSON.

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

### 3. `GET /api/v1/fetch`

Merender URL apapun menggunakan Lightpanda dan mengembalikan Markdown atau HTML.

#### Query Parameters:
- `url` (_string, required_): Target URL.
- `format` (_string, optional_): `markdown` (default) atau `html`.
- `wait_ms` (_number, optional_): Waktu tunggu rendering dalam ms (default `3000`).

#### Contoh cURL:
```bash
curl -G "http://localhost:3000/api/v1/fetch" \
  --data-urlencode "url=https://github.com/lightpanda-io/browser" \
  -d "format=markdown"
```

---

### 4. `GET /health`

Mengecek status server, binary Lightpanda, dan status koneksi Redis.

```bash
curl "http://localhost:3000/health"
```

**Contoh Response:**
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

Ketika Redis aktif, setiap response menyertakan header Rate Limit:
- `X-RateLimit-Limit`: Batas maksimum request per window.
- `X-RateLimit-Remaining`: Sisa kuota request untuk IP Anda.
- `X-RateLimit-Reset`: Waktu detik sebelum kuota di-reset.

Jika request melebihi batas, server mengembalikan status **HTTP 429 Too Many Requests**:
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

## 🛠️ Konfigurasi Environment Variables

Semua konfigurasi dapat diatur melalui file `.env` atau environment variables:

```env
# Server
SBROWSER_HOST=0.0.0.0
SBROWSER_PORT=3000
LIGHTPANDA_PATH=./bin/lightpanda
SBROWSER_TIMEOUT_MS=15000
SBROWSER_MAX_CONCURRENT=8
# HTTP_PROXY=http://proxy-host:port

# Redis Shield & Caching
REDIS_URL=redis://127.0.0.1:6379
RATE_LIMIT_ENABLED=true
RATE_LIMIT_REQUESTS=60
RATE_LIMIT_WINDOW_SECS=60
CACHE_ENABLED=true
CACHE_SEARCH_TTL_SECS=600
CACHE_FETCH_TTL_SECS=3600
```
