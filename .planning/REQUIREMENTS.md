# Requirements: WSL TUI

**Defined:** 2026-02-21
**Core Value:** A user can go from "WSL installed" to "fully provisioned dev environment" in under 5 minutes by selecting packs and hitting go — reproducibly, idempotently, every time.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Foundation

- [ ] **FOUND-01**: Cargo workspace compiles with all crate scaffolding and zero warnings
- [ ] **FOUND-02**: libsql embedded storage works on Windows with stack overflow workaround
- [ ] **FOUND-03**: JSON fallback storage activates transparently when libsql fails
- [ ] **FOUND-04**: Storage backend is configurable via `config.toml` (`auto` | `libsql` | `json`)
- [ ] **FOUND-05**: WSL command execution handles both UTF-16LE and UTF-8 output encoding
- [ ] **FOUND-06**: Plugin trait and registry system supports compile-time plugin registration
- [ ] **FOUND-07**: Configuration loaded from `~/.wsl-tui/config.toml` with sensible defaults
- [ ] **FOUND-08**: TUI event loop filters `KeyEventKind::Press` only (Windows crossterm fix)
- [ ] **FOUND-09**: Panic hook restores terminal on crash via `ratatui::init()`/`ratatui::restore()`
- [ ] **FOUND-10**: Workspace uses `resolver = "2"` to prevent feature unification issues

### Distro Management

- [ ] **DIST-01**: User can see all installed WSL distros with state (Running/Stopped), WSL version, and default indicator
- [ ] **DIST-02**: User can install a new distro from the available online list with progress feedback
- [ ] **DIST-03**: User can start a stopped distro
- [ ] **DIST-04**: User can stop a running distro
- [ ] **DIST-05**: User can terminate a distro (force stop)
- [ ] **DIST-06**: User can set a distro as the WSL default
- [ ] **DIST-07**: User can remove (unregister) a distro with a confirmation prompt
- [ ] **DIST-08**: User can export a distro to a `.tar` file
- [ ] **DIST-09**: User can import a distro from a `.tar` file
- [ ] **DIST-10**: User can update the WSL kernel from within the TUI

### Provisioning

- [ ] **PROV-01**: Pack engine parses TOML pack definitions with steps, variables, and dependencies
- [ ] **PROV-02**: Pack engine resolves dependency graph via topological sort
- [ ] **PROV-03**: Pack engine detects conflicts between packs before execution
- [ ] **PROV-04**: Each provisioning step executes idempotently (skips already-applied steps)
- [ ] **PROV-05**: Pack execution shows real-time progress with per-step status
- [ ] **PROV-06**: Failed steps can be retried; execution resumes from last failed step
- [ ] **PROV-07**: User can dry-run a pack to preview exactly what will change before applying
- [ ] **PROV-08**: Interactive wizard prompts user for per-pack variables (shell, Node version, etc.)
- [ ] **PROV-09**: All 9 built-in packs are available: home-setup, claude-code, nvm-node, python-dev, rust-dev, docker, ai-stack, gui-desktop, server-base
- [ ] **PROV-10**: User can create and load custom packs from `~/.wsl-tui/packs/` as TOML files
- [ ] **PROV-11**: User can re-provision an existing distro by selecting additional packs
- [ ] **PROV-12**: Pack application state is persisted in storage (which packs applied, when, with what variables)

### Connection

- [ ] **CONN-01**: User can connect to a distro via shell attach (TUI suspends, drops into shell, restores on exit)
- [ ] **CONN-02**: User can launch a distro in an external terminal (Windows Terminal, Alacritty, WezTerm)
- [ ] **CONN-03**: External terminal command is configurable with `{distro_name}` template substitution
- [ ] **CONN-04**: User can launch a distro in Termius with automatic SSH server provisioning
- [ ] **CONN-05**: Per-distro SSH port mapping for Termius (base_port + offset)
- [ ] **CONN-06**: User can configure a default connection mode globally and per distro

### Monitoring

- [ ] **MON-01**: User can see real-time CPU usage per running distro as a gauge
- [ ] **MON-02**: User can see real-time memory usage per running distro as a gauge
- [ ] **MON-03**: User can see disk usage per distro
- [ ] **MON-04**: Full-screen monitoring dashboard with sparklines/charts
- [ ] **MON-05**: Resource metrics are logged to storage with timestamps for historical queries
- [ ] **MON-06**: Polling interval is configurable (default 5s) and does not block the render thread

### Backup

