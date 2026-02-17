# Custom Emoji

> **Status: Complete**

Communities can upload custom emoji that members can use in their messages. Custom emoji are community-specific -- they are available to all members of the community but not visible in other communities.

## Uploading Custom Emoji

> **Permission Required:** Community Admin or higher

1. Open the community settings or emoji management panel.
2. Click **Upload Emoji**.
3. Provide:
   - **Image file** -- the emoji image to upload
   - **Shortcode** -- the text code used to insert the emoji (e.g., `party_parrot`)
4. Click **Upload**.

The emoji is immediately available for all community members to use.

### Image Requirements

| Property | Requirement |
|----------|-------------|
| **Formats** | PNG, GIF, WebP |
| **Max file size** | 256 KB |
| **Animated** | Yes (GIF and animated WebP are supported) |

> **Tip:** JPEG is not supported for custom emoji. Use PNG for static emoji and GIF for animated ones.

### Shortcode Rules

The shortcode is the text identifier for the emoji, used to insert it in messages.

- **Length:** 2--32 characters
- **Allowed characters:** letters (a-z, A-Z), digits (0-9), and underscores (_)
- **Examples:** `thumbs_up`, `party_parrot`, `cool42`

Shortcodes must be unique within the community.

## Using Custom Emoji

To use a custom emoji in a message, type `:shortcode:` where `shortcode` is the emoji's shortcode. For example, typing `:party_parrot:` will insert the community's `party_parrot` emoji.

Custom emoji can be used in:

- Text messages
- Reactions (if the client supports custom emoji reactions)

## Managing Custom Emoji

### Viewing Emoji

All community members can view the list of custom emoji available in the community.

### Deleting Emoji

> **Permission Required:** Community Admin or higher

To delete a custom emoji:

1. Open the emoji management panel.
2. Find the emoji you want to remove.
3. Click **Delete**.

The emoji is permanently removed. Existing messages that used the emoji will no longer display it.

## Limits

| Limit | Value |
|-------|-------|
| Max emoji per community | **50** |
| Max file size per emoji | **256 KB** |
| Shortcode length | 2--32 characters |

Once you reach the 50-emoji limit, you must delete an existing emoji before uploading a new one.

## Emoji Metadata

Each custom emoji stores the following information:

- **Shortcode** -- the text identifier
- **URL** -- served at `/api/emojis/{id}`
- **Content type** -- the image MIME type (e.g., `image/png`)
- **Uploaded by** -- the user who uploaded the emoji
- **Created at** -- when the emoji was uploaded

## Next Steps

- [Community Settings](./community-settings.md) -- configure other community options
- [Managing Members](./managing-members.md) -- manage who can upload emoji
- [Reactions](../messaging/reactions.md) -- use emoji as reactions on messages
