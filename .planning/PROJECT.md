# WSL TUI

## What This Is

A Rust-based terminal user interface and web companion for managing WSL2 on Windows 11. It replaces memorizing `wsl.exe` commands with a visual dashboard, replaces copy-pasting setup scripts with stackable provisioning packs, and provides resource monitoring, backup/restore, and multiple connection modes — all in a polished, lazygit-inspired TUI with a Catppuccin Mocha theme.

Delivered as a Cargo workspace monorepo producing two binaries (`wsl-tui` and `wsl-web`) sharing a common `wsl-core` library. Built for developers, homelab admins, and power users who want WSL2 management without the CLI friction.

## Core Value

A user can go from "WSL installed" to "fully provisioned dev environment" in under 5 minutes by selecting packs and hitting go — reproducibly, idempotently, every time.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Visual distro lifecycle management (list, install, start, stop, terminate, remove, set default)
- [ ] Stackable pack provisioning system with idempotent execution and dry-run
- [ ] 9 built-in packs (home-setup, claude-code, nvm-node, python-dev, rust-dev, docker, ai-stack, gui-desktop, server-base)
- [ ] Interactive provisioning wizard with variable prompts per pack
- [ ] Resource monitoring (CPU, memory, disk) per running distro with gauges and charts
- [ ] Backup/restore (export .tar/.vhdx, import, named snapshots)
- [ ] 4 connection modes: shell attach, embedded terminal (PTY), external terminal, Termius (SSH)
- [ ] Dual storage backend: embedded libsql (default) with JSON fallback
- [ ] Plugin system: compile-time (built-in crates) + runtime (Lua Phase 1)
- [ ] Polished TUI with Catppuccin Mocha theme, vim-style navigation, responsive layout
- [ ] Dashboard, Provision, Monitor, Backup, Logs, Settings views
- [ ] Help overlay, command palette, fuzzy search/filter
- [ ] Configurable keybindings
- [ ] Web UI binary (Axum + embedded SPA) with REST API mirroring TUI functionality
- [ ] CLAUDE.md living documents (root + per-crate)

### Out of Scope

- WSL1 management — WSL2 only, WSL1 is legacy
- Remote WSL management — local machine only for v1
- Pack marketplace / central registry — share packs as TOML files for now
- WASM plugin runtime — Phase 2 after Lua API stabilizes
- Windows Store distribution — GitHub releases + winget for v1
- Auto-update mechanism — manual updates for v1
- CI/CD and release automation — deferred to a later milestone

## Context

- **Ecosystem gap:** No mature Rust-based TUI exists for WSL management. Closest is `wsl2-distro-manager` (Flutter GUI).
- **WSL interaction:** Primary via `wsl.exe` shell-out with UTF-16LE parsing, supplemented by `wslapi` crate for native API.
- **Inspiration:** lazygit, k9s — polished TUIs with clear layout and vim-style navigation.
- **Documentation:** Extensive docs already written — Design doc, BRD, PRD, SOW, Theme Guidelines, Agent Kickoff prompts, README.
- **Author:** Mikkel Georgsen, MIT License.
- **GitHub:** github.com/mikl0s/wsl-tui

## Constraints

- **Platform:** Windows 11 (build 22000+) — WSL2 requirement
- **Language:** Rust stable, Edition 2024, Cargo workspace
- **TUI framework:** Ratatui 0.30+ with crossterm backend
- **Web framework:** Axum 0.8+ with tower-http
- **Theme:** Catppuccin Mocha (see docs/THEME_GUIDELINES.md for full spec)
- **Performance:** Startup < 500ms, idle memory < 50MB, binary < 30MB
- **Terminal:** Must work in Windows Terminal, ConHost, Alacritty, WezTerm
- **Quality:** Zero clippy warnings, all tests pass, rustdoc on public APIs

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Cargo workspace monorepo | Share wsl-core between TUI and Web binaries | — Pending |
| Compile-time + runtime plugins | Built-in plugins for core features, Lua for community extensibility | — Pending |
| libsql embedded + JSON fallback | No external DB install needed, graceful degradation | — Pending |
| Catppuccin Mocha theme | Popular, accessible palette with good contrast ratios | — Pending |
| TOML-based packs | Human-readable, easy to share and version control | — Pending |
| Shell out to wsl.exe | Most reliable WSL interaction, supplemented by wslapi | — Pending |
| Ratatui 0.30+ | Latest stable, good Windows support via crossterm | — Pending |
| Filter KeyEventKind::Press | Windows crossterm quirk — prevents double key events | — Pending |

---
*Last updated: 2026-02-21 after initialization*
