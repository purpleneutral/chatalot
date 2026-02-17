# Desktop App

Chatalot includes a native desktop application built with [Tauri 2.0](https://v2.tauri.app/), providing a lightweight alternative to the web client with OS-level integration.

> **Status: Beta** -- The desktop app is functional but under active development. Some features may change between releases.

---

## Overview

The Chatalot desktop app wraps the same Svelte 5 web client in a native window using the system WebView, giving you the full Chatalot experience with added benefits like OS keychain integration, native notifications, and single-instance enforcement. Because it uses the platform's built-in WebView rather than bundling a full browser engine, the app is small (under 10 MB on Linux) and uses significantly less memory than running Chatalot in a browser tab.

## In This Section

- [Installation](installation.md) -- Download pre-built packages or build from source
- [Features](features.md) -- Desktop-specific features and advantages over the web client
- [Troubleshooting](troubleshooting.md) -- Common issues and how to resolve them

## Quick Start

1. Download the appropriate package for your platform (see [Installation](installation.md))
2. Launch Chatalot and enter your server URL on the Connect screen
3. Log in with your existing account or register a new one

The desktop app connects to any Chatalot server instance -- you choose which server to use when you first launch the app.

---

[Back to Documentation](../README.md)
