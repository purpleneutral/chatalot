# Manual Deployment

> **Status: Complete**

Building and running Chatalot from source without Docker.

## Overview

A manual deployment involves four steps:

1. Build the Rust server binary
2. Build the WASM crypto module
3. Build the Svelte web client
4. Configure and run the server

This approach is useful if you do not want Docker, need to integrate with an existing system, or want to use your distribution's service manager.

## Prerequisites

| Software | Version | Install |
|----------|---------|---------|
| Rust | 1.85+ | [rustup.rs](https://rustup.rs/) |
| Node.js | 22+ | [nodejs.org](https://nodejs.org/) |
| PostgreSQL | 15+ | See [Database Setup](./database-setup.md) |
| OpenSSL | 1.1+ | Pre-installed on most Linux distributions |
| wasm-pack | 0.13+ | `cargo install wasm-pack` |

Verify your toolchain:

```bash
rustc --version     # rustc 1.85+
node --version      # v22+
npm --version       # 10+
psql --version      # psql 15+
wasm-pack --version # wasm-pack 0.13+
```

## Step 1: Build the Rust Server

```bash
cd /path/to/chatalot

# Set offline mode for sqlx (no running DB needed at compile time)
export SQLX_OFFLINE=true

# Build the release binary
cargo build --release
```

The binary is produced at `target/release/chatalot-server`.

Build time is approximately 3-8 minutes depending on your hardware. Subsequent builds with only source changes are much faster due to incremental compilation.

## Step 2: Build the WASM Crypto Module

The web client needs the WASM-compiled crypto library for end-to-end encryption:

```bash
./scripts/build-wasm.sh
```

This runs `wasm-pack build` on the `chatalot-crypto-wasm` crate and outputs the package to `clients/web/src/lib/crypto/wasm/`.

## Step 3: Build the Svelte Web Client

```bash
cd clients/web
npm ci        # Install dependencies (clean install)
npm run build # Build the production SPA
cd ../..
```

The build output goes to `clients/web/build/`. This directory will be served as static files by the Rust server.

## Step 4: Generate Secrets

```bash
# Generate JWT signing keys
./scripts/generate-keys.sh

# Or manually:
mkdir -p secrets
openssl genpkey -algorithm Ed25519 -out secrets/jwt_private.pem
openssl pkey -in secrets/jwt_private.pem -pubout -out secrets/jwt_public.pem
chmod 600 secrets/jwt_private.pem
chmod 644 secrets/jwt_public.pem
```

Generate a TOTP encryption key:

```bash
openssl rand -hex 32
# Copy the output into your environment/config
```

## Step 5: Set Up the Database

Create the PostgreSQL database and user (see [Database Setup](./database-setup.md) for details):

```bash
sudo -u postgres psql -c "CREATE USER chatalot WITH PASSWORD 'your_password';"
sudo -u postgres psql -c "CREATE DATABASE chatalot OWNER chatalot;"
```

Migrations run automatically when the server starts. No manual migration step is needed.

## Step 6: Configure and Run

### Environment Variables

Set the required environment variables. You can use a shell script, systemd environment file, or export them directly:

```bash
export DATABASE_URL="postgres://chatalot:your_password@localhost:5432/chatalot"
export JWT_PRIVATE_KEY_PATH="./secrets/jwt_private.pem"
export JWT_PUBLIC_KEY_PATH="./secrets/jwt_public.pem"
export TOTP_ENCRYPTION_KEY="your_hex_key_here"
export LISTEN_ADDR="0.0.0.0:8080"
export STATIC_FILES_PATH="./clients/web/build"
export FILE_STORAGE_PATH="./data/files"
export MAX_FILE_SIZE_MB="100"
export REGISTRATION_MODE="invite_only"
export RUST_LOG="chatalot_server=info,tower_http=info"
```

### Create the File Storage Directory

```bash
mkdir -p ./data/files
```

### Run the Server

```bash
./target/release/chatalot-server
```

You should see:

```
INFO chatalot_server: Starting Chatalot server on 0.0.0.0:8080
INFO chatalot_server: Database connected
INFO chatalot_server: Migrations applied
INFO chatalot_server: Listening on 0.0.0.0:8080
```

## Running as a systemd Service

For production, run Chatalot as a systemd service so it starts on boot and restarts on failure.

### Create a System User

```bash
sudo useradd --system --shell /usr/sbin/nologin --home-dir /opt/chatalot chatalot
```

### Install Files

```bash
sudo mkdir -p /opt/chatalot/{bin,static,secrets,data/files,migrations}

# Copy the binary
sudo cp target/release/chatalot-server /opt/chatalot/bin/

# Copy the web client build
sudo cp -r clients/web/build/* /opt/chatalot/static/

# Copy migrations
sudo cp -r migrations/* /opt/chatalot/migrations/

# Copy secrets
sudo cp secrets/jwt_private.pem secrets/jwt_public.pem /opt/chatalot/secrets/

# Set ownership
sudo chown -R chatalot:chatalot /opt/chatalot
sudo chmod 600 /opt/chatalot/secrets/jwt_private.pem
```

### Create an Environment File

Create `/opt/chatalot/.env`:

```bash
DATABASE_URL=postgres://chatalot:your_password@localhost:5432/chatalot
JWT_PRIVATE_KEY_PATH=/opt/chatalot/secrets/jwt_private.pem
JWT_PUBLIC_KEY_PATH=/opt/chatalot/secrets/jwt_public.pem
TOTP_ENCRYPTION_KEY=your_hex_key_here
LISTEN_ADDR=0.0.0.0:8080
STATIC_FILES_PATH=/opt/chatalot/static
FILE_STORAGE_PATH=/opt/chatalot/data/files
MAX_FILE_SIZE_MB=100
REGISTRATION_MODE=invite_only
RUST_LOG=chatalot_server=info,tower_http=info
```

```bash
sudo chown chatalot:chatalot /opt/chatalot/.env
sudo chmod 600 /opt/chatalot/.env
```

### Create the systemd Unit

Create `/etc/systemd/system/chatalot.service`:

```ini
[Unit]
Description=Chatalot Chat Server
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=chatalot
Group=chatalot
WorkingDirectory=/opt/chatalot
EnvironmentFile=/opt/chatalot/.env
ExecStart=/opt/chatalot/bin/chatalot-server
Restart=on-failure
RestartSec=5

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/chatalot/data
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

### Enable and Start

```bash
sudo systemctl daemon-reload
sudo systemctl enable chatalot
sudo systemctl start chatalot

# Check status
sudo systemctl status chatalot

# View logs
sudo journalctl -u chatalot -f
```

## Updating a Manual Deployment

```bash
cd /path/to/chatalot
git pull

# Rebuild
export SQLX_OFFLINE=true
cargo build --release

# Rebuild web client
cd clients/web && npm ci && npm run build && cd ../..

# Rebuild WASM (if crypto crate changed)
./scripts/build-wasm.sh

# Update installed files
sudo cp target/release/chatalot-server /opt/chatalot/bin/
sudo cp -r clients/web/build/* /opt/chatalot/static/
sudo cp -r migrations/* /opt/chatalot/migrations/
sudo chown -R chatalot:chatalot /opt/chatalot

# Restart (migrations run automatically on startup)
sudo systemctl restart chatalot
```

## Next Step

For backup procedures, see [Backup and Restore](./backup-and-restore.md).
