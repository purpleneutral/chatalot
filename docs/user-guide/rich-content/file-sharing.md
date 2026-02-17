# File Sharing

> **Status: Complete**

Share files with other members through encrypted uploads. Chatalot supports images, documents, archives, audio, video, and plain-text files.

## Uploading Files

There are three ways to attach a file to a channel:

### Upload Button

1. Click the **paperclip icon** to the left of the message input box.
2. Select a file from your device using the system file picker.
3. The file is uploaded and a file message is sent automatically to the channel.

### Drag and Drop

1. Drag a file from your desktop or file manager into the chat area.
2. A "Drop file to upload" overlay appears to confirm the drop zone.
3. Release to upload. The file message is sent immediately.

### Paste from Clipboard

1. Copy an image to your clipboard (for example, by taking a screenshot or right-clicking an image and selecting "Copy").
2. With the message input focused, press **Ctrl+V** (or **Cmd+V** on macOS).
3. The image is uploaded directly from the clipboard.

> **Tip:** Clipboard paste only works for image content. To share other file types, use the upload button or drag and drop.

## File Size and Quota Limits

| Setting | Default | Environment Variable |
|---------|---------|---------------------|
| Maximum file size per upload | **100 MB** | `MAX_FILE_SIZE_MB` |
| Per-user upload quota | **500 MB** | `UPLOAD_QUOTA_MB` (set to `0` for unlimited) |

If you exceed the per-upload size limit, the upload is rejected with an error message. If you exceed your total upload quota, you will see a message showing how much space remains.

Server administrators can change these limits through environment variables. See the [Configuration](../../self-hosting/configuration.md) guide for details.

## Supported File Types

Chatalot validates uploaded files using **magic byte detection** (not just the file extension) to ensure file type accuracy and block dangerous content. The following file types are allowed:

### Images
- PNG, JPEG, GIF, WebP, BMP

### Audio
- MP3 (including ID3-tagged files), OGG, FLAC, WAV

### Video
- MP4, WebM/MKV (Matroska)

### Documents
- PDF

### Archives
- ZIP (including DOCX, XLSX, PPTX, ODT), GZIP, BZIP2, XZ, 7Z, RAR

### Text
- Any valid UTF-8 text file (source code, Markdown, JSON, config files, etc.)

### Blocked File Types

The following are explicitly rejected for security reasons:

- Windows executables (`.exe`, `.dll` -- PE format)
- Linux executables (ELF binaries)
- macOS executables (Mach-O binaries)
- Java class files
- Shell scripts (files starting with `#!`)
- Windows batch scripts

Unrecognized binary formats are also rejected.

## Downloading Files

Every file message displays a **Download** link. Click it to download the file to your device. For images, videos, and audio, you can also interact with the content directly in the chat (see [Images and Media](./image-and-media.md)).

## File Security

- **End-to-end encryption.** Files are encrypted on the client before upload. The server stores only ciphertext.
- **SHA-256 checksums.** Every uploaded file is checksummed. Administrators can block specific file hashes to prevent re-upload of banned content.
- **Quarantine.** Administrators can quarantine files. Quarantined files cannot be downloaded by non-admin users.
- **Channel membership.** Only members of the channel associated with a file can download it.

## File Deletion

- **Your own files.** You can delete any file you uploaded.
- **Admins.** Server administrators and owners can delete any file. Deleted files are removed from both disk and database, and the uploader's quota usage is restored.

For administrator file management tools, see the Admin Guide.
