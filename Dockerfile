# syntax=docker/dockerfile:1

# Stage 1: Build the Rust server
FROM rust:1.93-bookworm AS builder
WORKDIR /build

# Cache dependencies by copying only manifests first
COPY Cargo.toml Cargo.lock ./
COPY crates/chatalot-server/Cargo.toml crates/chatalot-server/
COPY crates/chatalot-db/Cargo.toml crates/chatalot-db/
COPY crates/chatalot-crypto/Cargo.toml crates/chatalot-crypto/
COPY crates/chatalot-common/Cargo.toml crates/chatalot-common/

# Create dummy source files for dependency caching
RUN mkdir -p crates/chatalot-server/src && echo "fn main() {}" > crates/chatalot-server/src/main.rs \
    && mkdir -p crates/chatalot-db/src && echo "" > crates/chatalot-db/src/lib.rs \
    && mkdir -p crates/chatalot-crypto/src && echo "" > crates/chatalot-crypto/src/lib.rs \
    && mkdir -p crates/chatalot-common/src && echo "" > crates/chatalot-common/src/lib.rs \
    && mkdir -p migrations && touch migrations/.keep

# Build dependencies only — cache mounts persist the registry + build artifacts between builds
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/target \
    cargo build --release 2>/dev/null || true

# Copy actual source and rebuild (only our code recompiles, deps are cached)
COPY crates/ crates/
COPY migrations/ migrations/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/build/target \
    touch crates/*/src/*.rs && cargo build --release \
    && cp target/release/chatalot-server /build/chatalot-server

# Stage 2: Build the web client
FROM node:22-bookworm AS web-builder
WORKDIR /build
COPY clients/web/package.json clients/web/package-lock.json* ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci || npm install
COPY clients/web/ .
RUN npm run build

# Stage 3: Minimal runtime image
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r chatalot && useradd -r -g chatalot -s /bin/false chatalot
WORKDIR /app

COPY --from=builder /build/chatalot-server .
COPY --from=web-builder /build/build ./static
COPY migrations/ ./migrations/

RUN mkdir -p /app/data/files && chown -R chatalot:chatalot /app
USER chatalot

EXPOSE 8080

ENV RUST_LOG=info
HEALTHCHECK --interval=30s --timeout=10s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

CMD ["./chatalot-server"]
