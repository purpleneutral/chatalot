#!/usr/bin/env bash
set -euo pipefail

# ──────────────────────────────────────────────────────────────
# Chatalot Interactive Setup
# Sets up a new Chatalot instance with guided prompts.
# Usage: ./scripts/install.sh
# ──────────────────────────────────────────────────────────────

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SECRETS_DIR="$PROJECT_DIR/secrets"
ENV_FILE="$PROJECT_DIR/.env"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

print_banner() {
    echo ""
    echo -e "${CYAN}${BOLD}"
    echo "   _____ _           _        _       _   "
    echo "  / ____| |         | |      | |     | |  "
    echo " | |    | |__   __ _| |_ __ _| | ___ | |_ "
    echo " | |    | '_ \\ / _\` | __/ _\` | |/ _ \\| __|"
    echo " | |____| | | | (_| | || (_| | | (_) | |_ "
    echo "  \\_____|_| |_|\\__,_|\\__\\__,_|_|\\___/ \\__|"
    echo -e "${NC}"
    echo -e "${BOLD}  Your chat. Your server. Your rules.${NC}"
    echo ""
}

print_step() {
    echo -e "\n${BLUE}${BOLD}[$1/$TOTAL_STEPS]${NC} ${BOLD}$2${NC}"
}

print_success() {
    echo -e "  ${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "  ${YELLOW}!${NC} $1"
}

print_error() {
    echo -e "  ${RED}✗${NC} $1"
}

prompt() {
    local prompt_text="$1"
    local default="${2:-}"
    local result

    if [ -n "$default" ]; then
        echo -ne "  ${prompt_text} ${CYAN}[${default}]${NC}: "
    else
        echo -ne "  ${prompt_text}: "
    fi
    read -r result
    echo "${result:-$default}"
}

prompt_yes_no() {
    local prompt_text="$1"
    local default="${2:-n}"
    local result

    if [ "$default" = "y" ]; then
        echo -ne "  ${prompt_text} ${CYAN}[Y/n]${NC}: "
    else
        echo -ne "  ${prompt_text} ${CYAN}[y/N]${NC}: "
    fi
    read -r result
    result="${result:-$default}"
    [[ "$result" =~ ^[Yy] ]]
}

TOTAL_STEPS=6

# ──────────────────────────────────────────────────────────────
print_banner

echo -e "This script will walk you through setting up your own"
echo -e "Chatalot instance. It takes about 2 minutes."
echo ""
echo -e "Press ${BOLD}Enter${NC} to begin, or ${BOLD}Ctrl+C${NC} to cancel."
read -r

# ──────────────────────────────────────────────────────────────
# Step 1: Check prerequisites
# ──────────────────────────────────────────────────────────────
print_step 1 "Checking prerequisites"

MISSING=()

if command -v docker &>/dev/null; then
    DOCKER_VERSION=$(docker --version | grep -oP '\d+\.\d+\.\d+' | head -1)
    print_success "Docker $DOCKER_VERSION"
else
    print_error "Docker not found"
    MISSING+=("docker")
fi

if docker compose version &>/dev/null 2>&1; then
    COMPOSE_VERSION=$(docker compose version --short 2>/dev/null || echo "unknown")
    print_success "Docker Compose $COMPOSE_VERSION"
else
    print_error "Docker Compose v2 not found"
    MISSING+=("docker-compose")
fi

if command -v openssl &>/dev/null; then
    OPENSSL_VERSION=$(openssl version | awk '{print $2}')
    print_success "OpenSSL $OPENSSL_VERSION"
else
    print_error "OpenSSL not found"
    MISSING+=("openssl")
fi

if command -v git &>/dev/null; then
    GIT_VERSION=$(git --version | awk '{print $3}')
    print_success "Git $GIT_VERSION"
else
    print_warning "Git not found (optional, needed for updates)"
fi

