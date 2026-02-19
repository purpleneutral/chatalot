# Community Settings

> **Status: Complete**

Community settings control the appearance, policies, and behavior of your community. Only Owners and Admins can edit these settings.

## Editing Community Settings

> **Permission Required:** Community Admin or higher

To edit community settings, open the community and navigate to the settings panel.

## General Settings

### Name

The community name is displayed in the community picker and at the top of the sidebar.

- **Length:** 1--64 characters
- **Required:** Yes

### Description

An optional description shown in invite previews (when the community is discoverable) and in the community details.

- **Length:** up to 2,048 characters
- **Required:** No

### Icon

The community icon appears in the community picker sidebar. Upload an image to personalize your community.

- **Formats:** PNG, JPEG, WebP, GIF
- **Max size:** 10 MB

### Banner

A banner image displayed in the community header area.

- **Formats:** PNG, JPEG, WebP, GIF
- **Max size:** 10 MB

## Policy Settings

Communities have configurable policies that control who can perform certain actions. Each policy can be set to one of three levels:

| Policy value | Who qualifies |
|-------------|---------------|
| `everyone` | Any community member |
| `moderator` | Moderators, Admins, and the Owner |
| `admin` | Admins and the Owner only |

### Who Can Create Groups

Controls who can create new groups within the community.

- **Default:** `everyone`
- **Options:** `everyone`, `moderator`, `admin`

### Who Can Create Invites

Controls who can generate invite links for the community.

- **Default:** `everyone`
- **Options:** `everyone`, `moderator`, `admin`

## Discoverability

The `discoverable` toggle controls whether the community's name and description are shown in invite previews.

- **Enabled:** Invite links show the community name, description, and member count.
- **Disabled:** Invite links show "Private Community" with no description.

This does not affect whether the community is publicly listed -- Chatalot does not have a public community directory. Discoverability only affects what information is revealed through invite links.

## Welcome Message

An optional welcome message displayed to new members when they join the community.

- **Length:** up to 2,000 characters
- **Required:** No

## Community Theme

Communities can customize their visual appearance with a theme. When a user switches to a community that has a custom theme, the theme overrides the default colors.

### Color Overrides

The following CSS color variables can be customized:

| Key | CSS Variable | Description |
|-----|-------------|-------------|
| `accent` | `--accent` | Primary accent color (buttons, links, highlights) |
| `accentHover` | `--accent-hover` | Accent color on hover |
| `bgPrimary` | `--bg-primary` | Main background color |
| `bgSecondary` | `--bg-secondary` | Secondary background (sidebar, panels) |
| `bgTertiary` | `--bg-tertiary` | Tertiary background (inputs, cards) |
| `textPrimary` | `--text-primary` | Primary text color |
| `textSecondary` | `--text-secondary` | Secondary/muted text color |

All color values must be valid hex colors (e.g., `#ff5500`, `#fff`).

### Custom CSS

Communities can inject custom CSS for advanced theming. The CSS is sanitized server-side before being applied.

> **Tip:** Community themes only apply while you are viewing that community. Switching to a different community reverts to its theme (or the default).

### Theme Limits

- Maximum theme JSON size: 8 KB

## Deleting a Community

> **Permission Required:** Community Owner

Deleting a community permanently removes:

- All groups within the community
- All channels within those groups
- All messages, files, and data associated with the community
- All community memberships, bans, and invites

This action cannot be undone. Only the community Owner can delete a community.

## Settings Reference

| Setting | Type | Default | Editable by |
|---------|------|---------|-------------|
| Name | String (1--64 chars) | Set at creation | Admin+ |
| Description | String (up to 2,048 chars) | None | Admin+ |
| Icon | Image (PNG/JPEG/WebP/GIF, 10 MB) | None | Admin+ |
| Banner | Image (PNG/JPEG/WebP/GIF, 10 MB) | None | Admin+ |
| Who can create groups | Policy (`everyone`/`moderator`/`admin`) | `everyone` | Admin+ |
| Who can create invites | Policy (`everyone`/`moderator`/`admin`) | `everyone` | Admin+ |
| Discoverable | Boolean | `true` | Admin+ |
| Welcome message | String (up to 2,000 chars) | None | Admin+ |
| Community theme | JSON object (up to 8 KB) | None | Admin+ |

## Next Steps

- [Custom Emoji](./custom-emoji.md) -- upload custom emoji for your community
- [Managing Members](./managing-members.md) -- invite users and manage roles
- [Groups Overview](../groups/overview.md) -- organize your community with groups
