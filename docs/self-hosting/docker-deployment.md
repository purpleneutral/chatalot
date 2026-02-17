# Docker Deployment

> **Status: Complete**

Full reference for the Docker Compose setup, service configuration, and customization options.

## docker-compose.yml Reference

Chatalot ships with a `docker-compose.yml` that defines three services:

### Services

#### `chatalot` (Application Server)

The main service. Builds from the multi-stage `Dockerfile` which:

1. **Stage 1:** Compiles the Rust server binary (with dependency caching)
2. **Stage 2:** Builds the WASM crypto module using `wasm-pack`
3. **Stage 3:** Builds the Svelte web client
4. **Stage 4:** Creates a minimal Debian runtime image with the binary, static files, and migrations

```yaml
chatalot:
  build: .
  container_name: chatalot-server
  restart: unless-stopped
  depends_on:
    postgres:
      condition: service_healthy
  environment:
    DATABASE_URL: postgres://chatalot:${DB_PASSWORD}@postgres:5432/chatalot
    JWT_PRIVATE_KEY_PATH: /run/secrets/jwt_private_key
    JWT_PUBLIC_KEY_PATH: /run/secrets/jwt_public_key
    TOTP_ENCRYPTION_KEY: ${TOTP_ENCRYPTION_KEY:-}
    RUST_LOG: "${RUST_LOG:-chatalot_server=info,tower_http=info}"
    FILE_STORAGE_PATH: /app/data/files
    MAX_FILE_SIZE_MB: "100"
    LISTEN_ADDR: "0.0.0.0:8080"
    STATIC_FILES_PATH: /app/static
    REGISTRATION_MODE: ${REGISTRATION_MODE:-invite_only}
    ADMIN_USERNAME: ${ADMIN_USERNAME:-}
    PUBLIC_URL: ${PUBLIC_URL:-}
    COMMUNITY_CREATION_MODE: ${COMMUNITY_CREATION_MODE:-admin_only}
    GIPHY_API_KEY: ${GIPHY_API_KEY:-}
  volumes:
    - file_storage:/app/data/files
  secrets:
    - jwt_private_key
    - jwt_public_key
  ports:
    - "8080:8080"
```

Key details:
- **Waits for PostgreSQL** using the `depends_on` health check condition
- **JWT keys** are mounted as Docker secrets (read-only files at `/run/secrets/`)
- **File uploads** are persisted in a named Docker volume
- **Port 8080** is exposed to the host

#### `postgres` (Database)

```yaml
postgres:
  image: postgres:17
  container_name: chatalot-db
  restart: unless-stopped
  environment:
    POSTGRES_DB: chatalot
    POSTGRES_USER: chatalot
    POSTGRES_PASSWORD: ${DB_PASSWORD}
  volumes:
    - postgres_data:/var/lib/postgresql/data
  healthcheck:
    test: ["CMD-SHELL", "pg_isready -U chatalot"]
    interval: 10s
    timeout: 5s
    retries: 5
```

Key details:
- **Not exposed** to the host network by default (only accessible within the `chatalot-net` Docker network)
- **Health check** ensures the database is ready before the application starts
- **Data** is persisted in a named Docker volume

#### `cloudflared` (Cloudflare Tunnel)

Two tunnel services are defined behind Docker Compose profiles (they do not start by default):

```yaml
# Named tunnel (persistent domain, requires token)
cloudflared:
  image: cloudflare/cloudflared:latest
  command: tunnel --no-autoupdate run
  environment:
    TUNNEL_TOKEN: ${CLOUDFLARE_TUNNEL_TOKEN}
  profiles:
    - production

# Quick tunnel (temporary public URL, no token needed)
cloudflared-quick:
  image: cloudflare/cloudflared:latest
  command: tunnel --no-autoupdate --url http://chatalot:8080
  profiles:
    - quick-tunnel
```

See [Cloudflare Tunnel](./cloudflare-tunnel.md) for details.

