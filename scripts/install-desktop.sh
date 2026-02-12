#!/usr/bin/env bash
set -euo pipefail

# Chatalot Desktop Installer
# Downloads and installs the latest Chatalot desktop client

INSTALL_DIR="${HOME}/.local/bin"
APP_NAME="chatalot"

# Detect OS
case "$(uname -s)" in
    Linux)  OS="linux" ;;
    *)      echo "Error: This installer only supports Linux."; echo "For Windows, download the .exe installer from the releases page."; exit 1 ;;
esac

# Detect architecture
case "$(uname -m)" in
    x86_64|amd64) ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *)             echo "Error: Unsupported architecture $(uname -m)"; exit 1 ;;
esac

echo "Chatalot Desktop Installer"
echo "=========================="
echo "OS: ${OS} (${ARCH})"
echo ""

# Check for required tools
if ! command -v curl &>/dev/null; then
    echo "Error: curl is required but not installed."
    exit 1
fi

# Determine download URL
# Update this URL when hosting changes (e.g., to GitHub releases)
RELEASE_URL="${CHATALOT_RELEASE_URL:-}"
if [ -z "${RELEASE_URL}" ]; then
    echo "Error: Set CHATALOT_RELEASE_URL to the URL of the AppImage file."
    echo ""
    echo "Example:"
    echo "  CHATALOT_RELEASE_URL=https://git.example.com/user/chatalot/releases/download/v0.1.0/chatalot_0.1.0_amd64.AppImage \\"
    echo "    bash install-desktop.sh"
    exit 1
fi

# Create install directory if needed
mkdir -p "${INSTALL_DIR}"

# Download
TMPFILE=$(mktemp)
echo "Downloading from ${RELEASE_URL}..."
curl -fSL "${RELEASE_URL}" -o "${TMPFILE}"

# Install
chmod +x "${TMPFILE}"
mv "${TMPFILE}" "${INSTALL_DIR}/${APP_NAME}"

echo ""
echo "Installed to ${INSTALL_DIR}/${APP_NAME}"

# Check if install dir is in PATH
if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
    echo ""
    echo "Note: ${INSTALL_DIR} is not in your PATH."
    echo "Add this to your shell profile:"
    echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
fi

echo ""
echo "Run '${APP_NAME}' to start Chatalot."
