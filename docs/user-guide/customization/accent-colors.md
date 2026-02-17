# Accent Colors

> **Status: Complete**

The accent color controls the highlight color used throughout the Chatalot interface. It affects buttons, links, active states, selection indicators, and other interactive elements.

## Available Accent Colors

Chatalot offers 8 accent colors, each with dark and light mode variants:

| Color | Dark Mode | Light Mode |
|-------|-----------|------------|
| **Blue** | `#4e8fda` | `#3574c2` |
| **Purple** | `#9c7ae0` | `#7c5ec4` |
| **Green** | `#57ab5a` | `#3d8c40` |
| **Orange** (default) | `#e07a4f` | `#c75d35` |
| **Red** | `#e5534b` | `#cc3d35` |
| **Pink** | `#d96ba0` | `#c44e87` |
| **Teal** | `#4ecdc4` | `#2eb5ac` |
| **Cyan** | `#56b6c2` | `#3a9eab` |

## What the Accent Color Affects

The accent color is applied to the `--accent` and `--accent-hover` CSS variables and is used for:

- **Buttons** -- primary action buttons (Send, Save, Create, etc.)
- **Links** -- clickable text and hyperlinks in messages
- **Active indicators** -- the currently selected tab, channel, or setting
- **Selection rings** -- the highlight ring around the chosen accent color swatch
- **Toggle switches** -- the filled state of on/off toggles
- **Blockquote borders** -- the left border on quoted text in messages
- **Progress indicators** -- loading bars and status indicators
- **Input focus** -- the border color when a text field is focused

## How to Change the Accent Color

1. Open **Settings > Appearance**.
2. Find the **Accent Color** section (only visible when using the **Default** palette).
3. Click any color circle. The currently selected color shows a checkmark.

The change is applied instantly across the entire interface.

> **Note:** The Accent Color section is only shown when the **Default** color palette is selected. Other palettes (Monokai, Dracula, Nord, etc.) define their own accent color as part of the palette. To use a custom accent with a non-default palette, select the **Custom** palette and set the accent color manually.

## Related

- [Color Palettes](./color-palettes.md) -- palettes that include their own accent colors
- [Themes](./themes.md) -- dark/light mode changes which variant of the accent color is used
