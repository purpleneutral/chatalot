# Code Highlighting

> **Status: Complete**

Chatalot supports syntax-highlighted code blocks in messages, powered by [highlight.js](https://highlightjs.org/). Share code snippets with proper formatting and a convenient copy button.

## Inline Code

Wrap text in single backticks to render it as inline code:

```
Check the `config.rs` file for the default values.
```

Inline code appears in a monospace font with a subtle background, making it easy to distinguish from surrounding text.

## Fenced Code Blocks

Use triple backticks to create a multi-line code block:

````
```
function hello() {
    console.log("Hello, world!");
}
```
````

Without a language specifier, highlight.js will attempt to **auto-detect** the language.

## Specifying a Language

Add a language identifier after the opening triple backticks for accurate syntax highlighting:

````
```javascript
function hello() {
    console.log("Hello, world!");
}
```
````

A **language label** appears in the top-left corner of the code block when a language is specified.

## Supported Languages

The following languages are registered and available for syntax highlighting:

| Language | Identifiers |
|----------|-------------|
| JavaScript | `javascript`, `js` |
| TypeScript | `typescript`, `ts` |
| Python | `python`, `py` |
| Rust | `rust`, `rs` |
| CSS | `css` |
| HTML | `html` |
| XML | `xml` |
| JSON | `json` |
| Bash / Shell | `bash`, `sh`, `shell` |
| SQL | `sql` |
| YAML | `yaml`, `yml` |
| Go | `go` |
| Java | `java` |
| C / C++ | `c`, `cpp` |
| Markdown | `markdown`, `md` |

If you specify a language that is not in this list, highlight.js falls back to auto-detection. You can use any of the identifiers shown above -- for example, `js` and `javascript` are interchangeable.

## Auto-Detection

When no language identifier is provided, highlight.js uses heuristics to detect the most likely language. Auto-detection works well for distinctive languages (like Python or HTML) but may be less accurate for similar-looking languages. For best results, always specify the language.

## Copy Button

Every code block includes a **Copy** button in the top-right corner. The button appears when you hover over the code block.

1. Hover over the code block to reveal the **Copy** button.
2. Click **Copy**. The button text changes to "Copied!" to confirm.
3. The full contents of the code block are placed on your clipboard.

If the copy operation fails (for example, due to browser permission settings), the button briefly shows "Failed" before resetting.

## Theming

Code highlighting colors adapt to the active theme:

- **Dark mode** uses warm-toned syntax colors optimized for dark backgrounds.
- **Light mode** uses a separate set of colors designed for readability on light backgrounds.

Both themes are defined in the application's CSS and apply automatically. The colors cover keywords, strings, numbers, comments, types, function names, and other token categories.

## Markdown Rendering

Code blocks are part of Chatalot's broader Markdown support. Messages are parsed with [marked](https://marked.js.org/) (GitHub Flavored Markdown) and sanitized with DOMPurify before rendering. This means you can freely combine code blocks with other Markdown features like bold, italic, lists, blockquotes, and links in the same message.

For more on Markdown formatting, see [Sending Messages](../messaging/sending-messages.md).
