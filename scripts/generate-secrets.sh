#!/usr/bin/env bash
set -euo pipefail

# Generate secrets for Chatalot deployment
# Usage: ./scripts/generate-secrets.sh

SECRETS_DIR="$(cd "$(dirname "$0")/.." && pwd)/secrets"

mkdir -p "$SECRETS_DIR"

# Generate Ed25519 JWT signing keys
if [ -f "$SECRETS_DIR/jwt_private.pem" ]; then
    echo "JWT keys already exist in $SECRETS_DIR, skipping."
else
    openssl genpkey -algorithm Ed25519 -out "$SECRETS_DIR/jwt_private.pem"
    openssl pkey -in "$SECRETS_DIR/jwt_private.pem" -pubout -out "$SECRETS_DIR/jwt_public.pem"
    chmod 600 "$SECRETS_DIR/jwt_private.pem"
    chmod 644 "$SECRETS_DIR/jwt_public.pem"
    echo "Generated Ed25519 JWT keys in $SECRETS_DIR"
fi

# Generate .env from .env.example if it doesn't exist
ENV_FILE="$(cd "$(dirname "$0")/.." && pwd)/.env"
if [ -f "$ENV_FILE" ]; then
    echo ".env already exists, skipping."
else
    DB_PASSWORD=$(openssl rand -base64 32 | tr -d '/+=' | head -c 32)
    TOTP_KEY=$(openssl rand -hex 32)

    cat > "$ENV_FILE" <<EOF
DATABASE_URL=postgres://chatalot:${DB_PASSWORD}@postgres:5432/chatalot
DB_PASSWORD=${DB_PASSWORD}
JWT_PRIVATE_KEY_PATH=/run/secrets/jwt_private_key
JWT_PUBLIC_KEY_PATH=/run/secrets/jwt_public_key
TOTP_ENCRYPTION_KEY=${TOTP_KEY}
CLOUDFLARE_TUNNEL_TOKEN=your_tunnel_token_here
RUST_LOG=chatalot_server=info,tower_http=info
FILE_STORAGE_PATH=/app/data/files
MAX_FILE_SIZE_MB=100
LISTEN_ADDR=0.0.0.0:8080
EOF
    chmod 600 "$ENV_FILE"
    echo "Generated .env with random DB password and TOTP key"
    echo "  -> Edit CLOUDFLARE_TUNNEL_TOKEN if using Cloudflare Tunnel"
fi

echo ""
echo "Setup complete! Run with:"
echo "  docker compose up -d              # Development (no tunnel)"
echo "  docker compose --profile production up -d  # Production (with Cloudflare Tunnel)"
