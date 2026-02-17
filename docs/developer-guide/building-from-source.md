# Building from Source

> **Status: Complete**

This guide covers building all Chatalot components from source: the Rust server, WASM crypto module, Svelte web client, and Tauri desktop app.

## Prerequisites

| Component | Requirement |
|-----------|------------|
| Rust | 1.84+ (edition 2024) |
| Node.js | 22+ |
| PostgreSQL | 17 |
| wasm-pack | Latest (`cargo install wasm-pack`) |

### Platform-Specific

**Linux (for Tauri desktop):**
- WebKitGTK development libraries
- On Ubuntu/Debian: `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libayatana-appindicator3-dev librsvg2-dev`

**Windows (for Tauri desktop):**
- WebView2 (included with Windows 10/11)
- Visual Studio Build Tools with C++ workload

## Clone the Repository

```bash
git clone https://github.com/purpleneutral/chatalot.git
cd chatalot
```

## Generate Secrets

```bash
./scripts/generate-secrets.sh
```

This creates:
- `secrets/jwt_private.pem` - Ed25519 private key for JWT signing
- `secrets/jwt_public.pem` - Ed25519 public key for JWT verification
- `.env` file from `.env.example` template

## Database Setup

Start PostgreSQL (easiest via Docker):

```bash
docker compose up postgres -d
```

Or configure an existing PostgreSQL 17 instance and set `DATABASE_URL` in `.env`:
```
DATABASE_URL=postgres://chatalot:password@localhost:5432/chatalot
```

Migrations run automatically on server startup.

## Build the Rust Server

```bash
# Development build
cargo build

# Production build (optimized)
cargo build --release

# Run directly
cargo run
```

The server listens on `0.0.0.0:8080` by default (configurable via `LISTEN_ADDR` in `.env`).

> **Note:** Chatalot uses sqlx with runtime-checked queries. Set `SQLX_OFFLINE=true` if building without a live database connection.

## Build the WASM Crypto Module

The crypto module must be built before the web client:

```bash
./scripts/build-wasm.sh          # Release build (optimized for size)
./scripts/build-wasm.sh --dev    # Development build (faster)
```

This compiles `crates/chatalot-crypto-wasm/` to WebAssembly and outputs to `clients/web/src/lib/crypto/wasm/`.

The WASM build uses:
- `opt-level = "s"` (size optimization)
- LTO enabled (link-time optimization)
- `getrandom` with `js` feature (uses `crypto.getRandomValues()` in browser)

## Build the Web Client

```bash
cd clients/web
npm install
npm run build    # Production build
npm run dev      # Development server (port 5173, proxies API to 8080)
```

The production build outputs static files that the Rust server serves from `./static`.

## Build the Tauri Desktop App

```bash
# Install Tauri CLI
cargo install tauri-cli

# Build the desktop app
cd clients/desktop/src-tauri
cargo tauri build
```

The output binary is platform-specific (`.deb`/`.AppImage` on Linux, `.msi` on Windows, `.dmg` on macOS).

> **Status: Beta** — The desktop app builds and runs but needs polish. The web client is the primary interface.

## Docker Build

For production deployment, use the multi-stage Docker build:

```bash
docker compose up -d
```

The Dockerfile has four stages:
1. **Rust builder** — Compiles the server binary (`rust:1.93-bookworm`)
2. **WASM builder** — Compiles crypto to WebAssembly
3. **Web builder** — Builds the Svelte SPA (`node:22-bookworm`)
4. **Runtime** — Minimal Debian image with only the binary and static files

## Development Workflow

A typical development setup:

```bash
# Terminal 1: Database
docker compose up postgres -d

# Terminal 2: Rust server
cargo run

# Terminal 3: Web dev server (with hot reload)
cd clients/web && npm run dev
```

The Vite dev server proxies API requests to the Rust server on port 8080.

## Verification

```bash
# Run all tests
cargo test

# Lint Rust code
cargo clippy -- -W clippy::all

# Check Svelte types
cd clients/web && npm run check

# Build web client (should produce 0 warnings)
cd clients/web && npm run build
```

## Environment Variables

Copy `.env.example` to `.env` and configure:

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | (required) | PostgreSQL connection string |
| `LISTEN_ADDR` | `0.0.0.0:8080` | Server bind address |
| `JWT_PRIVATE_KEY_PATH` | `./secrets/jwt_private.pem` | Ed25519 private key |
| `JWT_PUBLIC_KEY_PATH` | `./secrets/jwt_public.pem` | Ed25519 public key |
| `TOTP_ENCRYPTION_KEY` | (optional) | Hex key to encrypt TOTP secrets at rest |
| `FILE_STORAGE_PATH` | `./data/files` | File upload storage |
| `MAX_FILE_SIZE_MB` | `100` | Max file upload size |
| `UPLOAD_QUOTA_MB` | `500` | Per-user upload quota |
| `REGISTRATION_MODE` | `invite_only` | open/invite_only/closed |
| `RUST_LOG` | `info` | Log level |

## Related Pages

- [Architecture](./architecture.md)
- [Project Structure](./project-structure.md)
- [Testing](./testing.md)
