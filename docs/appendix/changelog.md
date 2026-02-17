# Changelog

All notable changes to Chatalot are documented here. Versions follow [Semantic Versioning](https://semver.org/).

---

## v0.17.0

- Add ARM64 support with multi-architecture Docker builds (amd64 + arm64)
- Add pre-built container images on GHCR
- Add CI workflow for automated multi-arch builds
- Add platform detection in install script

## v0.16.0

- Add threaded replies with thread panel UI
- Add read receipts with real-time broadcast
- Add privacy toggle for read receipts
- Thread panel: rich message rendering, hover actions, reactions, composer

## v0.15.0

- Add message edit history tracking
- Add edit history viewer UI
- Track old ciphertext/nonce on each edit

## v0.14.x

- Add search filters: sender, date range, file type
- Add scheduled messages with send-later UI and scheduled panel
- Add bookmarks ("Saved Items") panel
- Add bio and pronouns to user profiles and profile cards
- Accessibility improvements: ARIA labels throughout
- v0.14.1: Error feedback improvements, minor fixes

## v0.13.x

- Add announcements system (admin → all users)
- Add custom emoji (50 per community, PNG/GIF/WebP)
- Add content reporting system
- Add idle status tracking
- v0.13.1: Mark-all-read, confirmation dialogs, type fixes
- v0.13.2: Accessibility and code quality improvements
- v0.13.3: Responsive UI and server hardening

## v0.12.0

- Add polls (2-10 options, multi-select, anonymous, expiry)
- Add webhooks (per-channel, configurable name/avatar)

## v0.11.0

- Add theme customization: 8 color palettes, 8 accent colors
- Add message density options (cozy/compact)
- Add font size settings (small/medium/large)
- Add custom theme support

## v0.10.0

- Add permissions system for groups, channels, and communities
- Add group and channel settings popovers
- Add community policies (who can create groups, who can create invites)

## v0.9.0

- Add mobile-responsive layout
- Add touch-friendly UI components
- Add adaptive media sizing for small screens

## v0.8.0

- Add preset themes: Default, Monokai, Dracula, Nord, Solarized, AMOLED, Catppuccin
- Add custom theme editor
- Add bubble-style message layout option
- Add relative timestamps
- Add reduce-motion accessibility option

## v0.7.0

- Add invite link system with codes, expiry, and usage limits
- Add WebSocket connection cleanup and reconnection
- Add accessibility improvements
- Security hardening: rate limiting, input validation

## v0.5.x

- Add voice channel volume amplification (0-500%)
- Add screen sharing with audio pipeline
- Fix screen share context menu and audio capture
- Add PipeWire system audio auto-capture

## v0.1.0

Initial release.

- Communities, groups, and channels
- Text messaging with Markdown formatting
- Voice and video calls (WebRTC full-mesh)
- Direct messages
- End-to-end encryption infrastructure (X3DH + Double Ratchet + Sender Keys)
- File uploads with previews
- Emoji reactions
- User profiles with avatars, status, and presence
- Admin panel (users, invites, files, reports, audit log)
- Moderation tools (warn, timeout, kick, ban)
- Dark/light theme with system preference detection
- Docker deployment with PostgreSQL
- PWA support

---

## Versioning Policy

Chatalot uses semantic versioning:
- **Major** (x.0.0): Breaking changes to the API or database schema
- **Minor** (0.x.0): New features, non-breaking changes
- **Patch** (0.0.x): Bug fixes, minor improvements

Database migrations are applied automatically on server startup. Breaking schema changes will include migration scripts.

## Related Pages

- [Feature Status](./feature-status.md) -- Current implementation status of all features
- [FAQ](./faq.md) -- Frequently asked questions
