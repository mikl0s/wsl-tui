# WSL TUI — Theme Guidelines

**Palette:** Catppuccin Mocha
**Date:** 2026-02-21

---

## 1. Color Palette

WSL TUI uses the **Catppuccin Mocha** palette as its foundation. We use a wide range of colors from the palette to create visual hierarchy, clear affordances, and a cohesive aesthetic.

### Primary Colors (Always Used)

| Role | Color Name | Hex | Usage |
|------|-----------|-----|-------|
| **Background** | Base | `#1e1e2e` | Main background |
| **Surface** | Surface0 | `#313244` | Panels, cards, raised surfaces |
| **Surface Alt** | Surface1 | `#45475a` | Borders, separators, inactive tabs |
| **Surface Bright** | Surface2 | `#585b70` | Scrollbars, subtle UI elements |
| **Text Primary** | Text | `#cdd6f4` | Primary text, headings |
| **Text Secondary** | Subtext1 | `#bac2de` | Secondary text, descriptions |
| **Text Muted** | Subtext0 | `#a6adc8` | Placeholder text, disabled items |
| **Text Dim** | Overlay0 | `#6c7086` | Comments, timestamps, metadata |

### Accent Colors (Functional)

| Role | Color Name | Hex | Usage |
|------|-----------|-----|-------|
| **Primary Accent** | Mauve | `#cba6f7` | Selected items, active tab, focus ring, primary buttons |
| **Secondary Accent** | Blue | `#89b4fa` | Links, secondary actions, informational highlights |
| **Success** | Green | `#a6e3a1` | Running state, success messages, applied packs, gauges (OK) |
| **Warning** | Yellow | `#f9e2af` | Warnings, pending state, attention indicators |
| **Error** | Red | `#f38ba8` | Errors, stopped state, failed steps, destructive actions |
| **Info** | Sapphire | `#74c7ec` | Tips, help text, informational banners |
| **Highlight** | Peach | `#fab387` | Search matches, highlighted text, special indicators |
| **Accent Alt** | Lavender | `#b4befe` | Tab headers, section titles, category labels |
| **Connection** | Teal | `#94e2d5` | Connection status, terminal indicators, Termius |
| **Provisioning** | Pink | `#f5c2e7` | Pack categories, wizard step indicators, progress |
| **Resource CPU** | Flamingo | `#f2cdcd` | CPU usage gauge and charts |
| **Resource Memory** | Rosewater | `#f5e0dc` | Memory usage gauge and charts |

### Full Catppuccin Mocha Reference

```
Rosewater  #f5e0dc    Flamingo   #f2cdcd
Pink       #f5c2e7    Mauve      #cba6f7
Red        #f38ba8    Maroon     #eba0ac
Peach      #fab387    Yellow     #f9e2af
Green      #a6e3a1    Teal       #94e2d5
Sky        #89dceb    Sapphire   #74c7ec
Blue       #89b4fa    Lavender   #b4befe
Text       #cdd6f4    Subtext1   #bac2de
Subtext0   #a6adc8    Overlay2   #9399b2
Overlay1   #7f849c    Overlay0   #6c7086
Surface2   #585b70    Surface1   #45475a
Surface0   #313244    Base       #1e1e2e
Mantle     #181825    Crust      #11111b
```

## 2. Typography & Spacing

### TUI Text Styles

| Element | Style | Color |
|---------|-------|-------|
| Title bar | Bold | Lavender (`#b4befe`) on Mantle (`#181825`) |
| Section headers | Bold | Mauve (`#cba6f7`) |
| Selected list item | Bold, reverse | Mauve bg, Base text |
| Normal list item | Regular | Text (`#cdd6f4`) |
| Inactive list item | Regular | Subtext0 (`#a6adc8`) |
| Key hints (status bar) | Bold key, regular desc | Yellow key, Subtext1 desc |
| Status bar | Regular | Subtext1 on Mantle (`#181825`) |
| Error message | Bold | Red (`#f38ba8`) |
| Success message | Bold | Green (`#a6e3a1`) |

### Spacing
- Panel padding: 1 character on each side
- Between sections: 1 blank line
- List item height: 1 line (compact mode), 2 lines (detailed mode)
- Modal margin: 4 characters from terminal edge (responsive, min 2)