- [ ] **BACK-01**: User can create named snapshots of distros with description and timestamp
- [ ] **BACK-02**: User can see snapshot history with names, timestamps, and file sizes
- [ ] **BACK-03**: User can restore from a named snapshot

### TUI Interface

- [ ] **TUI-01**: Dashboard view shows distro list, details panel, and resource monitor summary
- [ ] **TUI-02**: Provision view as a modal overlay with pack selection, variable wizard, and execution progress
- [ ] **TUI-03**: Monitor view with full-screen resource charts and per-distro breakdown
- [ ] **TUI-04**: Backup view with snapshot manager (create, list, restore)
- [ ] **TUI-05**: Logs view with scrollable execution history and filtering
- [ ] **TUI-06**: Settings view with TUI-based config editor
- [ ] **TUI-07**: Status bar showing active distro, state, storage indicator, and clock
- [ ] **TUI-08**: Vim-style navigation (h/j/k/l, arrows, Tab for panels)
- [ ] **TUI-09**: Help overlay (`?`) showing context-aware keybindings per active view
- [ ] **TUI-10**: Fuzzy search/filter (`/`) across distros and packs
- [ ] **TUI-11**: Command palette (`:`) with fuzzy-matched command list
- [ ] **TUI-12**: Responsive layout adapting to terminal size with min-width guards
- [ ] **TUI-13**: Catppuccin Mocha theme applied consistently (per THEME_GUIDELINES.md)
- [ ] **TUI-14**: Keybindings are configurable via `config.toml`
- [ ] **TUI-15**: Views accessible via number keys (1-5: Dashboard, Provision, Monitor, Backup, Logs)

### Extensibility

- [ ] **EXT-01**: Lua plugins load from `~/.wsl-tui/plugins/*.lua` via mlua runtime
- [ ] **EXT-02**: Lua plugins are sandboxed (no os/io/package stdlib access by default)
- [ ] **EXT-03**: Lua plugins declare required permissions; user approves on first load
- [ ] **EXT-04**: A failing Lua plugin never crashes the host application
- [ ] **EXT-05**: Plugin API includes access to distro list, pack operations, and notification system

### Web UI

- [ ] **WEB-01**: `wsl-web` binary starts an Axum server on `127.0.0.1:3000`
- [ ] **WEB-02**: REST API endpoints mirror all TUI functionality (distros, packs, monitor, backup, connect, config)
- [ ] **WEB-03**: SPA frontend is embedded in the binary via rust-embed (no separate file serving)
- [ ] **WEB-04**: Real-time resource metrics stream via WebSocket/SSE to the browser
- [ ] **WEB-05**: Web UI uses Catppuccin Mocha theme (CSS custom properties from THEME_GUIDELINES.md)
- [ ] **WEB-06**: API returns consistent JSON error format `{ "error": "message" }`

### Developer Experience

- [ ] **DX-01**: CLAUDE.md at repo root with coding standards, architecture patterns, and Rust conventions
- [ ] **DX-02**: Per-crate CLAUDE.md files for wsl-core, wsl-tui, and wsl-web with crate-specific context
- [ ] **DX-03**: `cargo clippy --workspace` passes with zero warnings
- [ ] **DX-04**: `cargo test --workspace` passes all tests
- [ ] **DX-05**: Startup time under 500ms to first render
- [ ] **DX-06**: Idle memory usage under 50MB
- [ ] **DX-07**: Binary size under 30MB (without WASM runtime)

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Connection

- **CONN-V2-01**: User can connect via embedded terminal pane within the TUI (ConPTY/PTY split-pane)

### Extensibility

- **EXT-V2-01**: WASM plugin runtime via wasmtime for multi-language plugin support
- **EXT-V2-02**: Plugin API versioning (API_VERSION constant for compatibility checks)

### Distribution

- **DIST-V2-01**: GitHub Actions CI/CD pipeline (build, test, lint, clippy)
- **DIST-V2-02**: Release workflow producing Windows binaries
- **DIST-V2-03**: winget package manifest
- **DIST-V2-04**: Scoop bucket manifest

## Out of Scope

