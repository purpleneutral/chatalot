# Feature Status

The definitive reference for all Chatalot features and their implementation status.

> **Status: Complete** -- This table is actively maintained as features are added or updated.

---

## Status Key

| Status | Meaning |
|--------|---------|
| **Complete** | Fully implemented and available |
| **Beta** | Functional but under active development; may change |
| **Planned** | On the roadmap but not yet implemented |

---

## Core Features

| Feature | Status | Notes |
|---------|--------|-------|
| Communities | Complete | Top-level organization with roles, policies, and invite codes |
| Groups | Complete | Sub-organization within communities for channel grouping |
| Personal Groups | Complete | Moderator-assigned isolated spaces for individual members |
| Text Channels | Complete | Standard messaging with topic support |
| Voice Channels | Complete | WebRTC mesh, up to 25 participants |
| Direct Messages | Complete | Private 1:1 conversations |
| Channel Roles | Complete | Owner, admin, and member with graduated permissions |
| Channel Topics | Complete | Inline editing in channel header |
| Invite System | Complete | Codes with optional usage limits and expiration (1-8760 hours) |
| Privacy Controls | Complete | Visibility (public/private) and discoverable toggles per community, group, and channel |

## Messaging

| Feature | Status | Notes |
|---------|--------|-------|
| Real-time Delivery | Complete | WebSocket-based with optimistic UI updates |
| Markdown Formatting | Complete | Bold, italic, strikethrough, code, links, lists, blockquotes |
| Message Editing | Complete | Edit own messages; "(edited)" indicator with timestamp tooltip |
| Message Deletion | Complete | Delete own or others' (admin/owner); confirmation dialog |
| Edit History | Complete | Click "(edited)" to view all previous versions |
| Message Forwarding | Complete | Forward to another channel with attributed quote |
| Reply Threads | Complete | Threaded replies with dedicated panel, full message rendering |
| Reactions | Complete | Emoji reactions with tooltips showing who reacted |
| Mentions | Complete | @username, @everyone, @here, @channel with autocomplete |
| Pinned Messages | Complete | Max 50 per channel; slide-out panel; real-time updates |
| Bookmarks / Saved Items | Complete | Save messages with notes; dedicated browsing panel |
| Polls | Complete | 2-10 options, multi-select, anonymous voting, optional expiry |
| Scheduled Messages | Complete | Pick date and time; 30-day max, 50 per user limit |
| Message Search | Complete | Full-text search with filters for sender, date range, file type |
| Typing Indicators | Complete | Shows who is typing by display name; 3s throttle |
| Message Grouping | Complete | Consecutive messages within 5 min collapse |
| Unread Tracking | Complete | Per-channel unread counts with badge display |
| Unread Separator | Complete | "NEW MESSAGES" divider on channel entry |
| Mark All Read | Complete | Sidebar button + Shift+Esc shortcut |
| Read Receipts | Complete | DMs show "Read 5m ago"; channels show avatar stack; opt-out toggle |
| Smart Auto-Scroll | Complete | Only scrolls to new messages when near bottom |
| Date Separators | Complete | Visual dividers between messages on different days |
| Slash Commands | Complete | /shrug, /tableflip, /unflip, /lenny |
| Context Menu | Complete | Right-click for reply, edit, delete, copy, pin, forward, report |

## Rich Content

| Feature | Status | Notes |
|---------|--------|-------|
| File Upload | Complete | Drag-and-drop, clipboard paste; configurable max size (default 100 MB) |
| Inline Image Preview | Complete | Images render inline with lightbox viewer |
| Image Lightbox | Complete | Full-screen overlay with arrow key navigation |
| Inline Video Player | Complete | Native video controls |
| Inline Audio Player | Complete | MP3, WAV, FLAC, AAC, OGG, M4A with playback controls |
| GIF Search | Complete | GIPHY-powered picker with search and trending |
| Link Previews | Complete | URL embeds with title, description, thumbnail; toggleable per user |
| Code Highlighting | Complete | 15+ languages with auto-detection; one-click copy |
| Emoji Autocomplete | Complete | Type :shortcode to search 250+ emoji; includes custom emoji |
| Custom Emoji | Complete | Per-community uploads (PNG/GIF/WebP, max 256 KB, 50 per community) |

