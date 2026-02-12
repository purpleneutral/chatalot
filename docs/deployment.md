# Deployment Guide

## Prerequisites

- A Linux server with Docker Engine 24+ and Docker Compose v2
- At least 2 GB RAM (for building the Rust binary in Docker)
- OpenSSL (for generating Ed25519 JWT keys)
- Git

## Quick Deploy

```bash
git clone <repo-url> chatalot
cd chatalot
./scripts/generate-secrets.sh
docker compose up -d --build
```

The app will be available at `http://<server-ip>:8080`.

## What Gets Created

### Secrets

The `generate-secrets.sh` script creates:

| File | Purpose |
|------|---------|
| `secrets/jwt_private.pem` | Ed25519 private key for signing JWTs |
| `secrets/jwt_public.pem` | Ed25519 public key for verifying JWTs |
| `.env` | Environment variables (DB password, TOTP key, etc.) |

These files are gitignored and should be backed up securely.

### Docker Volumes

| Volume | Purpose |
|--------|---------|
| `postgres_data` | PostgreSQL database files |
| `file_storage` | Encrypted uploaded files |

### Containers

| Container | Image | Purpose |
|-----------|-------|---------|
| `chatalot-server` | Built from Dockerfile | Rust server + Svelte SPA |
| `chatalot-db` | `postgres:17` | PostgreSQL database |
| `chatalot-tunnel` | `cloudflare/cloudflared` | Cloudflare Tunnel (production profile only) |

## Configuration

### Environment Variables

Edit `.env` to customize:

```bash
# Database (auto-generated, usually no need to change)
DATABASE_URL=postgres://chatalot:<password>@postgres:5432/chatalot
DB_PASSWORD=<password>

# JWT keys (paths inside the container, mapped via Docker secrets)
JWT_PRIVATE_KEY_PATH=/run/secrets/jwt_private_key
JWT_PUBLIC_KEY_PATH=/run/secrets/jwt_public_key

# TOTP encryption key (hex-encoded, auto-generated)
TOTP_ENCRYPTION_KEY=<hex>

# Server
LISTEN_ADDR=0.0.0.0:8080
STATIC_FILES_PATH=/app/static
RUST_LOG=chatalot_server=info,tower_http=info

# Files
FILE_STORAGE_PATH=/app/data/files
MAX_FILE_SIZE_MB=100

# Cloudflare Tunnel (optional, for production profile)
CLOUDFLARE_TUNNEL_TOKEN=your_token_here
```

### Log Levels

Adjust `RUST_LOG` for debugging:

```bash
# Production
RUST_LOG=chatalot_server=info,tower_http=info

# Debug all server logs
RUST_LOG=chatalot_server=debug,tower_http=debug

# Debug specific modules
RUST_LOG=chatalot_server::ws=debug,chatalot_server::routes::auth=debug
```

## Reverse Proxy Setup

The server listens on port 8080 and serves both HTTP API and WebSocket connections. Any reverse proxy must support WebSocket passthrough.

### Traefik

Create `docker-compose.override.yml` (not tracked in git):

```yaml
services:
  chatalot:
    labels:
      - "traefik.enable=true"
      - "traefik.docker.network=<your-traefik-network>"
      - "traefik.http.routers.chatalot.rule=Host(`chat.example.com`)"
      - "traefik.http.routers.chatalot.entrypoints=websecure"
      - "traefik.http.routers.chatalot.tls=true"
      - "traefik.http.services.chatalot.loadbalancer.server.port=8080"
    networks:
      - <your-traefik-network>

networks:
  <your-traefik-network>:
    external: true
```

### nginx

```nginx
server {
    listen 443 ssl http2;
    server_name chat.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /ws {
        proxy_pass http://localhost:8080/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_read_timeout 86400;
    }
}
```

### Cloudflare Tunnel

1. Create a tunnel in the Cloudflare dashboard
2. Add the tunnel token to `.env`:
   ```bash
   CLOUDFLARE_TUNNEL_TOKEN=eyJ...
   ```
3. Start with the production profile:
   ```bash
   docker compose --profile production up -d
   ```

The `cloudflared` container will establish an encrypted tunnel to Cloudflare's edge, which then proxies to `chatalot:8080` on the internal Docker network.

## Automated Deployment Script

The `scripts/deploy.sh` script automates the full workflow:

```bash
# Full deploy: commit, push, pull on server, rebuild
./scripts/deploy.sh "Add new feature"

# Just pull and restart (no commit)
./scripts/deploy.sh --pull-only
```

What it does:
1. Stages and commits all changes locally
2. Pushes to the configured git remote
3. SSHs to the server
4. Clones (first time) or pulls latest code
5. Generates secrets if missing (JWT keys, .env, Traefik override)
6. Runs `docker compose up -d --build`
7. Waits for the health check to pass

### Customizing the deploy script

Set these environment variables before running `deploy.sh`:

```bash
export DEPLOY_HOST="user@your-server"          # Required
export DEPLOY_DIR="/srv/chatalot"              # Default: /srv/chatalot
export DEPLOY_GIT_URL="git@github.com:you/chatalot.git"  # Required
export DEPLOY_BRANCH="master"                  # Default: master
export DEPLOY_DOMAIN="chat.example.com"        # Optional, for Traefik setup
export DEPLOY_NETWORK="web"                    # Default: web
```

## Database Management

### Migrations

Migrations run automatically on server startup. The server calls `sqlx::migrate!()` which applies any pending migrations from the `migrations/` directory.

### Backups

Back up the PostgreSQL data:

```bash
# Dump the database
docker exec chatalot-db pg_dump -U chatalot chatalot > backup.sql

# Or back up the Docker volume directly
docker run --rm -v chatalot_postgres_data:/data -v $(pwd):/backup \
  alpine tar czf /backup/postgres-backup.tar.gz /data
```

### Reset

To completely reset the database:

```bash
docker compose down -v   # removes volumes
docker compose up -d     # recreates everything fresh
```

## Monitoring

### Health Check

```bash
curl http://localhost:8080/api/health
# {"status":"ok","version":"0.1.0","uptime_secs":3600}
```

### Logs

```bash
# All services
docker compose logs -f

# Just the server
docker compose logs -f chatalot

# Just the database
docker compose logs -f postgres
```

### Resource Usage

```bash
docker stats chatalot-server chatalot-db
```

## Troubleshooting

### Server won't start

Check logs: `docker compose logs chatalot`

Common issues:
- **"DATABASE_URL must be set"**: `.env` file is missing or not mounted
- **"failed to connect to database"**: PostgreSQL isn't ready yet (check health check)
- **"JWT key file not found"**: `secrets/` directory missing or wrong path

### WebSocket won't connect

- Ensure your reverse proxy supports WebSocket upgrade
- Check that the `/ws` path is being proxied correctly
- Verify the JWT token is valid (not expired)

### Build fails in Docker

- **Out of memory**: Rust compilation needs ~2 GB RAM. Increase Docker memory limit or add swap.
- **Cargo.lock conflicts**: Delete `Cargo.lock` and rebuild

### Database migration errors

- Check `docker compose logs chatalot` for the specific migration error
- Migrations are idempotent — restarting the server will retry