if [ ${#MISSING[@]} -gt 0 ]; then
    echo ""
    print_error "Missing required tools: ${MISSING[*]}"
    echo ""
    echo "  Install them and run this script again."
    echo "  On Ubuntu/Debian: sudo apt install docker.io docker-compose-v2 openssl"
    echo "  On Fedora/RHEL:   sudo dnf install docker docker-compose openssl"
    echo "  On Arch:          sudo pacman -S docker docker-compose openssl"
    exit 1
fi

# Check if docker daemon is running
if ! docker info &>/dev/null 2>&1; then
    print_error "Docker daemon is not running"
    echo ""
    echo "  Start it with: sudo systemctl start docker"
    echo "  To start on boot: sudo systemctl enable docker"
    exit 1
fi

print_success "All prerequisites met"

# ──────────────────────────────────────────────────────────────
# Step 2: Configure your instance
# ──────────────────────────────────────────────────────────────
print_step 2 "Configure your instance"

echo ""
echo -e "  ${BOLD}Admin account${NC}"
echo -e "  The first user you register will become the admin."
echo -e "  Optionally, you can pre-set the admin username here."
echo ""

ADMIN_USERNAME=$(prompt "Admin username (leave blank to set later)" "")

echo ""
echo -e "  ${BOLD}Registration mode${NC}"
echo -e "  ${CYAN}open${NC}        — anyone can register (recommended for initial setup)"
echo -e "  ${CYAN}invite_only${NC} — users need an invite code (recommended after setup)"
echo -e "  ${CYAN}closed${NC}      — registration disabled"
echo ""

REG_MODE=$(prompt "Registration mode" "open")
case "$REG_MODE" in
    open|invite_only|closed) ;;
    *)
        print_warning "Invalid mode '$REG_MODE', defaulting to 'open'"
        REG_MODE="open"
        ;;
esac

echo ""
echo -e "  ${BOLD}Server port${NC}"
echo -e "  The port Chatalot will listen on."
echo ""

SERVER_PORT=$(prompt "Port" "8080")

echo ""
echo -e "  ${BOLD}Max upload size${NC}"
echo -e "  Maximum file upload size in megabytes."
echo ""

MAX_FILE_SIZE=$(prompt "Max file size (MB)" "100")

echo ""
echo -e "  ${BOLD}Internet access${NC}"
echo -e "  How do you want to expose your instance?"
echo -e "  ${CYAN}1${NC} — Local only (http://localhost:$SERVER_PORT)"
echo -e "  ${CYAN}2${NC} — Cloudflare Quick Tunnel (free temporary public URL)"
echo -e "  ${CYAN}3${NC} — Cloudflare Named Tunnel (persistent domain, requires token)"
echo -e "  ${CYAN}4${NC} — Reverse proxy (Traefik/nginx/Caddy — configure separately)"
echo ""

EXPOSE_MODE=$(prompt "Choose" "1")
CF_TOKEN=""
COMPOSE_PROFILE=""

case "$EXPOSE_MODE" in
    2) COMPOSE_PROFILE="quick-tunnel" ;;
    3)
        echo ""
        CF_TOKEN=$(prompt "Cloudflare Tunnel token" "")
        if [ -z "$CF_TOKEN" ]; then
            print_warning "No token provided, falling back to local only"
            EXPOSE_MODE="1"
        else
            COMPOSE_PROFILE="production"
        fi
        ;;
    4)
        echo ""
        print_warning "Remember to configure your reverse proxy to forward to port $SERVER_PORT"
        print_warning "WebSocket connections at /ws must be proxied as well"
        ;;
esac

echo ""
echo -e "  ${BOLD}GIF search (optional)${NC}"
echo -e "  Chatalot supports inline GIF search via Google Tenor API."
echo -e "  Get a free API key at: https://developers.google.com/tenor/guides/quickstart"
echo ""

TENOR_KEY=$(prompt "Tenor API key (leave blank to skip)" "")

