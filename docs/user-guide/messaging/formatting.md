# Formatting

> **Status: Complete**

Chatalot supports Markdown formatting in messages. Text is rendered using the `marked` library and sanitized with DOMPurify for security.

## Markdown Syntax

### Inline Formatting

| Syntax | Result | Keyboard Shortcut |
|--------|--------|-------------------|
| `**bold**` | **bold** | `Ctrl+B` |
| `_italic_` or `*italic*` | _italic_ | `Ctrl+I` |
| `~~strikethrough~~` | ~~strikethrough~~ | -- |
| `` `inline code` `` | `inline code` | `Ctrl+E` |
| `[link text](url)` | Clickable link | -- |

### Block Formatting

#### Code Blocks

Fenced code blocks with optional language syntax highlighting:

````
```javascript
function hello() {
  console.log("Hello, world!");
}
```
````

#### Supported Languages

Syntax highlighting is provided by highlight.js with the following languages registered:

| Language | Aliases |
|----------|---------|
| JavaScript | `javascript`, `js` |
| TypeScript | `typescript`, `ts` |
| Python | `python`, `py` |
| Rust | `rust`, `rs` |
| CSS | `css` |
| HTML/XML | `html`, `xml` |
| JSON | `json` |
| Bash | `bash`, `sh`, `shell` |
| SQL | `sql` |
| YAML | `yaml`, `yml` |
| Go | `go` |
| Java | `java` |
| C/C++ | `cpp`, `c` |
| Markdown | `markdown`, `md` |

If no language is specified, the code block is rendered without syntax highlighting.

## Keyboard Formatting Shortcuts

These shortcuts work while the message input is focused:

| Shortcut | Action | Wraps Selection With |
|----------|--------|---------------------|
| `Ctrl+B` | Bold | `**text**` |
| `Ctrl+I` | Italic | `_text_` |
| `Ctrl+E` | Inline code | `` `text` `` |

When you use a formatting shortcut:

- If text is selected, the formatting is applied around the selection.
- If no text is selected, the formatting markers are inserted with a placeholder word "text" selected, ready for you to type over.

## Formatting Toolbar

The message input area includes a formatting toolbar with buttons for common formatting options. Click a toolbar button to insert the corresponding Markdown syntax at the cursor position.

## Links

URLs in messages are automatically detected and rendered as clickable links. You can also use Markdown link syntax for custom link text:

```
[Chatalot Documentation](https://docs.example.com)
```

## Link Previews

When a message contains a URL, Chatalot fetches a preview of the linked page (title, description, and thumbnail) and displays it below the message. This is handled server-side through the link preview API.

## Image Rendering

Images shared via file upload are rendered inline in the message. Clicking an image opens a **lightbox viewer** that allows you to:

- View the full-size image
- Navigate between images in the channel using arrow keys or on-screen buttons
- Close the lightbox with `Esc` or by clicking outside the image

## All Keyboard Shortcuts

Press `?` (or `Ctrl+/`) anywhere in the app to view the full keyboard shortcuts reference. The shortcuts modal lists:

**Messages:**
- Send message: `Enter` (or `Ctrl+Enter` based on settings)
- New line: `Shift+Enter` (or `Enter` based on settings)

**Formatting:**
- Bold: `Ctrl+B`
- Italic: `Ctrl+I`
- Inline code: `Ctrl+E`

**Navigation:**
- Quick switcher: `Ctrl+K`
- Search messages: `Ctrl+F`
- Show shortcuts: `?`
- Jump to latest: `End`
- Scroll to top: `Home`
- Focus message input: `Ctrl+T`
- Mark all read: `Shift+Esc`
- Close modal: `Esc`
- Upload file: Paste image from clipboard

## Related Pages

- [Sending Messages](./sending-messages.md)
- [Editing and Deleting](./editing-and-deleting.md)