### Volumes

| Volume | Mount Point | Purpose |
|--------|-------------|---------|
| `postgres_data` | `/var/lib/postgresql/data` | PostgreSQL data files |
| `file_storage` | `/app/data/files` | Uploaded files |

### Secrets

| Secret | Source File | Container Path |
|--------|------------|----------------|
| `jwt_private_key` | `./secrets/jwt_private.pem` | `/run/secrets/jwt_private_key` |
| `jwt_public_key` | `./secrets/jwt_public.pem` | `/run/secrets/jwt_public_key` |

### Network

All services share a bridge network called `chatalot-net`. This allows the application to reach PostgreSQL via the hostname `postgres` without exposing the database to the host.

## Customization with docker-compose.override.yml

Docker Compose automatically merges `docker-compose.override.yml` (if present) with the main file. Use this for local customizations that should not be committed to git.

### Change the Host Port

```yaml
# docker-compose.override.yml
services:
  chatalot:
    ports:
      - "3000:8080"
```

### Expose PostgreSQL to the Host

```yaml
# docker-compose.override.yml
services:
  postgres:
    ports:
      - "5432:5432"
```

> **Warning:** Do not expose the database port on a public-facing server.

### Use a Pre-Built Image

Instead of building from source:

```yaml
# docker-compose.override.yml
services:
  chatalot:
    image: ghcr.io/purpleneutral/chatalot:latest
    build: !reset null
```

### Add Traefik Labels

```yaml
# docker-compose.override.yml
services:
  chatalot:
    labels:
      - "traefik.enable=true"
      - "traefik.docker.network=web"
      - "traefik.http.routers.chatalot.rule=Host(`chat.example.com`)"
      - "traefik.http.routers.chatalot.entrypoints=websecure"
      - "traefik.http.routers.chatalot.tls=true"
      - "traefik.http.services.chatalot.loadbalancer.server.port=8080"
    networks:
      - web

networks:
  web:
    external: true
```

### Use an External PostgreSQL

To use an existing PostgreSQL server instead of the bundled container:

```yaml
# docker-compose.override.yml
services:
  chatalot:
    depends_on: []  # Remove the postgres dependency
  postgres:
    profiles:
      - disabled  # Prevent postgres from starting
```

Then update `DATABASE_URL` in `.env` to point to your external database.

## Building

### First Build

```bash
docker compose up -d --build
```

The first build takes 5-10 minutes due to Rust compilation. Docker cache mounts are used to speed up subsequent builds -- only changed source files trigger recompilation.

### Rebuild After Code Changes

```bash
docker compose up -d --build
```

### Build Without Starting

```bash
docker compose build
```

### Clean Build (No Cache)

```bash
docker compose build --no-cache
```

## Container Health Check

The Chatalot container includes a built-in health check:

```dockerfile
HEALTHCHECK --interval=30s --timeout=10s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1
```

Check the health status:

```bash
docker inspect --format='{{json .State.Health.Status}}' chatalot-server
```

## Container User

The application runs as a non-root user (`chatalot:chatalot`) inside the container for security. The Dockerfile creates this user and sets appropriate file ownership.

## Common Commands

```bash
# Start all services
docker compose up -d

# Start with Cloudflare Tunnel
docker compose --profile production up -d

# Start with Quick Tunnel
docker compose --profile quick-tunnel up -d

# Stop all services
docker compose down

# Stop and remove volumes (DESTRUCTIVE -- deletes all data)
docker compose down -v

# View logs
docker compose logs -f chatalot
docker compose logs -f postgres

# Restart a single service
docker compose restart chatalot

# View resource usage
docker stats chatalot-server chatalot-db

# Execute a command in the running container
docker exec -it chatalot-server sh
docker exec -it chatalot-db psql -U chatalot
```

## Next Step

To build and run without Docker, see [Manual Deployment](./manual-deployment.md).