## 3. Borders & Decorations

| Element | Border Style | Color |
|---------|-------------|-------|
| Main panels | Rounded (`╭─╮│╰─╯`) | Surface1 (`#45475a`) |
| Active/focused panel | Rounded | Mauve (`#cba6f7`) |
| Modal overlays | Double (`╔═╗║╚═╝`) | Mauve (`#cba6f7`) |
| Inner sections | Plain (`─`) | Surface2 (`#585b70`) |
| Status bar | No border (full-width) | — |
| Preview boxes | Rounded, thin | Surface1 (`#45475a`) |

## 4. Component Patterns

### Gauges / Progress Bars
```
CPU  ▓▓▓▓▓▓░░░░░░░░░░  32%
```
- Filled: Flamingo (`#f2cdcd`) for CPU, Rosewater (`#f5e0dc`) for Memory, Blue (`#89b4fa`) for Disk
- Empty: Surface0 (`#313244`)
- Above 80%: Yellow (`#f9e2af`)
- Above 95%: Red (`#f38ba8`)

### State Indicators
| State | Symbol | Color |
|-------|--------|-------|
| Running | `●` | Green (`#a6e3a1`) |
| Stopped | `○` | Red (`#f38ba8`) |
| Starting | `◐` (animated) | Yellow (`#f9e2af`) |
| Default distro | `▸` (prefix) | Mauve (`#cba6f7`) |

### Tabs
- Active tab: Mauve (`#cba6f7`) text, underline
- Inactive tab: Subtext0 (`#a6adc8`) text
- Tab separator: Surface1 (`#45475a`)

### Buttons / Actions
- Primary: Mauve (`#cba6f7`) background
- Destructive: Red (`#f38ba8`) background
- Secondary: Surface1 (`#45475a`) background, Blue (`#89b4fa`) text

### Wizard Steps (Provisioning Modal)
```
  ● Shell  ─  ● Editor  ─  ○ Packages  ─  ○ Services
```
- Completed step: Green (`#a6e3a1`)
- Current step: Pink (`#f5c2e7`), bold
- Pending step: Surface2 (`#585b70`)
- Connector line: Surface1 (`#45475a`)

## 5. Web UI Adaptation

When adapting the theme for `wsl-web`:

- Use the same Catppuccin Mocha hex values as CSS custom properties
- Map TUI border patterns to CSS `border-radius: 8px` and `border: 1px solid`
- Gauge components become HTML/CSS progress bars or SVG radial gauges
- State indicators become colored dots (same colors)
- Font: Use a monospace font stack (`"JetBrains Mono", "Fira Code", "Cascadia Code", monospace`) for data, system sans-serif for UI text
- Respect `prefers-color-scheme` — Catppuccin Mocha is the dark theme, Catppuccin Latte for light (future)

```css
:root {
  --ctp-base: #1e1e2e;
  --ctp-surface0: #313244;
  --ctp-surface1: #45475a;
  --ctp-surface2: #585b70;
  --ctp-text: #cdd6f4;
  --ctp-subtext1: #bac2de;
  --ctp-subtext0: #a6adc8;
  --ctp-overlay0: #6c7086;
  --ctp-mauve: #cba6f7;
  --ctp-blue: #89b4fa;
  --ctp-green: #a6e3a1;
  --ctp-yellow: #f9e2af;
  --ctp-red: #f38ba8;
  --ctp-sapphire: #74c7ec;
  --ctp-peach: #fab387;
  --ctp-lavender: #b4befe;
  --ctp-teal: #94e2d5;
  --ctp-pink: #f5c2e7;
  --ctp-flamingo: #f2cdcd;
  --ctp-rosewater: #f5e0dc;
  --ctp-mantle: #181825;
  --ctp-crust: #11111b;
}
```

## 6. Accessibility Notes

- All foreground colors meet WCAG AA contrast ratio (4.5:1) against Base (`#1e1e2e`)
- Never rely on color alone — always pair with symbols (●/○), text labels, or position
- State indicators use both color AND symbol shape
- Focus is always visible (Mauve border/highlight)
- Error messages include text, not just red coloring
