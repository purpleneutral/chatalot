# Search

> **Status: Complete**

Chatalot provides message search with filtering options. You can search within the current channel or across all channels you have access to.

## Opening Search

Press `Ctrl+F` (or `Cmd+F` on macOS) to open the search panel. The search bar appears at the top of the channel content area.

Press `Esc` to close the search panel.

## Search Scopes

Toggle between two search scopes using the button next to the search input:

| Scope | Description |
|-------|-------------|
| **Channel** | Search only within the currently active channel |
| **All** (Global) | Search across all channels and DMs you are a member of |

When using global search, each result includes the channel name so you know where the message was posted. Clicking a result from a different channel navigates you to that channel.

## How Search Works

- Type at least **2 characters** to trigger a search (maximum 256 characters).
- Search is debounced -- results appear after a 300ms pause in typing.
- Results are displayed in reverse chronological order (newest first).
- Search matches are highlighted in the results.

## Search Filters

Click **Filters** below the search input to expand advanced filtering options:

| Filter | Description |
|--------|-------------|
| **From user** | Filter results to messages sent by a specific username |
| **After date** | Only show messages sent after this date |
| **Before date** | Only show messages sent before this date |
| **Has file** | Only show messages that contain file attachments |

Filters can be combined. Click **Clear** to reset all filters.

## Navigating Results

- Click on a search result to jump to that message in the channel.
- When using global search, clicking a result from another channel switches to that channel first.
- The search panel remains open after clicking a result so you can continue browsing.

## Search Limitations

- Search queries must be between 2 and 256 characters.
- Each search returns up to **50 results** (20 by default).
- Search operates on the server-side ciphertext. Since messages are end-to-end encrypted, the server performs a byte-level search on the stored ciphertext. In practice, this means search works on the plaintext content when the fallback (non-encrypted) path is used.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+F` | Open/close search panel |
| `Esc` | Close search panel |

## Related Pages

- [Sending Messages](./sending-messages.md)
- [Text Channels](../channels/text-channels.md)
