# What is Chatalot?

> **Status: Complete**

Chatalot is a self-hosted, end-to-end encrypted chat platform designed as a privacy-focused alternative to Discord.

![Chatalot login page](../../screenshots/00-login-page.png)

## Overview

Chatalot gives communities a place to communicate through text channels, voice calls, and direct messages -- all while keeping conversations private through end-to-end encryption (E2E). Because it is self-hosted, your organization controls the server, the data, and who has access.

## How It Compares to Discord

If you have used Discord before, Chatalot will feel familiar. The organizational hierarchy is similar:

| Discord | Chatalot | Description |
|---------|----------|-------------|
| Server | **Community** | The top-level container for your group |
| Category | **Group** | A folder-like grouping of channels within a community |
| Channel | **Channel** | A text or voice channel where conversations happen |
| Direct Message | **DM** | Private one-on-one conversations |

## Key Differences

- **Self-hosted.** You run the server on your own infrastructure. No third-party has access to your data.
- **End-to-end encrypted.** Messages are encrypted using the Signal protocol (X3DH key agreement + Double Ratchet). The server never sees plaintext message content.
- **No tracking.** There are no analytics, telemetry, or advertising. The platform exists to serve its users, not to monetize them.
- **Open source.** The entire codebase -- Rust server, Svelte web client, and Tauri desktop app -- is available for audit and contribution.

## Core Features

- **Communities, Groups, and Channels** -- Organize conversations with the same hierarchy you are used to from Discord.
- **Text channels** -- Rich messaging with Markdown formatting, file attachments, GIFs, emoji reactions, threads, pins, and polls.
- **Voice and video calls** -- WebRTC-based calls with microphone/camera controls, screen sharing, and noise suppression.
- **Direct messages** -- Private E2E encrypted conversations between two users.
- **Theming** -- Dark/light mode, multiple color palettes (Default, Monokai, Dracula, Nord, Solarized, AMOLED, Catppuccin), and 8 accent colors.
- **Keyboard shortcuts** -- Quick switcher (Ctrl+K), formatting shortcuts, and full keyboard navigation.
- **Desktop app** -- A native Tauri-based desktop application alongside the web client.
- **PWA support** -- Install Chatalot as a Progressive Web App on mobile and desktop.

## Next Step

Ready to dive in? Continue to [Creating an Account](./creating-an-account.md).
