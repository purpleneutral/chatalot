# Link Previews

> **Status: Complete**

When you share a URL in a message, Chatalot automatically generates a rich preview card showing the linked website's title, description, and thumbnail image.

## How It Works

1. You send a message containing one or more URLs (starting with `http://` or `https://`).
2. The client detects the URLs and sends each one to the server's link preview endpoint.
3. The server fetches the target page, parses its [Open Graph](https://ogp.me/) metadata (and falls back to standard HTML `<title>` and `<meta>` tags), and returns the extracted data.
4. The client renders a preview card below your message.

## What Is Shown

Each link preview card displays the following metadata (when available):

| Field | Source | Max Length |
|-------|--------|------------|
| **Site name** | `og:site_name` meta tag | 100 characters |
| **Title** | `og:title` meta tag, or the `<title>` element as fallback | 200 characters |
| **Description** | `og:description` or `description` meta tag | 500 characters |
| **Thumbnail image** | `og:image` meta tag | -- |

The preview card is styled with an accent-colored left border and links to the original URL. Clicking the card opens the link in a new browser tab.

## Preview Limits

- A maximum of **3 link previews** are shown per message. If a message contains more than 3 URLs, only the first 3 generate preview cards.
- Image URLs (ending in `.png`, `.jpg`, `.gif`, `.webp`, `.svg`, `.bmp`, or `.ico`) are rendered as **inline images** instead of link previews, and do not count toward the 3-preview limit.
- The server limits the fetched page body to **512 KB** to prevent excessive resource usage.
- Only **HTML pages** (`text/html` content type) generate rich previews. Non-HTML URLs (PDFs, images, JSON endpoints, etc.) return an empty preview.

## Server-Side Caching

The server caches link preview results for **1 hour** (3,600 seconds) to reduce redundant outbound requests. The cache holds up to 1,000 entries before older entries are evicted.

## Security Protections

The link preview system includes protections against abuse:

- **SSRF protection.** The server blocks requests to private and internal IP ranges (`127.0.0.1`, `10.x.x.x`, `172.16-31.x.x`, `192.168.x.x`, `::1`, `0.0.0.0`), as well as `.local` and `.internal` domains.
- **URL validation.** Only `http://` and `https://` URLs are accepted. URLs must be between 1 and 2,048 characters.
- **User agent.** The server identifies itself as `ChatalotBot/1.0 (link preview)` when fetching pages.
- **Relative image URLs** are resolved to absolute URLs using the original page's scheme and host.

## Disabling Link Previews

If you prefer not to see link preview cards, you can disable them in your user settings:

1. Go to **Settings** (gear icon in the sidebar).
2. Find the **Link previews** toggle under the Chat section.
3. Turn the toggle **off**.

This is a client-side preference. Other users will still see link previews on their end based on their own settings.

> **Tip:** Disabling link previews does not affect inline image rendering. Image URLs will still display as inline images regardless of this setting.
