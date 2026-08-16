# 🌐 SBrowser — REST API Pencarian Web & Scraping dengan Lightpanda

[![CI](https://github.com/seaavey/SBrowser/actions/workflows/ci.yml/badge.svg)](https://github.com/seaavey/SBrowser/actions/workflows/ci.yml)
[![Docker](https://github.com/seaavey/SBrowser/actions/workflows/docker-publish.yml/badge.svg)](https://github.com/seaavey/SBrowser/actions/workflows/docker-publish.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**SBrowser** adalah layanan REST API pencarian web (*web search*) dan ekstraksi konten (*web scraping*) berperforma tinggi yang dibangun menggunakan bahasa pemrograman **Rust** ([Axum](https://github.com/tokio-rs/axum) + [Tokio](https://tokio.rs)).

Layanan ini ditenagai oleh headless browser berbasis Zig, **[Lightpanda](https://github.com/lightpanda-io/browser)**, untuk rendering JavaScript ultra-cepat dan ekstraksi Markdown bersih, serta dilengkapi dengan perisai **Redis Anti-DDoS (Rate Limiter & Caching)**.

---

## ✨ Fitur Utama

- 🔍 **Pencarian Web Brave (Brave Search)**: Pencarian web independen dan berkecepatan tinggi tanpa pelacak.
- ⚡ **Integrasi Headless Browser Lightpanda**: Rendering halaman web berbasis JavaScript super cepat dan langsung dikonversi ke format Markdown bersih atau HTML yang telah di-render.
- 🧠 **AI & LLM / RAG Ready**: Fitur **Deep Markdown Content Scrape** otomatis merender konten halaman web teratas ke format Markdown untuk langsung dijadikan konteks prompt LLM / agen AI.
- 🛡️ **Redis Anti-DDoS & Rate Limiting**: Pembatasan request berbasis IP otomatis (*HTTP 429 Too Many Requests*) untuk mencegah serangan *brute force* dan *flood/DDoS*.
- ⚡ **Redis Result Caching**: Caching cerdas untuk hasil pencarian dan render halaman dengan respon sub-milidetik (< 1ms) serta menghemat penggunaan CPU server.
- 🐳 **Siap Docker & Docker Compose**: Deployment instan satu perintah (`docker compose up -d`).
- 🚀 **REST API Murni**: Ringan, cepat, tanpa dependensi frontend (*API only*).

---

## 🚀 Panduan Memulai Cepat (Quick Start)

### Opsi 1: Menggunakan Docker Compose (Direkomendasikan)

Cukup jalankan satu perintah berikut di root folder:

```bash
docker compose up -d --build
```

Perintah ini akan otomatis:
1. Mengompilasi binary Rust SBrowser dalam container.
2. Mengunduh binary headless browser Lightpanda terbaru.
3. Menjalankan service SBrowser dan Redis.

Untuk melihat log kontainer:
```bash
docker compose logs -f
```

Untuk menghentikan kontainer:
```bash
docker compose down
```

---

### Opsi 2: Menjalankan Secara Lokal (Native Cargo)

#### 1. Prasyarat & Binary Lightpanda
Binary `lightpanda` sudah terpasang di folder `bin/lightpanda`. Jika ingin mengunduh ulang versi terbaru:
```bash
curl -fsSL https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-x86_64-linux -o bin/lightpanda
chmod +x bin/lightpanda
```

#### 2. Jalankan Server API
```bash
cargo run
```

Server akan aktif secara default di `http://0.0.0.0:3000`.

---

## 📡 Dokumentasi Endpoint API

### 1. Pencarian Web (`GET /api/v1/search`)

Melakukan pencarian web menggunakan Brave Search dan mengekstrak daftar hasil pencarian.

#### Query Parameters:
| Parameter | Tipe | Default | Wajib? | Keterangan |
|---|---|---|---|---|
| `q` | `string` | - | **Ya** | Kata kunci pencarian. |
| `limit` | `number` | `10` | Tidak | Jumlah hasil pencarian yang dikembalikan. |
| `scrape` | `boolean` | `false` | Tidak | Jika `true`, konten halaman teratas akan di-render ke format Markdown. |
| `scrape_limit` | `number` | `3` | Tidak | Batas jumlah halaman yang di-scrape kontennya. |

#### Contoh Request (cURL):
```bash
curl -G "http://localhost:3000/api/v1/search" \
  --data-urlencode "q=rust tokio tutorial" \
  -d "limit=3" \
  -d "scrape=true"
```

#### Contoh Response:
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

### 2. Pencarian Web via JSON (`POST /api/v1/search`)

Melakukan pencarian menggunakan payload JSON pada body request.

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

### 3. Render & Ekstraksi Halaman (`GET /api/v1/fetch`)

Merender URL apa pun menggunakan headless browser Lightpanda dan mengembalikannya dalam format Markdown atau HTML.

#### Query Parameters:
| Parameter | Tipe | Default | Wajib? | Keterangan |
|---|---|---|---|---|
| `url` | `string` | - | **Ya** | URL halaman web yang ingin di-render. |
| `format` | `string` | `markdown` | Tidak | Format output: `markdown` atau `html`. |
| `wait_ms` | `number` | `3000` | Tidak | Waktu tunggu rendering JavaScript dalam milidetik. |

#### Contoh Request (cURL):
```bash
curl -G "http://localhost:3000/api/v1/fetch" \
  --data-urlencode "url=https://news.ycombinator.com" \
  -d "format=markdown"
```

#### Contoh Response:
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

### 4. Status Server & Health Check (`GET /health` atau `GET /`)

Mengecek status kesehatan server, kesiapan binary Lightpanda, status koneksi Redis, dan lama waktu aktif server (*uptime*).

```bash
curl "http://localhost:3000/health"
```

#### Contoh Response:
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

## 🛡️ Header Anti-DDoS & Rate Limiting

Ketika koneksi Redis aktif, setiap response API akan menyertakan header penanda kuota:
- `X-RateLimit-Limit`: Jumlah batas request per periode jendela waktu.
- `X-RateLimit-Remaining`: Sisa kuota request untuk alamat IP pengirim.
- `X-RateLimit-Reset`: Jumlah detik tersisa sebelum kuota di-reset kembali.

Jika request melebihi batas yang ditentukan, server otomatis merespons dengan status **HTTP 429 Too Many Requests**:
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

## ⚙️ Konfigurasi Environment Variables

Semua pengaturan server dapat dikonfigurasi melalui file `.env` atau environment variable:

| Variabel | Default | Deskripsi |
|---|---|---|
| `SBROWSER_HOST` | `0.0.0.0` | Host IP binding server. |
| `SBROWSER_PORT` | `3000` | Port listening server. |
| `LIGHTPANDA_PATH` | `./bin/lightpanda` | Jalur lokasi file binary Lightpanda. |
| `SBROWSER_TIMEOUT_MS` | `15000` | Batas timeout rendering browser (ms). |
| `SBROWSER_MAX_CONCURRENT` | `8` | Batas konkurensi proses headless browser bersamaan. |
| `HTTP_PROXY` | - | Proxy HTTP opsional (`http://proxy:port`). |
| `REDIS_URL` | `redis://127.0.0.1:6379` | URL koneksi server Redis (opsional). |
| `RATE_LIMIT_ENABLED` | `true` | Mengaktifkan/menonaktifkan proteksi rate limit. |
| `RATE_LIMIT_REQUESTS` | `60` | Jumlah batas request per IP dalam satu jendela waktu. |
| `RATE_LIMIT_WINDOW_SECS` | `60` | Durasi jendela waktu rate limit (detik). |
| `CACHE_ENABLED` | `true` | Mengaktifkan/menonaktifkan fitur caching hasil di Redis. |
| `CACHE_SEARCH_TTL_SECS` | `600` | Durasi cache hasil pencarian (10 menit). |
| `CACHE_FETCH_TTL_SECS` | `3600` | Durasi cache render halaman (1 jam). |

---

## 🔄 Alur CI/CD (GitHub Actions)

Proyek ini telah dilengkapi dengan workflow otomatis di folder `.github/workflows/`:
1. **CI (`ci.yml`)**: Otomatis menjalankan `cargo check`, `cargo test`, dan uji build image Docker pada setiap push & pull request ke branch `main`.
2. **CD (`docker-publish.yml`)**: Otomatis membangun image Docker multi-tag dan mempublikasikannya ke **GitHub Container Registry (GHCR)** saat git tag baru (`v*`) dibuat.

---

## 📄 Lisensi

Didistribusikan di bawah lisensi MIT. Silakan gunakan secara bebas untuk keperluan personal maupun komersial.
