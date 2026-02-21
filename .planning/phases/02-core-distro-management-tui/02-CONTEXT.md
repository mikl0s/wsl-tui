# Phase 2: Core Distro Management TUI - Context

**Gathered:** 2026-02-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Full distro lifecycle management (list, install, start/stop/terminate, set default, remove, export/import, WSL kernel update) and shell attach in a polished Catppuccin Mocha-themed TUI with vim navigation, help overlay, fuzzy filter, and number-key view switching. This is the first shippable version.

</domain>

<decisions>
## Implementation Decisions

### Dashboard layout
- Right-side details panel: distro list on left, details for selected distro on right
- Details panel content and info density: Claude's discretion
- Distro list format (table rows vs styled list items): Claude's discretion
- Responsive behavior on narrow terminals: Claude's discretion

### Destructive action confirmation
- Use y/N single-key confirmation for destructive actions (remove distro)
- Simple and fast: press 'y' to confirm, anything else cancels
- Modal popup showing what will happen, with clear "cannot be undone" warning

### Install progress
- Modal overlay with progress bar and step-by-step status
- Blocks the distro list until install completes (no background install)

### Export/import file paths
- Claude's discretion on file path UX (text input, default path, etc.)

### Shell attach
- Instant swap: TUI clears immediately and drops into shell, no transition message
- Auto-start: pressing Enter on a stopped distro starts it first, then attaches
- Exit shell normally: type 'exit' or Ctrl+D to return to TUI
- Full restore on return: exact scroll position, selection, and view state preserved

### Fuzzy filter
- Inline filter bar: pressing '/' shows a text input at the top/bottom of the list
- Distros filter as you type, Esc to clear filter

### Claude's Discretion
- Distro list format (table rows vs styled list items)
- Details panel content and action hints
- Responsive layout behavior on narrow terminals
- Install flow initiation (popup picker vs dedicated view)
- Export/import file path UX
- Status bar content and layout
- Help overlay style (full-screen vs centered modal)
- View switching feel (instant vs tab bar)

</decisions>

<specifics>
## Specific Ideas

- Details panel inspired by the Midnight Commander / lazygit split-pane layout
- y/N confirmation preferred over type-to-confirm for speed
- Shell attach should feel invisible — instant in, full restore out
- Inline filter bar (like the `/` search in the distro list mockup) rather than a popup

</specifics>

<deferred>
## Deferred Ideas

- Auto-start distro on Windows boot / run distro in background after reboot — potential future phase or settings feature

</deferred>

---

*Phase: 02-core-distro-management-tui*
*Context gathered: 2026-02-21*
