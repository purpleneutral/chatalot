# Desktop App Troubleshooting

Common issues with the Chatalot desktop app and how to resolve them.

> **Status: Beta**

---

## Installation Issues

### Linux: "libwebkit2gtk" not found

The Chatalot desktop app requires WebKitGTK 4.1. Install it for your distribution:

**Debian / Ubuntu:**
```bash
sudo apt-get install libwebkit2gtk-4.1-0
```

**Arch Linux:**
```bash
sudo pacman -S webkit2gtk-4.1
```

**Fedora:**
```bash
sudo dnf install webkit2gtk4.1
```

### Linux: AppImage fails to launch

1. Ensure the AppImage has execute permission:
   ```bash
   chmod +x Chatalot_*.AppImage
   ```

2. If you see FUSE-related errors, install FUSE:
   ```bash
   # Debian/Ubuntu
   sudo apt-get install fuse libfuse2

   # Arch
   sudo pacman -S fuse2
   ```

3. Alternatively, extract and run directly:
   ```bash
   ./Chatalot_*.AppImage --appimage-extract
   ./squashfs-root/AppRun
   ```

### Windows: WebView2 missing

The app requires Microsoft WebView2 Runtime. It is pre-installed on Windows 10 1803+ and Windows 11. If missing:

1. Download the WebView2 Runtime from [Microsoft's website](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
2. Install the Evergreen Bootstrapper
3. Restart Chatalot

### Windows: "Windows protected your PC" SmartScreen warning

Since the app is not code-signed with an EV certificate, Windows SmartScreen may block it:

1. Click "More info"
2. Click "Run anyway"

This only appears on the first launch.

## Connection Issues

### "Could not reach server"

This error appears on the Connect screen when the app cannot communicate with your Chatalot server.

1. **Check the URL:** Ensure you entered the correct server address (e.g., `https://chat.example.com`, not `https://chat.example.com/login`)
2. **Check the protocol:** The app defaults to HTTPS. If your server uses HTTP (not recommended), enter the full URL: `http://chat.example.com`
3. **Check your network:** Ensure you can reach the server from your machine (try opening the URL in a browser)
4. **Check the server:** Verify the Chatalot server is running and the `/api/health` endpoint returns `{"status":"ok"}`

### "Connection timed out"

The app waits 10 seconds for a response from the server. If it times out:

- The server may be under heavy load
- There may be a network issue between you and the server
- A firewall may be blocking the connection

### WebSocket connection drops

If the app connects but frequently loses the real-time connection:

1. Check if a proxy or firewall is terminating WebSocket connections
2. Ensure your server supports WebSocket upgrades (if behind a reverse proxy, configure it for WebSocket pass-through)
3. The app will automatically attempt to reconnect -- if it consistently fails, check the server logs

## Display Issues

### Blank white screen on launch

This typically indicates the web client assets failed to load:

1. The Tauri app loads the web client from the built Svelte output. If you built from source, ensure the web client was built first:
   ```bash
   cd clients/web && npm run build
   ```
2. Try clearing the app's WebView cache (see "Clearing App Data" below)

### UI looks wrong or outdated

The system WebView's rendering engine determines how CSS and JavaScript are processed. Ensure your system WebView is up to date:

**Linux:** Update WebKitGTK through your package manager:
```bash
# Debian/Ubuntu
sudo apt-get upgrade libwebkit2gtk-4.1-0

# Arch
sudo pacman -Syu webkit2gtk-4.1
```

**Windows:** WebView2 updates automatically through Windows Update.

### Scaling issues on HiDPI displays

If the app appears too small or too large:

- **Linux:** Set the `GDK_SCALE` environment variable:
  ```bash
  GDK_SCALE=2 ./Chatalot.AppImage
  ```
- **Windows:** Right-click the app shortcut > Properties > Compatibility > Change high DPI settings

## Keychain Issues

### Linux: "No keyring available" or credential storage fails

The desktop app stores credentials in the OS keychain via the Secret Service API. If this fails:

1. Ensure GNOME Keyring or KWallet is installed and running:
   ```bash
   # GNOME Keyring
   sudo apt-get install gnome-keyring

   # KWallet (KDE)
   sudo apt-get install kwalletmanager
   ```

2. If running a minimal desktop environment without a keyring daemon, start one manually:
   ```bash
   eval $(gnome-keyring-daemon --start --components=secrets)
   ```

3. The app will fall back gracefully -- you may need to log in again after each restart if the keychain is unavailable.

### Windows: credential storage fails

Ensure the Windows Credential Manager service is running:

1. Press Win+R, type `services.msc`
2. Find "Credential Manager" and ensure it is set to "Automatic" and running

## Audio and Video Issues

### Microphone or camera not detected

WebView permissions for media devices work differently than in a browser:

1. Ensure your desktop environment grants the app access to audio/video devices
2. On Linux with PipeWire, ensure the WebView has access to PipeWire sockets
3. Check that no other application has exclusive access to the device

### Screen sharing not available

Screen sharing via `getDisplayMedia` depends on the WebView's capabilities:

- **Linux (WebKitGTK):** Screen sharing support depends on the WebKitGTK version and your compositor. Ensure you are running a recent version (2.40+) with PipeWire support.
- **Windows (WebView2):** Screen sharing should work out of the box.

## Clearing App Data

If you need to reset the desktop app to a clean state:

### Linux

App data is stored in the XDG data directory:

```bash
# Remove app data
rm -rf ~/.local/share/com.chatalot.desktop/

# Remove WebView cache
rm -rf ~/.cache/com.chatalot.desktop/

# Remove keychain entries (optional)
secret-tool clear service com.chatalot.app
```

### Windows

App data is stored in the user's AppData directory:

```
%LOCALAPPDATA%\com.chatalot.desktop\
```

Delete this folder to clear all app data and cache.

## Reporting Bugs

If you encounter an issue not covered here:

1. Check the [GitHub Issues](https://github.com/purpleneutral/chatalot/issues) page for known problems
2. Use the in-app feedback form (available from the sidebar) to report the issue
3. Include your operating system, desktop environment, and WebView version when reporting

---

[Back to Desktop App](README.md) | [Back to Documentation](../README.md)