# ──────────────────────────────────────────────────────────────
# Step 3: Generate secrets
# ──────────────────────────────────────────────────────────────
print_step 3 "Generating secrets"

mkdir -p "$SECRETS_DIR"

if [ -f "$SECRETS_DIR/jwt_private.pem" ]; then
    print_warning "JWT keys already exist, keeping existing keys"
else
    openssl genpkey -algorithm Ed25519 -out "$SECRETS_DIR/jwt_private.pem" 2>/dev/null
    openssl pkey -in "$SECRETS_DIR/jwt_private.pem" -pubout -out "$SECRETS_DIR/jwt_public.pem" 2>/dev/null
    chmod 600 "$SECRETS_DIR/jwt_private.pem"
    chmod 644 "$SECRETS_DIR/jwt_public.pem"
    print_success "Generated Ed25519 JWT signing keys"
fi

# ──────────────────────────────────────────────────────────────
# Step 4: Create .env
# ──────────────────────────────────────────────────────────────
print_step 4 "Creating configuration"

if [ -f "$ENV_FILE" ]; then
    if prompt_yes_no "  .env already exists. Overwrite?" "n"; then
        rm "$ENV_FILE"
    else
        print_warning "Keeping existing .env"
    fi
fi

if [ ! -f "$ENV_FILE" ]; then
    DB_PASSWORD=$(openssl rand -base64 32 | tr -d '/+=' | head -c 32)
    TOTP_KEY=$(openssl rand -hex 32)

    cat > "$ENV_FILE" <<EOF
# Chatalot Configuration
# Generated by install.sh on $(date -u +"%Y-%m-%d %H:%M:%S UTC")

# Database (auto-generated, do not change unless you know what you're doing)
DATABASE_URL=postgres://chatalot:${DB_PASSWORD}@postgres:5432/chatalot
DB_PASSWORD=${DB_PASSWORD}

# JWT signing keys (paths inside Docker container)
JWT_PRIVATE_KEY_PATH=/run/secrets/jwt_private_key
JWT_PUBLIC_KEY_PATH=/run/secrets/jwt_public_key

# 2FA encryption key (auto-generated)
TOTP_ENCRYPTION_KEY=${TOTP_KEY}

# Server
LISTEN_ADDR=0.0.0.0:8080
RUST_LOG=chatalot_server=info,tower_http=info
FILE_STORAGE_PATH=/app/data/files
MAX_FILE_SIZE_MB=${MAX_FILE_SIZE}

# Registration
REGISTRATION_MODE=${REG_MODE}
ADMIN_USERNAME=${ADMIN_USERNAME}

# Cloudflare Tunnel (leave empty if not using)
CLOUDFLARE_TUNNEL_TOKEN=${CF_TOKEN}

# GIF search (optional — https://developers.google.com/tenor/guides/quickstart)
TENOR_API_KEY=${TENOR_KEY}
EOF

    chmod 600 "$ENV_FILE"
    print_success "Created .env with secure random credentials"
fi

# ──────────────────────────────────────────────────────────────
# Step 5: Update docker-compose port if custom
# ──────────────────────────────────────────────────────────────
print_step 5 "Preparing Docker"

OVERRIDE_FILE="$PROJECT_DIR/docker-compose.override.yml"
if [ "$SERVER_PORT" != "8080" ] && [ ! -f "$OVERRIDE_FILE" ]; then
    cat > "$OVERRIDE_FILE" <<EOF
# Auto-generated by install.sh — custom port mapping
services:
  chatalot:
    ports:
      - "${SERVER_PORT}:8080"
EOF
    print_success "Configured custom port: $SERVER_PORT"
else
    print_success "Using default port: $SERVER_PORT"
fi

# ──────────────────────────────────────────────────────────────
# Step 6: Build and start
# ──────────────────────────────────────────────────────────────
print_step 6 "Starting Chatalot"