## Voice and Video

| Feature | Status | Notes |
|---------|--------|-------|
| Voice Chat | Complete | WebRTC mesh, up to 25 participants with adaptive quality |
| Video Calls | Complete | Camera toggle with video grid |
| Screen Sharing | Complete | With audio capture; volume/mute controls per stream |
| Noise Suppression | Complete | 3 tiers: Noise Gate, Speex, RNNoise |
| Per-User Volume | Complete | 0-500% via audio processing (GainNode) |
| Input/Output Device Selection | Complete | With live mic test and level meter |
| Echo Cancellation | Complete | Toggleable in voice settings |
| Auto Gain Control | Complete | Toggleable in voice settings |
| Active Speaker Highlight | Complete | Visual indicator on active speaker |
| Tiling Layout | Complete | Stream takes main pane with participant tiles alongside |
| Focus Mode | Complete | Hide tiles to watch stream full-width; auto-focus option |
| Persistent Voice Controls | Complete | Stay in sidebar when browsing other channels |
| Voice Call Backgrounds | Complete | 6 animated presets + solid color, gradient, or custom image |
| Per-Channel Ambiance | Complete | Admins set shared background for voice channels |
| Admin Kick from Voice | Complete | Admins can remove participants from calls |

## Customization

| Feature | Status | Notes |
|---------|--------|-------|
| Themes (Dark/Light) | Complete | System preference detection |
| Color Palettes | Complete | 8 built-in (Monokai, Dracula, Nord, Solarized, AMOLED, Catppuccin) + custom |
| Custom Theme Creator | Complete | Pick colors for backgrounds, text, and accents |
| Accent Colors | Complete | 8 options: Blue, Purple, Green, Orange, Red, Pink, Teal, Cyan |
| Message Density | Complete | Cozy (with avatars) / Compact (tight spacing) |
| Font Sizes | Complete | Small (13px) / Medium (14px) / Large (16px) |
| Time Format | Complete | 12-hour / 24-hour |
| Send Behavior | Complete | Enter or Ctrl+Enter (configurable) |
| Chat Bubble Style | Complete | Flat (classic) / Rounded (bubble-style) |
| Relative Timestamps | Complete | "5m ago" instead of exact times; toggleable |
| Animated Accent | Complete | Subtle color shift on accent elements |
| Reduce Motion | Complete | Disable all animations for accessibility |
| Formatting Toolbar | Complete | Quick-access buttons for bold, italic, code; toggleable |
| Link Preview Toggle | Complete | Show/hide link previews per user |
| Custom Scrollbars | Complete | Themed to match active palette |

## Profile and Presence

| Feature | Status | Notes |
|---------|--------|-------|
| User Profile Cards | Complete | Avatar, name, status, role, nickname, join date, bio, pronouns |
| Profile Banners | Complete | Upload banner image (max 5 MB); PNG, JPEG, WebP, GIF |
| Avatars | Complete | User-uploaded with GIF support |
| Bio and Pronouns | Complete | Editable in settings; displayed on profile cards |
| Online/Offline Presence | Complete | Online, idle, do not disturb, invisible |
| Custom Status | Complete | Text status visible on profile |
| Auto-Idle | Complete | Status set to idle after 5 minutes of inactivity |

## Community Customization

| Feature | Status | Notes |
|---------|--------|-------|
| Community Theme Editor | Complete | Override 7 CSS color variables per community |
| Custom CSS | Complete | Server-side sanitized; allowlisted properties only |
| Community Icon and Banner | Complete | Upload icon (2 MB) and banner (5 MB) |
| Welcome Message | Complete | Splash screen for first-time visitors |
| Group Icon and Banner | Complete | Per-group uploads |
| Group Accent Color | Complete | Displayed on group names in sidebar |

