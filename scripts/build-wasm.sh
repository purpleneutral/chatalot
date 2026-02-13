#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CRATE_DIR="$SCRIPT_DIR/../crates/chatalot-crypto-wasm"
OUT_DIR="$SCRIPT_DIR/../clients/web/src/lib/crypto/wasm"

MODE="${1:---release}"

echo "Building WASM crypto module ($MODE)..."
cd "$CRATE_DIR"

if [ "$MODE" = "--dev" ]; then
    wasm-pack build --target web --out-dir "$OUT_DIR" --dev
else
    wasm-pack build --target web --out-dir "$OUT_DIR" --release
fi

# Remove the auto-generated package.json and .gitignore from wasm-pack output
rm -f "$OUT_DIR/package.json" "$OUT_DIR/.gitignore"

echo "WASM built successfully at $OUT_DIR"
