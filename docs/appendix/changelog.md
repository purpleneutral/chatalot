# Changelog

All notable changes to Chatalot are documented here. Versions follow [Semantic Versioning](https://semver.org/).

---

## v0.23.0

### Added
- **OIDC/SSO support** — login via Authentik, Keycloak, or any OpenID Connect provider
- OIDC user provisioning (auto-create accounts on first SSO login)
- Option to disable password login when OIDC is enabled (`OIDC_DISABLE_PASSWORD_LOGIN`)
- CI workflow for automated testing on push/PR (cargo check, clippy, frontend build)
- Root-level `SECURITY.md`, `CONTRIBUTING.md`, and `CHANGELOG.md`
- TURN/coturn documentation in self-hosting guide
- Build hash cache-busting for static assets

### Fixed
- CSP no longer hardcodes specific domains — WebSocket origins derived from `PUBLIC_URL`
- CORS fallback no longer leaks author's domain — derived from `PUBLIC_URL`
- Fixed `og:image` using absolute URL (now relative)
- Version sync across all manifests (workspace, desktop, frontend)

### Security
- Removed internal homelab hostnames from CSP headers
- Added responsible disclosure policy (`SECURITY.md`)

---

## v0.22.6

### Added
- **UI zoom** (50–200%) with keyboard shortcuts (Ctrl+/Ctrl-), synced across tabs

### Fixed
- Sidebar tab not remembering last selection
- Removed dead Tauri bridge iframe code

---

## v0.22.5

### Fixed
- All TypeScript errors resolved across the frontend codebase
- Removed dead shell/console feature code

---

## v0.21.0

### Added
- **Gallery channels** with image thumbnails and lightbox viewer
- Automated backup script with remote pg_dump and file sync
- GitHub Action to auto-update AUR package on release
- `.desktop` file categories for Linux packaging

### Fixed
- 5 WebRTC video/screen sharing bugs
- WebRTC memory leak in voice connections
- Voice rate limiting and broadcast channel hardening
- ICE candidates silently dropped (voice calls failing)
- Voice auto-kick race condition with multiple tabs
- RwLock crash risk in concurrent access paths
- Group invite permissions for instance owner
- 3-person mesh call connectivity
- Desktop app auto-update (navigate to server SPA on version mismatch)
- Docker build race condition (separate cargo cache IDs per stage)
- CSP blocking SvelteKit (per-request nonce injection)

### Security
- Multiple security hardening rounds: CORS, TOTP encryption, EXIF stripping, WebSocket auth
- JWT audience validation, prekey verification
- Secrets management overhaul, rate limiting, auth audit logging
- Fixed 2 critical + 5 high findings from security audit
- CSP hardened with nonce-based script policy
- Replaced `unwrap()` with proper error handling in channel routes

---

## v0.20.0

- Permissions overhaul: unified 5-tier role hierarchy (Instance Owner > Instance Admin > Owner/Admin > Moderator > Member)
- Instance Owner can now delete any message anywhere, including other users' DMs
- Instance Admin now bypasses community membership checks (previously only Instance Owner could)
- Channel moderator role: owners can promote members to moderator in channel member panels
- Moderators can delete others' messages, pin/unpin messages, close polls, and kick/ban members
- Role cycling in channel member panel for intuitive role management
- Consolidated duplicate permission logic across server codebase

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