## Notifications

| Feature | Status | Notes |
|---------|--------|-------|
| Desktop Notifications | Complete | Browser Notification API / OS-native in desktop app |
| Web Push Notifications | Complete | DM notifications when tab is closed; metadata only (never message content); VAPID-signed |
| Notification Sounds | Complete | Per-type with volume control |
| Per-Channel Levels | Complete | All messages, @mentions only, or nothing |
| Permission Prompt | Complete | Polished slide-up prompt after first message |

## Admin and Moderation

| Feature | Status | Notes |
|---------|--------|-------|
| Admin Panel | Complete | User management, invites, announcements, reports, feedback |
| User Management | Complete | View, promote, manage, password reset |
| Invite System | Complete | Generate codes with limits and expiration |
| Announcements | Complete | Server-wide banners with per-user dismissal |
| Reports | Complete | Users report messages; admins review in panel |
| Audit Log | Complete | Auth events logged with IP and user agent |
| Warnings | Complete | Moderators issue warnings with reason tracking |
| Timeouts | Complete | Temporary mutes with configurable duration |
| Kicks and Bans | Complete | Remove or permanently ban from communities |
| User Blocking | Complete | Hide messages and prevent DMs from blocked users |
| Slow Mode | Complete | Rate-limit messages per channel (5s to 10m) |
| Read-Only Channels | Complete | Lock channels so only admins can post |
| Channel Archiving | Complete | Preserve channels in read-only state |
| Webhooks | Complete | Per-channel incoming webhooks for external integrations |

## Security

| Feature | Status | Notes |
|---------|--------|-------|
| Argon2id Password Hashing | Complete | 64 MiB memory, 3 iterations, 4 lanes |
| Ed25519-Signed JWTs | Complete | 15-minute access tokens with refresh token rotation |
| TOTP Two-Factor Auth | Complete | RFC 6238; any authenticator app |
| 2FA Backup Codes | Complete | 8 single-use codes; regeneratable |
| Account Recovery Codes | Complete | Self-service password reset without admin |
| Rate Limiting | Complete | Token-bucket per IP (20 req/s general, 5 req/s auth) |
| Security Headers | Complete | HSTS, CSP, X-Frame-Options, Permissions-Policy |
| SSRF Protection | Complete | Link preview proxy blocks private/internal IPs |
| Session Management | Complete | View and revoke active sessions; logout all devices |
| E2E Encryption (DMs) | Complete | Signal protocol (X3DH + Double Ratchet); compiled to WASM, running client-side |
| E2E Encryption (Groups) | Complete | Sender Keys; compiled to WASM, running client-side |
| Key Verification | Complete | Safety numbers, fingerprints, verification modal with copy |
| TOFU Key Change Detection | Complete | Yellow warning banner when peer identity key changes; acknowledge or re-verify |
| Encryption Status Indicators | Complete | Per-message lock icons (green = encrypted, red = decryption failed); header E2E badge |
| Disappearing Messages | Planned | Auto-delete after configurable time |

## Deployment

| Feature | Status | Notes |
|---------|--------|-------|
| Docker Compose | Complete | Single `docker compose up -d` -- two containers (app + PostgreSQL) |
| ARM64 Support | Complete | Multi-arch Docker images |
| PWA | Complete | Installable progressive web app |
| Desktop App (Tauri) | Beta | Linux and Windows; macOS planned |
| Interactive Install Script | Complete | Guided setup for new instances |

## Accessibility

| Feature | Status | Notes |
|---------|--------|-------|
| Keyboard Navigation | Complete | Full keyboard support with focus indicators |
| ARIA Labels | Complete | All interactive elements labeled |
| Screen Reader Support | Complete | Semantic HTML with proper roles |
| Reduce Motion | Complete | Disable animations system-wide |
| Mobile Responsive | Complete | Full-width panels, scrollable tabs, touch targets |

---

[Back to Appendix](README.md) | [Back to Documentation](../README.md)
