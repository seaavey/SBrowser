# ==========================================
# Stage 1: Build Rust binary
# ==========================================
FROM rust:1-slim-bookworm AS builder

WORKDIR /usr/src/sbrowser

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests and source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

# ==========================================
# Stage 2: Runtime image
# ==========================================
FROM debian:bookworm-slim AS runner

# Install essential runtime tools & SSL certificates
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled Rust binary
COPY --from=builder /usr/src/sbrowser/target/release/sbrowser /usr/local/bin/sbrowser

# Download and install Lightpanda headless browser binary
RUN curl -fsSL https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-x86_64-linux -o /usr/local/bin/lightpanda \
    && chmod +x /usr/local/bin/lightpanda /usr/local/bin/sbrowser

# Default environment variables
ENV SBROWSER_HOST=0.0.0.0 \
    SBROWSER_PORT=3000 \
    LIGHTPANDA_PATH=/usr/local/bin/lightpanda \
    RUST_LOG=sbrowser=info,tower_http=info,axum=info

# Expose API port
EXPOSE 3000

# Run API server
ENTRYPOINT ["/usr/local/bin/sbrowser"]
