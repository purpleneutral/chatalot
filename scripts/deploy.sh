#!/usr/bin/env bash
set -euo pipefail

# Chatalot deploy script
# Commits locally, pushes to git remote, pulls on server, and starts containers.
#
# Configuration (set these for your environment):
#   DEPLOY_HOST    - SSH target (e.g. user@your-server)
#   DEPLOY_DIR     - Remote directory (e.g. /srv/chatalot)
#   DEPLOY_GIT_URL - Git remote URL (e.g. ssh://git@github.com/you/chatalot.git)
#   DEPLOY_DOMAIN  - Your domain for Traefik routing (e.g. chat.example.com)
#   DEPLOY_NETWORK - Docker network for Traefik (e.g. web)
#
# Usage:
#   ./scripts/deploy.sh                  # Commit, push, pull, restart
#   ./scripts/deploy.sh "commit message" # Custom commit message
#   ./scripts/deploy.sh --pull-only      # Only pull + restart on server (no commit/push)

REMOTE_HOST="${DEPLOY_HOST:?Set DEPLOY_HOST (e.g. user@your-server)}"
REMOTE_DIR="${DEPLOY_DIR:-/srv/chatalot}"
GIT_REMOTE="${DEPLOY_GIT_REMOTE:-origin}"
GIT_REMOTE_URL="${DEPLOY_GIT_URL:?Set DEPLOY_GIT_URL (e.g. ssh://git@github.com/you/chatalot.git)}"
BRANCH="${DEPLOY_BRANCH:-master}"
DOMAIN="${DEPLOY_DOMAIN:-}"
NETWORK="${DEPLOY_NETWORK:-web}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[deploy]${NC} $*"; }
warn()  { echo -e "${YELLOW}[deploy]${NC} $*"; }
error() { echo -e "${RED}[deploy]${NC} $*" >&2; }

cd "$(git rev-parse --show-toplevel)"

# --- Step 1: Commit & push (unless --pull-only) ---
if [[ "${1:-}" != "--pull-only" ]]; then
    COMMIT_MSG="${1:-deploy: $(date +%Y-%m-%d\ %H:%M:%S)}"

    # Ensure remote exists
    if ! git remote get-url "$GIT_REMOTE" &>/dev/null; then
        info "Adding git remote '$GIT_REMOTE' -> $GIT_REMOTE_URL"
        git remote add "$GIT_REMOTE" "$GIT_REMOTE_URL"
    fi

    # Stage all tracked + new files
    info "Staging changes..."
    git add -A

    # Check if there's anything to commit
    if git diff --cached --quiet 2>/dev/null; then
        warn "No changes to commit, pushing existing commits..."
    else
        info "Committing: $COMMIT_MSG"
        git commit -m "$COMMIT_MSG"
    fi

    info "Pushing to $GIT_REMOTE/$BRANCH..."
    git push -u "$GIT_REMOTE" "$BRANCH"
fi

# --- Step 2: Pull & restart on server ---
info "Deploying to $REMOTE_HOST:$REMOTE_DIR..."

ssh "$REMOTE_HOST" bash -s "$REMOTE_DIR" "$GIT_REMOTE_URL" "$BRANCH" "${DOMAIN:-__NONE__}" "$NETWORK" <<'REMOTE_SCRIPT'
set -euo pipefail
REMOTE_DIR="$1"
GIT_URL="$2"
BRANCH="$3"
DOMAIN="${4:-}"
[ "$DOMAIN" = "__NONE__" ] && DOMAIN=""
NETWORK="${5:-web}"

echo "[server] Starting deploy..."

# Clone if first deploy, otherwise pull
if [ ! -d "$REMOTE_DIR/.git" ]; then
    echo "[server] First deploy — cloning..."
    git clone "$GIT_URL" "$REMOTE_DIR"
    cd "$REMOTE_DIR"
    git checkout "$BRANCH"
else
    cd "$REMOTE_DIR"
    echo "[server] Pulling latest..."
    git fetch origin
    git reset --hard "origin/$BRANCH"
fi

# Generate secrets if missing
if [ ! -f secrets/jwt_private.pem ]; then
    echo "[server] Generating JWT keys..."
    mkdir -p secrets
    openssl genpkey -algorithm Ed25519 -out secrets/jwt_private.pem
    openssl pkey -in secrets/jwt_private.pem -pubout -out secrets/jwt_public.pem
    chmod 600 secrets/jwt_private.pem
    chmod 644 secrets/jwt_public.pem
fi

# Generate .env if missing
if [ ! -f .env ]; then
    echo "[server] Generating .env..."
    DB_PASSWORD=$(openssl rand -base64 32 | tr -d '/+=' | head -c 32)
    TOTP_KEY=$(openssl rand -hex 32)
    cat > .env <<EOF
DATABASE_URL=postgres://chatalot:${DB_PASSWORD}@postgres:5432/chatalot
DB_PASSWORD=${DB_PASSWORD}
JWT_PRIVATE_KEY_PATH=/run/secrets/jwt_private_key
JWT_PUBLIC_KEY_PATH=/run/secrets/jwt_public_key
TOTP_ENCRYPTION_KEY=${TOTP_KEY}
RUST_LOG=chatalot_server=info,tower_http=info
FILE_STORAGE_PATH=/app/data/files
MAX_FILE_SIZE_MB=100
LISTEN_ADDR=0.0.0.0:8080
STATIC_FILES_PATH=/app/static
EOF
    chmod 600 .env
fi

# Create docker-compose.override.yml for reverse proxy routing (server-specific, not in git)
if [ -n "$DOMAIN" ] && [ ! -f docker-compose.override.yml ]; then
    echo "[server] Creating Traefik override ($DOMAIN)..."
    cat > docker-compose.override.yml <<OVERRIDE
services:
  chatalot:
    labels:
      - "traefik.enable=true"
      - "traefik.docker.network=$NETWORK"
      - "traefik.http.routers.chatalot.rule=Host(\`$DOMAIN\`)"
      - "traefik.http.routers.chatalot.entrypoints=websecure"
      - "traefik.http.routers.chatalot.tls=true"
      - "traefik.http.services.chatalot.loadbalancer.server.port=8080"
    networks:
      - $NETWORK

networks:
  $NETWORK:
    external: true
OVERRIDE
fi

# Build and start (nohup so it survives SSH disconnect on long builds)
echo "[server] Building and starting containers (logging to /tmp/chatalot-build.log)..."
nohup docker compose up -d --build > /tmp/chatalot-build.log 2>&1 &
BUILD_PID=$!
echo "[server] Build started (PID $BUILD_PID), waiting..."

# Wait for the build to finish (check every 10s)
while kill -0 "$BUILD_PID" 2>/dev/null; do
    sleep 10
    # Show last line of progress
    tail -1 /tmp/chatalot-build.log 2>/dev/null | head -c 120
    echo
done

# Check exit code
wait "$BUILD_PID"
BUILD_EXIT=$?
if [ "$BUILD_EXIT" -ne 0 ]; then
    echo "[server] BUILD FAILED (exit $BUILD_EXIT). Last 50 lines:"
    tail -50 /tmp/chatalot-build.log
    exit 1
fi

echo "[server] Build complete. Waiting for health check..."
for i in $(seq 1 30); do
    if curl -sf http://localhost:8080/api/health >/dev/null 2>&1; then
        echo "[server] Health check passed!"
        docker compose ps
        exit 0
    fi
    sleep 2
done

echo "[server] WARNING: Health check did not pass within 60s"
docker compose logs --tail=50
exit 1
REMOTE_SCRIPT

info "Deploy complete!"