| Feature | Reason |
|---------|--------|
| WSL1 management | WSL2 only; WSL1 is legacy with different architecture |
| Remote WSL management | Local machine only; triples scope with network/auth layer |
| Pack marketplace / registry | Share packs as TOML files; community can use GitHub repos |
| Auto-update mechanism | Use winget + GitHub releases; winget handles updates |
| Windows Store distribution | Conflicts with wsl.exe shell-out and PTY sandboxing |
| WASM plugins in v1 | Defer until Lua API stabilizes; WASM adds ~10MB binary size |
| Real-time sub-100ms polling | 1-5s interval is sufficient; aggressive polling destabilizes slow distros |
| GUI window (native Windows) | TUI + Web UI covers all use cases |
| Distro cloning | Named snapshots + import-as-new achieves same outcome |
| Interactive root shell default | Security risk; require explicit sudo |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| FOUND-01 | Phase 1 | Pending |
| FOUND-02 | Phase 1 | Pending |
| FOUND-03 | Phase 1 | Pending |
| FOUND-04 | Phase 1 | Pending |
| FOUND-05 | Phase 1 | Pending |
| FOUND-06 | Phase 1 | Pending |
| FOUND-07 | Phase 1 | Pending |
| FOUND-08 | Phase 1 | Pending |
| FOUND-09 | Phase 1 | Pending |
| FOUND-10 | Phase 1 | Pending |
| DIST-01 | Phase 2 | Pending |
| DIST-02 | Phase 2 | Pending |
| DIST-03 | Phase 2 | Pending |
| DIST-04 | Phase 2 | Pending |
| DIST-05 | Phase 2 | Pending |
| DIST-06 | Phase 2 | Pending |
| DIST-07 | Phase 2 | Pending |
| DIST-08 | Phase 2 | Pending |
| DIST-09 | Phase 2 | Pending |
| DIST-10 | Phase 2 | Pending |
| TUI-01 | Phase 2 | Pending |
| TUI-07 | Phase 2 | Pending |
| TUI-08 | Phase 2 | Pending |
| TUI-09 | Phase 2 | Pending |
| TUI-10 | Phase 2 | Pending |
| TUI-12 | Phase 2 | Pending |
| TUI-13 | Phase 2 | Pending |
| TUI-14 | Phase 2 | Pending |
| TUI-15 | Phase 2 | Pending |
| PROV-01 | Phase 3 | Pending |
| PROV-02 | Phase 3 | Pending |
| PROV-03 | Phase 3 | Pending |
| PROV-04 | Phase 3 | Pending |
| PROV-05 | Phase 3 | Pending |
| PROV-06 | Phase 3 | Pending |
| PROV-07 | Phase 3 | Pending |
| PROV-08 | Phase 3 | Pending |
| PROV-09 | Phase 3 | Pending |
| PROV-10 | Phase 3 | Pending |
| PROV-11 | Phase 3 | Pending |
| PROV-12 | Phase 3 | Pending |
| TUI-02 | Phase 3 | Pending |
| MON-01 | Phase 4 | Pending |
| MON-02 | Phase 4 | Pending |
| MON-03 | Phase 4 | Pending |
| MON-04 | Phase 4 | Pending |
| MON-05 | Phase 4 | Pending |
| MON-06 | Phase 4 | Pending |
| BACK-01 | Phase 4 | Pending |
| BACK-02 | Phase 4 | Pending |
| BACK-03 | Phase 4 | Pending |
| TUI-03 | Phase 4 | Pending |
| TUI-04 | Phase 4 | Pending |
| TUI-05 | Phase 4 | Pending |
| CONN-01 | Phase 5 | Pending |
| CONN-02 | Phase 5 | Pending |
| CONN-03 | Phase 5 | Pending |
| CONN-04 | Phase 5 | Pending |
| CONN-05 | Phase 5 | Pending |
| CONN-06 | Phase 5 | Pending |
| EXT-01 | Phase 6 | Pending |
| EXT-02 | Phase 6 | Pending |
| EXT-03 | Phase 6 | Pending |
| EXT-04 | Phase 6 | Pending |
| EXT-05 | Phase 6 | Pending |
| TUI-06 | Phase 6 | Pending |
| TUI-11 | Phase 6 | Pending |
| WEB-01 | Phase 7 | Pending |
| WEB-02 | Phase 7 | Pending |
| WEB-03 | Phase 7 | Pending |
| WEB-04 | Phase 7 | Pending |
| WEB-05 | Phase 7 | Pending |
| WEB-06 | Phase 7 | Pending |
| DX-01 | Phase 1 | Pending |
| DX-02 | Phase 1 | Pending |
| DX-03 | All | Pending |
| DX-04 | All | Pending |
| DX-05 | Phase 2 | Pending |
| DX-06 | Phase 2 | Pending |
| DX-07 | Phase 2 | Pending |

**Coverage:**
- v1 requirements: 73 total
- Mapped to phases: 73
- Unmapped: 0 ✓

---
*Requirements defined: 2026-02-21*
*Last updated: 2026-02-21 after initial definition*
