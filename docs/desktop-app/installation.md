# Desktop App Installation

Download and install the Chatalot desktop application for your platform.

> **Status: Beta**

---

## Supported Platforms

| Platform | Architecture | Package Format | Status |
|----------|-------------|----------------|--------|
| Linux | x86_64, ARM64 | AppImage, .deb | Available |
| Windows | x86_64 | NSIS installer | Available |
| macOS | x86_64, ARM64 | .dmg | Planned |

## Download

Pre-built packages are available from the [GitHub Releases](https://github.com/purpleneutral/chatalot/releases) page. Download the appropriate file for your platform:

- **Linux (AppImage):** `Chatalot_x.x.x_amd64.AppImage` -- portable, no installation required
- **Linux (Debian/Ubuntu):** `Chatalot_x.x.x_amd64.deb` -- installs system-wide
- **Windows:** `Chatalot_x.x.x_x64-setup.exe` -- NSIS installer (installs per-user)

## System Requirements

### All Platforms

- A running Chatalot server instance to connect to
- Network access to the server (HTTPS recommended)

### Linux

- WebKitGTK 4.1 (`libwebkit2gtk-4.1-0`)
- OpenSSL 3 (`libssl3`)
- A running display server (X11 or Wayland)
- For keychain storage: GNOME Keyring or KWallet (Secret Service API)

### Windows

- Windows 10 version 1803 or later (WebView2 runtime required)
- WebView2 is pre-installed on Windows 10 1803+ and Windows 11

### macOS

- macOS 10.15 (Catalina) or later

## Installing on Linux

### AppImage

```bash
# Download the AppImage
chmod +x Chatalot_*.AppImage

# Run it
./Chatalot_*.AppImage
```

No installation is required. The AppImage is a self-contained executable.

### Debian / Ubuntu

```bash
sudo dpkg -i Chatalot_*_amd64.deb
```

If you encounter dependency errors:

```bash
sudo apt-get install -f
```

#### Dependencies (Debian/Ubuntu)

The .deb package declares these dependencies:

- `libwebkit2gtk-4.1-0`
- `libssl3`

Install them manually if needed:

```bash
sudo apt-get install libwebkit2gtk-4.1-0 libssl3
```

### Arch Linux

Install the WebKitGTK dependency:

```bash
sudo pacman -S webkit2gtk-4.1
```

Then run the AppImage, or extract and install manually.

### Fedora / RHEL

```bash
sudo dnf install webkit2gtk4.1 openssl
```

## Installing on Windows

Run the NSIS installer (`Chatalot_*_x64-setup.exe`). The installer:

- Installs to the current user's AppData directory (no admin required)
- Creates a Start Menu shortcut
- Registers an uninstaller

## Building from Source

If pre-built packages are not available for your platform, or you want to run the latest code, you can build from source.

### Prerequisites

1. **Rust toolchain** (stable, 1.85+):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (20+) and **npm**:
   ```bash
   # Using your system package manager, or:
   curl -fsSL https://fnm.vercel.app/install | bash
   fnm install 20
   ```

3. **Tauri CLI**:
   ```bash
   cargo install tauri-cli --version "^2"
   ```

4. **Platform-specific dependencies:**

   **Linux (Debian/Ubuntu):**
   ```bash
   sudo apt-get install libwebkit2gtk-4.1-dev libssl-dev \
     libayatana-appindicator3-dev librsvg2-dev \
     build-essential pkg-config
   ```

   **Linux (Arch):**
   ```bash
   sudo pacman -S webkit2gtk-4.1 base-devel openssl
   ```

   **Linux (Fedora):**
   ```bash
   sudo dnf install webkit2gtk4.1-devel openssl-devel \
     libappindicator-gtk3-devel librsvg2-devel
   ```

### Build Steps

```bash
# Clone the repository
git clone https://github.com/purpleneutral/chatalot.git
cd chatalot

# Install web client dependencies
cd clients/web
npm install

# Build the desktop app
cd ../desktop
cargo tauri build
```

The built packages will be in `clients/desktop/src-tauri/target/release/bundle/`.

### Development Mode

To run the desktop app in development mode with hot reload:

```bash
cd clients/desktop
cargo tauri dev
```

This starts the Svelte dev server on `http://localhost:5173` and opens the Tauri window pointing to it.

## First Launch

When you first launch the desktop app, you will see the **Connect** screen:

1. Enter the URL of your Chatalot server (e.g., `https://chat.example.com`)
2. The app validates the connection by checking the server's health endpoint
3. Once connected, you are redirected to the login/register screen
4. The server URL is saved and used automatically on future launches

To connect to a different server later, go to **Settings > Account** and use the server disconnect option.

## Updating

The desktop app includes built-in auto-update support. When a new version is available, a banner appears at the top of the chat window. Click it to download, install, and restart the app automatically.

If the auto-updater is unavailable (e.g., network issues), you can update manually:

1. Download the latest release from the GitHub Releases page
2. Install it over your existing installation (same process as initial install)

Your login session, preferences, and encryption keys are preserved across updates.

---

[Back to Desktop App](README.md) | [Back to Documentation](../README.md)
