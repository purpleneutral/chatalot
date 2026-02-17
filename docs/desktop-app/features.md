# Desktop App Features

What the desktop app offers beyond the web client, and how it takes advantage of native OS integration.

> **Status: Beta**

---

## Feature Comparison

| Feature | Web Client | Desktop App |
|---------|-----------|-------------|
| Full chat experience | Yes | Yes |
| Voice and video calls | Yes | Yes |
| Server connection | Automatic (same origin) | Manual (any server URL) |
| OS keychain integration | No | Yes |
| Native notifications | Browser API | OS-native via Tauri |
| Single-instance enforcement | No | Yes |
| External link handling | Same tab/new tab | Opens in default browser |
| Memory usage | Browser tab overhead | Lightweight (system WebView) |
| Installation | None (bookmark/PWA) | AppImage, .deb, or installer |
| Offline indicator | Browser-dependent | System-aware |

## OS Keychain Integration

The desktop app stores sensitive credentials in the operating system's native keychain rather than browser storage:

- **Linux:** Secret Service API (GNOME Keyring or KWallet)
- **Windows:** Credential Manager

This means your authentication tokens are protected by the OS-level credential store, which is encrypted and access-controlled. The keychain is accessed through the `keyring` crate with the service name `com.chatalot.app`.

Three operations are supported:
- **Store key** -- Save a named credential to the keychain
- **Get key** -- Retrieve a credential by name
- **Delete key** -- Remove a credential from the keychain

## Native Notifications

The desktop app uses the Tauri notification plugin for OS-native notifications. These integrate with your system's notification center rather than relying on the browser's Notification API:

- Notifications appear in your system tray/notification area
- They follow your OS notification preferences (Do Not Disturb, etc.)
- No browser permission prompt required -- the app requests notification permission through the OS

## Single-Instance Enforcement

The desktop app ensures only one instance runs at a time. If you try to launch Chatalot while it is already running, the existing window is focused instead of opening a duplicate. This prevents confusion from multiple windows connecting to the same server with the same account.

## Multi-Server Support

Unlike the web client (which connects to the server hosting it), the desktop app lets you connect to any Chatalot server:

1. On first launch, enter the server URL (e.g., `https://chat.example.com`)
2. The app validates the connection via the `/api/health` endpoint
3. HTTPS is assumed by default -- if you enter `chat.example.com`, it tries `https://chat.example.com`
4. The server URL is persisted in local storage for future launches

To switch servers, go to **Settings > Account** and disconnect from the current server.

## External Link Handling

When you click a URL in a message, the desktop app opens it in your default system browser via the Tauri shell plugin. This keeps the chat window focused while browsing links externally.

## Performance Advantages

The desktop app uses the system WebView (WebKitGTK on Linux, WebView2 on Windows) instead of bundling a full browser engine. This results in:

- **Smaller package size** -- Under 10 MB for the AppImage compared to hundreds of MB for Electron-based alternatives
- **Lower memory usage** -- The system WebView shares resources with other applications
- **Faster startup** -- No browser engine to initialize
- **Native look and feel** -- Window decorations and behavior match your desktop environment

## Window Configuration

The desktop app opens with these default settings:

| Property | Value |
|----------|-------|
| Default size | 1200 x 800 pixels |
| Minimum size | 800 x 600 pixels |
| Resizable | Yes |
| Centered on launch | Yes |

## Developer Tools

In debug builds, the Tauri DevTools are available for inspecting the web content. This is useful for development and troubleshooting but is not included in release builds.

## What Is Not Yet Available

The following features are planned but not yet implemented in the desktop app:

- **System tray icon** -- Minimize to tray with unread badge
- **Auto-update** -- Automatic download and installation of new versions
- **Global keyboard shortcuts** -- System-wide hotkeys (e.g., push-to-talk)
- **macOS support** -- Build infrastructure for macOS packages

---

[Back to Desktop App](README.md) | [Back to Documentation](../README.md)
