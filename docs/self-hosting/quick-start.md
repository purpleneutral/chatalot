# Quick Start

> **Status: Complete**

Get a working Chatalot instance in 5 minutes using Docker.

## Prerequisites

Make sure you have Docker and Docker Compose installed. See [Requirements](./requirements.md) for details.

## Option A: Interactive Setup (Recommended)

The interactive installer walks you through every option with sensible defaults:

```bash
git clone https://github.com/purpleneutral/chatalot.git
cd chatalot
./scripts/install.sh
```

The installer will:

1. Check that Docker, Docker Compose, and OpenSSL are installed
2. Ask you about registration mode, server port, and optional features
3. Generate Ed25519 JWT signing keys
4. Create a `.env` file with secure random credentials
5. Build and start the Docker containers
6. Wait for the health check to pass

When it finishes, open `http://localhost:8080` in your browser.

## Option B: Manual Setup

If you prefer to set things up by hand:

### Step 1: Clone the Repository

```bash
git clone https://github.com/purpleneutral/chatalot.git
cd chatalot
```

### Step 2: Generate Secrets

```bash
./scripts/generate-secrets.sh
```

This creates:
- `secrets/jwt_private.pem` -- Ed25519 private key for signing JWTs
- `secrets/jwt_public.pem` -- Ed25519 public key for verifying JWTs
- `.env` -- Environment variables with a random database password and TOTP encryption key

### Step 3: Start the Containers

```bash
docker compose up -d
```

The first build takes **5-10 minutes** because it compiles the Rust server and WASM crypto module. Subsequent starts are nearly instant.

> **Tip:** On ARM64 systems (Raspberry Pi, Oracle Cloud, etc.), the install script offers a pre-built image that skips the compilation step entirely.

### Step 4: Verify It Is Running

```bash
curl http://localhost:8080/api/health
```

Expected response:

```json
{
  "status": "ok",
  "version": "0.17.0",
  "uptime_secs": 42,
  "db_healthy": true
}
```

### Step 5: Create Your Admin Account

1. Open `http://localhost:8080` in your browser.
2. Click **Register** and create your account.
3. The first registered user can be promoted to admin. Set `ADMIN_USERNAME` in your `.env` to your username and restart:

```bash
# Edit .env and set ADMIN_USERNAME=your_username
docker compose up -d
```

The server will grant admin privileges to that user on startup.

> **Important:** Save your **recovery code** when it is displayed during registration. It is the only way to reset your password without admin help.

## Step 6: Invite Others

By default, registration is set to `invite_only`. As an admin, you can generate invite codes from the admin panel to share with others.

If you set `REGISTRATION_MODE=open` during setup, switch to invite-only after your initial users have registered:

```bash
# In .env, change:
REGISTRATION_MODE=invite_only

# Then restart:
docker compose up -d
```

## What Is Running

After setup, you have two containers:

| Container | Image | Purpose |
|-----------|-------|---------|
| `chatalot-server` | Built from `Dockerfile` | Rust API server + Svelte web client |
| `chatalot-db` | `postgres:17` | PostgreSQL database |

And two Docker volumes:

| Volume | Purpose |
|--------|---------|
| `postgres_data` | Database files |
| `file_storage` | Uploaded files |

## Quick Reference

```bash
# View logs
docker compose logs -f chatalot

# Stop everything
docker compose down

# Start again
docker compose up -d

# Rebuild after code changes
docker compose up -d --build

# Check resource usage
docker stats chatalot-server chatalot-db
```

## Next Steps

- [Configuration](./configuration.md) -- Customize all environment variables
- [TLS and Reverse Proxy](./tls-and-reverse-proxy.md) -- Set up HTTPS for production
- [Cloudflare Tunnel](./cloudflare-tunnel.md) -- Expose your instance publicly without port forwarding
- [Security Hardening](./security-hardening.md) -- Lock down your production instance
