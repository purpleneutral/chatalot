# Images and Media

> **Status: Complete**

Chatalot provides inline previews for image, video, and audio files shared in channels. Media is displayed directly in the chat so you can view content without downloading.

## Image Support

### Uploading Images

Images can be shared using any of the methods described in [File Sharing](./file-sharing.md):

- **Upload button** -- Click the paperclip icon and select an image file.
- **Drag and drop** -- Drag an image from your desktop into the chat area.
- **Paste from clipboard** -- Press Ctrl+V (Cmd+V on macOS) to upload a screenshot or copied image directly.

### Supported Image Formats

The following formats are detected and displayed inline:

| Format | Extensions |
|--------|------------|
| PNG | `.png` |
| JPEG | `.jpg`, `.jpeg` |
| GIF | `.gif` |
| WebP | `.webp` |
| SVG | `.svg` |
| BMP | `.bmp` |
| ICO | `.ico` |

### Inline Image Previews

When an image file is uploaded, it is rendered directly in the chat as a thumbnail. The preview shows:

- The image itself, constrained to a maximum size suitable for the chat view.
- The file name and size below the image.
- A **Download** link to save the full image.

Images that appear as URLs in regular text messages (for example, `https://example.com/photo.png`) are also detected and rendered inline.

### Lightbox Viewer

Click any image in the chat to open it in the **lightbox** -- a full-screen overlay for viewing images at full resolution. The lightbox includes:

- **Navigation arrows** -- Browse through all images in the current channel using the left/right arrow buttons, or press the **Left Arrow** and **Right Arrow** keys on your keyboard.
- **Download button** -- Save the image directly from the lightbox.
- **Close** -- Click the background, press **Escape**, or click the close button to exit.

> **Tip:** You can navigate between all images shared in the current channel using the arrow keys while the lightbox is open.

## Video Support

### Supported Video Formats

| Format | Extensions |
|--------|------------|
| MP4 | `.mp4` |
| WebM | `.webm` |
| QuickTime | `.mov` |

### Video Playback

Uploaded video files are displayed with a native HTML5 video player embedded in the chat. The player includes standard controls:

- Play/pause
- Volume control
- Seek bar
- Fullscreen toggle

The file name, size, and a download link appear below the player.

## Audio Support

### Supported Audio Formats

| Format | Extensions |
|--------|------------|
| MP3 | `.mp3` |
| WAV | `.wav` |
| OGG | `.ogg` |
| FLAC | `.flac` |
| M4A | `.m4a` |
| AAC | `.aac` |
| Opus | `.opus` |
| WMA | `.wma` |

### Audio Playback

Audio files are displayed as a compact card showing:

- A music note icon
- The file name and size
- A **Download** link
- A native HTML5 audio player with play/pause, seek, and volume controls

## Other File Types

Files that are not images, video, or audio are displayed as a generic file card showing:

- A document icon
- The file name
- The file size
- A **Download** link

Examples include PDFs, ZIP archives, text files, and source code files.

## Encryption Note

All media files are encrypted on your device before upload. The server stores only encrypted blobs. When you view a file, the client fetches the encrypted blob and decrypts it locally before displaying or playing it. This means media previews work seamlessly while maintaining end-to-end encryption.
