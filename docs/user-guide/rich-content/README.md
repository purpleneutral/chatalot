# Rich Content

Chatalot goes beyond plain text -- you can share files, images, GIFs, polls, and code snippets, all end-to-end encrypted.

## Pages

| Page | Description |
|------|-------------|
| [File Sharing](./file-sharing.md) | Upload, download, and manage file attachments |
| [Images and Media](./image-and-media.md) | Image previews, video playback, and audio players |
| [Polls](./polls.md) | Create and vote on polls within channels |
| [Link Previews](./link-previews.md) | Automatic website previews for shared URLs |
| [Code Highlighting](./code-highlighting.md) | Syntax-highlighted code blocks with copy support |
| [GIF Search](./gif-search.md) | Search and send GIFs powered by GIPHY |

## How It All Works Together

When you send a message, Chatalot automatically detects the type of content and renders it appropriately:

- **A message containing a URL** generates a link preview card with the site's title, description, and thumbnail.
- **A message containing an image URL** (ending in `.png`, `.jpg`, `.gif`, `.webp`, etc.) displays the image inline.
- **A file attachment** is uploaded to the server and displayed with a type-appropriate preview (image, video, audio, or generic file card).
- **Markdown code fences** are parsed and syntax-highlighted automatically.
- **A GIF** selected from the picker is sent as a URL and rendered inline.
- **A poll** is created through a dedicated modal and appears in the polls panel for the channel.

All uploaded files and message content are encrypted before leaving your device. The server stores only ciphertext.