echo ""
echo -e "  This will build the Docker images and start the containers."
echo -e "  The first build takes ${BOLD}5-10 minutes${NC} (compiling Rust + WASM)."
echo -e "  Subsequent starts are nearly instant."
echo ""

if ! prompt_yes_no "Start Chatalot now?" "y"; then
    echo ""
    echo -e "  No problem. When you're ready, run:"
    if [ -n "$COMPOSE_PROFILE" ]; then
        echo -e "    ${CYAN}docker compose --profile $COMPOSE_PROFILE up -d${NC}"
    else
        echo -e "    ${CYAN}docker compose up -d${NC}"
    fi
    exit 0
fi

echo ""
echo -e "  Building and starting containers... (this takes a while on first run)"
echo ""

cd "$PROJECT_DIR"

if [ -n "$COMPOSE_PROFILE" ]; then
    docker compose --profile "$COMPOSE_PROFILE" up -d --build 2>&1 | while IFS= read -r line; do
        echo "  $line"
    done
else
    docker compose up -d --build 2>&1 | while IFS= read -r line; do
        echo "  $line"
    done
fi

# Wait for health check
echo ""
echo -e "  Waiting for Chatalot to be ready..."

MAX_WAIT=120
WAITED=0
while [ $WAITED -lt $MAX_WAIT ]; do
    if curl -sf http://localhost:8080/api/health >/dev/null 2>&1; then
        break
    fi
    sleep 2
    WAITED=$((WAITED + 2))
    echo -ne "\r  Waiting... ${WAITED}s"
done
echo ""

if curl -sf http://localhost:8080/api/health >/dev/null 2>&1; then
    print_success "Chatalot is running!"
else
    print_error "Health check failed after ${MAX_WAIT}s"
    echo ""
    echo "  Check the logs with: docker compose logs chatalot"
    exit 1
fi

# Quick tunnel URL
if [ "$EXPOSE_MODE" = "2" ]; then
    echo ""
    echo -e "  Fetching your public URL..."
    sleep 5
    TUNNEL_URL=$(docker compose logs cloudflared-quick 2>/dev/null | grep -oP 'https://[a-z0-9-]+\.trycloudflare\.com' | tail -1)
    if [ -n "$TUNNEL_URL" ]; then
        print_success "Public URL: $TUNNEL_URL"
    else
        print_warning "Couldn't detect tunnel URL. Check with:"
        echo "    docker compose logs cloudflared-quick | grep trycloudflare"
    fi
fi

# ──────────────────────────────────────────────────────────────
# Done!
# ──────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}${BOLD}  Setup complete!${NC}"
echo ""
echo -e "  ${BOLD}Next steps:${NC}"
echo ""
echo -e "  1. Open ${CYAN}http://localhost:${SERVER_PORT}${NC} in your browser"
echo -e "  2. Register your admin account"
if [ "$REG_MODE" = "open" ]; then
    echo -e "  3. Switch to invite-only registration:"
    echo -e "     Edit ${CYAN}.env${NC} and set ${CYAN}REGISTRATION_MODE=invite_only${NC}"
    echo -e "     Then run: ${CYAN}docker compose up -d${NC}"
fi
echo -e "  4. Generate invite codes from the admin panel to invite friends"
echo ""
echo -e "  ${BOLD}Save your recovery code!${NC} It's shown once at registration."
echo -e "  It's the only way to reset your password without admin help."
echo ""
echo -e "  ${BOLD}Useful commands:${NC}"
echo -e "    ${CYAN}docker compose logs -f chatalot${NC}  — view server logs"
echo -e "    ${CYAN}docker compose down${NC}              — stop everything"
echo -e "    ${CYAN}docker compose up -d${NC}             — start again"
echo -e "    ${CYAN}docker compose pull && docker compose up -d --build${NC}  — update"
echo ""
echo -e "  ${BOLD}Questions?${NC} https://github.com/purpleneutral/chatalot"
echo ""
