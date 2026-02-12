#!/usr/bin/env bash
# Generate Ed25519 key pair for JWT signing
set -euo pipefail

SECRETS_DIR="$(dirname "$0")/../secrets"
mkdir -p "$SECRETS_DIR"

if [ -f "$SECRETS_DIR/jwt_private.pem" ]; then
    echo "Keys already exist in $SECRETS_DIR. Delete them first to regenerate."
    exit 0
fi

echo "Generating Ed25519 key pair..."
openssl genpkey -algorithm Ed25519 -out "$SECRETS_DIR/jwt_private.pem"
openssl pkey -in "$SECRETS_DIR/jwt_private.pem" -pubout -out "$SECRETS_DIR/jwt_public.pem"

chmod 600 "$SECRETS_DIR/jwt_private.pem"
chmod 644 "$SECRETS_DIR/jwt_public.pem"

echo "Keys generated:"
echo "  Private: $SECRETS_DIR/jwt_private.pem"
echo "  Public:  $SECRETS_DIR/jwt_public.pem"
