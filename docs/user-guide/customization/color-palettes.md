# Color Palettes

> **Status: Complete**

Color palettes change the entire look of Chatalot by replacing the core background, surface, text, and accent colors. Each palette has both a dark and a light variant that automatically matches your current [theme mode](./themes.md).

## Built-in Palettes

Chatalot ships with 7 built-in palettes plus a fully custom option:

| Palette | Description |
|---------|-------------|
| **Default** | Warm, earthy tones with an orange accent. The standard Chatalot look. |
| **Monokai** | Inspired by the classic code editor theme. Dark greens and muted yellows with a vibrant green accent. |
| **Dracula** | Deep purple-gray backgrounds with a soft purple accent. Based on the popular Dracula theme. |
| **Nord** | Cool, arctic blue-gray palette with a calming blue accent. Based on the Nord color scheme. |
| **Solarized** | Ethan Schoonover's precision-engineered color scheme with a blue accent. Works beautifully in both light and dark modes. |
| **AMOLED** | True black backgrounds designed for OLED screens. Maximizes contrast and saves battery on mobile devices. Uses a purple accent. |
| **Catppuccin** | Pastel palette with soft lavender accents. Warm and easy on the eyes, based on the Catppuccin theme. |
| **Custom** | Define your own colors with a full set of color pickers. |

## What Each Palette Changes

Every palette defines 7 color variables:

| Variable | Purpose |
|----------|---------|
| **Background** (`bgPrimary`) | Main app background |
| **Surface** (`bgSecondary`) | Cards, sidebars, panels |
| **Elevated** (`bgTertiary`) | Hover states, code blocks, input backgrounds |
| **Text** (`textPrimary`) | Primary text color |
| **Muted text** (`textSecondary`) | Secondary/placeholder text |
| **Accent** | Buttons, links, active indicators |
| **Accent hover** | Hover state of accent-colored elements |

## How to Switch Palettes

1. Open **Settings > Appearance**.
2. Find the **Color Palette** section.
3. Click any palette card to apply it. Each card shows a preview strip of the palette's primary background, elevated surface, and accent colors.

The change is instant and applies across the entire interface.

## Custom Palette

If none of the built-in palettes suit your taste, select **Custom** to define your own:

1. Click the **Custom** card (the one with the paint palette icon).
2. A row of 7 color pickers appears below the palette grid.
3. Adjust each color:
   - **Background** -- the main app canvas
   - **Surface** -- cards and sidebars
   - **Elevated** -- hover/active areas
   - **Text** -- primary text
   - **Muted text** -- secondary text
   - **Accent** -- buttons and highlights
   - **Accent hover** -- button hover states
4. Colors update in real time as you pick them.

> **Tip:** When using a custom palette, the accent color pickers in the next section are hidden since you control the accent directly from the custom palette color pickers.

## Palette Colors Reference

### Dark Mode

| Palette | Background | Surface | Accent |
|---------|-----------|---------|--------|
| Default | `#1a1816` | `#221f1c` | `#e07a4f` |
| Monokai | `#272822` | `#1e1f1c` | `#a6e22e` |
| Dracula | `#282a36` | `#21222c` | `#bd93f9` |
| Nord | `#2e3440` | `#272c36` | `#88c0d0` |
| Solarized | `#002b36` | `#001e27` | `#268bd2` |
| AMOLED | `#000000` | `#0a0a0a` | `#7c3aed` |
| Catppuccin | `#1e1e2e` | `#181825` | `#cba6f7` |

### Light Mode

| Palette | Background | Surface | Accent |
|---------|-----------|---------|--------|
| Default | `#f8f5f0` | `#ffffff` | `#c75d35` |
| Monokai | `#fafaf5` | `#ffffff` | `#669900` |
| Dracula | `#f8f8f2` | `#ffffff` | `#7c3aed` |
| Nord | `#eceff4` | `#ffffff` | `#5e81ac` |
| Solarized | `#fdf6e3` | `#ffffff` | `#268bd2` |
| AMOLED | `#f5f3ff` | `#ffffff` | `#6d28d9` |
| Catppuccin | `#eff1f5` | `#ffffff` | `#8839ef` |

## Related

- [Themes](./themes.md) -- switch between dark and light mode
- [Accent Colors](./accent-colors.md) -- change just the accent color when using the Default palette
