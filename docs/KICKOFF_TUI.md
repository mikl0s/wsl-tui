# WSL TUI — Agent Developer Kickoff Prompt

Copy this prompt to start a new session with an AI coding agent to implement the TUI binary.

---

## Kickoff Prompt

```
You are building WSL TUI — a Rust-based terminal user interface for managing WSL2 on Windows 11.

## Project Context

This is a Cargo workspace monorepo. Read these docs before writing any code:
- docs/plans/2026-02-21-wsl-tui-design.md — Full architecture and design
- docs/PRD.md — Product requirements and user stories
- docs/SOW.md — Statement of work and phased delivery plan
- docs/THEME_GUIDELINES.md — Catppuccin Mocha theme specification

## What You're Building

A polished TUI (think lazygit/k9s quality) for managing WSL2 distros on Windows 11. Key features:

1. **Distro Management** — List/start/stop/install/remove/set-default WSL distros
2. **Stackable Pack Provisioning** — Post-install setup via composable TOML packs (home-setup, claude-code, nvm-node, etc.). Each pack is idempotent with dry-run support.
3. **Resource Monitoring** — CPU/memory/disk gauges per distro
4. **Backup/Restore** — Export/import/snapshot management
5. **4 Connection Modes** — Shell attach, embedded terminal (PTY), external terminal, Termius (SSH)
6. **Plugin System** — Compile-time (built-in workspace crates) + runtime (Lua scripts, later WASM)
7. **Dual Storage** — Embedded libsql (default) with JSON fallback

## Architecture

Cargo workspace with these crates:
- `wsl-core` — Shared library (plugin trait, WSL API, storage, pack engine, config, models)
- `wsl-plugin-distro` — Distro lifecycle management
- `wsl-plugin-provision` — Pack provisioning system
- `wsl-plugin-monitor` — Resource monitoring
- `wsl-plugin-backup` — Export/import/snapshot
- `wsl-plugin-connect` — Connection modes (shell, embedded, external, Termius)
- `wsl-tui` — Ratatui terminal UI binary (this is your primary focus)
- `wsl-web` — Axum web UI binary (separate effort)

## Key Technical Decisions

- **TUI:** Ratatui 0.30+ with crossterm backend. Filter KeyEventKind::Press only (Windows quirk).
- **Storage:** `libsql` crate with `core` feature (embedded). `serde_json` for JSON fallback. Trait-based abstraction.
- **WSL:** Shell out to `wsl.exe` + `wslapi` crate. Handle UTF-16LE output.
- **Async:** Tokio runtime
- **Config:** TOML via `serde`
- **CLI:** clap 4.x
- **Theme:** Catppuccin Mocha — see THEME_GUIDELINES.md for exact colors and patterns

## Implementation Order (follow SOW phases)

Phase 1: Foundation → Cargo workspace, wsl-core, storage, WSL execution layer, plugin system, basic TUI shell
Phase 2: Core Features → distro plugin, dashboard, pack engine, provisioning plugin + modal, built-in packs
Phase 3: Connectivity → connection modes (all 4), monitoring, backup
Phase 4: Extensibility → Lua plugins, settings view, help overlay, command palette, keybindings
Phase 5: Web UI → wsl-web (separate phase)

## Quality Standards

- `cargo clippy --workspace` — zero warnings
- `cargo test --workspace` — all pass
- Rustdoc on all public API surfaces
- Integration tests with mocked WSL output
- Startup < 500ms, memory < 50MB idle, binary < 30MB

## Key Files to Read First

1. Cargo.toml (workspace)
2. crates/wsl-core/src/lib.rs
3. crates/wsl-tui/src/main.rs
4. docs/THEME_GUIDELINES.md

Start with Phase 1: scaffold the workspace, implement wsl-core with storage and WSL execution, get the basic TUI rendering.
```
