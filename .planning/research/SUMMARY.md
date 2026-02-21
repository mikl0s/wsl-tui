# Project Research Summary

**Project:** wsl-tui
**Domain:** Rust TUI + Web UI for WSL2 management on Windows 11
**Researched:** 2026-02-21
**Confidence:** HIGH (stack and architecture), MEDIUM-HIGH (features and pitfalls)

## Executive Summary

wsl-tui is a local desktop tool that wraps WSL2 management into a keyboard-driven TUI and an optional web companion. Experts build this class of tool as a Cargo workspace with a shared `wsl-core` library and two separate binaries — one for the terminal UI (Ratatui + crossterm) and one for the web interface (Axum + embedded SPA). The Elm Architecture (Model-Update-View) is the established Ratatui pattern for multi-view tools of this complexity; it pairs cleanly with an async tokio event loop that isolates long-running WSL commands from the render thread. The plugin architecture should be dual: compile-time workspace crates (for the five functional domains — distro, provision, monitor, backup, connect) and runtime Lua plugins (mlua, Phase 1) to enable community extensibility without WASM complexity in v1.

The recommended delivery sequence is: foundation scaffolding first (storage, WSL command layer, plugin host), then core distro management TUI, then the provisioning pack engine, then monitoring and backup, then full connectivity (embedded PTY), then the extensibility layer (Lua runtime), and finally the web binary. This order respects hard technical dependencies — nothing else can work until the distro list is reliable — and places the provisioning pack system (the primary competitive differentiator) in Phase 3 where it has a solid scaffold to build on. Both existing competitors (wsl2-distro-manager and wsl-gui-tool) are basic lifecycle managers with no provisioning, monitoring, snapshots, or plugin support, leaving the entire differentiator column open.

The most dangerous risks are all Windows-specific and all preventable in Phase 1: libsql causes a stack overflow on Windows without a linker flag, crossterm fires duplicate key events without a filter, and wsl.exe output requires encoding detection (UTF-16LE vs UTF-8 depending on WSL_UTF8 env var). All three require one-time platform adaptations, not architectural changes. The ConPTY embedded terminal is the one component that needs a dedicated spike — it is the most complex connectivity feature and the most likely to require iteration before it is shippable.

---

## Key Findings

### Recommended Stack

The stack is fully resolved at HIGH confidence for the core components. Ratatui 0.30 with crossterm 0.29 is the definitive TUI choice; tui-rs is unmaintained and termion is Linux-only. Axum 0.8 (January 2025) with tokio 1.x is the web tier. libsql 0.9 is the embedded database with a JSON fallback trait for portability. mlua 0.11.5 with `vendored` feature provides the Lua 5.4 runtime with no system dependency. The workspace MSRV is 1.88 (driven by sysinfo 0.37.2, which supersedes ratatui's 1.86 requirement).

**Core technologies:**
- **Rust 1.88+ / Rust 2024 edition** — MSRV dictated by sysinfo; Rust 2024 edition required by ratatui 0.30
- **ratatui 0.30 + crossterm 0.29** — sole viable TUI stack for Windows-compatible terminal rendering
- **axum 0.8 + tokio 1.x (LTS 1.47.x)** — Tokio-native web framework; WebSocket built-in for real-time metric streaming
- **libsql 0.9 (embedded)** — async-native SQLite fork; JSON trait fallback for portability; requires Windows linker stack flag
- **mlua 0.11.5 (lua54 + vendored + async)** — Lua 5.4 runtime, no system dependency, async-native; vendored builds Lua from source
- **portable-pty 0.8** — WezTerm-maintained ConPTY wrapper; the only battle-tested cross-platform PTY for Windows
- **catppuccin 2.4 (ratatui feature)** — official crate; direct `ratatui::style::Color` conversion, eliminates manual hex mapping
- **rust-embed 8.11** — compile-time SPA asset embedding; dev filesystem, release embedded
- **sysinfo 0.37.2** — per-distro CPU/memory/disk; Windows-native; sets the workspace MSRV ceiling

See `.planning/research/STACK.md` for full dependency table with versions, rationale, and alternatives considered.

### Expected Features

Both existing competitors (wsl2-distro-manager, wsl-gui-tool) cover only basic lifecycle management. No competitor provides provisioning, monitoring, snapshots, or plugins. The entire differentiator column is uncontested.

**Must have (table stakes):**
- Distro list with Running/Stopped/WSL version/disk status — root dependency for everything else
- Start, stop, terminate, set default — core lifecycle
- Install from online list with progress stream — first-run experience
- Remove with confirmation modal — guards destructive operations
- Shell attach (suspend TUI, launch wsl, restore) — primary connection need
- Export and import .tar — built into wsl.exe; users expect the GUI wrapper
- Vim-style navigation (hjkl), help overlay (`?`), fuzzy filter (`/`) — TUI-literate user baseline
- Catppuccin Mocha theme throughout — brand identity from day one
- Configurable keybindings via config.toml
- Responsive layout adapting to terminal size

**Should have (competitive differentiators):**
- Stackable TOML provisioning pack engine with idempotency — the core product value; no competitor has this
- Interactive provisioning wizard with per-variable prompts
- Dry-run mode showing exact planned changes before execution
- 9 built-in packs (home-setup, claude-code, nvm-node, python-dev, rust-dev, docker, ai-stack, gui-desktop, server-base)
- Custom TOML pack authoring and import
- Resource monitoring dashboard (CPU/memory/disk gauges + sparklines per distro)
- Embedded terminal pane (ConPTY) — users stay in TUI while running commands
- Termius connection mode with SSH auto-setup
- Named snapshots with timestamps and metadata stored in libsql
- Lua plugin runtime (mlua) with sandboxed permissions
- Command palette (`:` prefix) with fuzzy matching via nucleo
- Resource history logging queryable from monitor view

**Defer (v2+):**
- WASM plugin runtime — defer until Lua API is stable and documented
- wsl-web binary (Axum REST API + embedded SPA) — defer until TUI is stable
- Pack import by URL — local file import covers v1 use case
- Settings view (TUI config editor) — config.toml works for power users
- Remote WSL management — triples scope; out of v1

See `.planning/research/FEATURES.md` for full feature dependency graph and prioritization matrix.

### Architecture Approach

The architecture is a Cargo workspace with a strict dependency hierarchy: two independent binaries (`wsl-tui`, `wsl-web`) both depending on a shared library (`wsl-core`) that contains all business logic. `wsl-core` must contain zero UI imports. Functional domains are separated into compile-time plugin crates (`wsl-plugin-distro`, `wsl-plugin-provision`, `wsl-plugin-monitor`, `wsl-plugin-backup`, `wsl-plugin-connect`) that implement a `Plugin` trait and are registered at startup via a `PluginRegistry`. The TUI binary follows the Elm Architecture (Model-Update-View) with a tokio mpsc channel bridging the async WSL command layer to the synchronous render loop. `resolver = "2"` in the workspace root is mandatory to prevent feature unification across binaries.

**Major components:**
1. **wsl-core** — All shared business logic: WSL executor, storage backend trait (libsql + JSON impl), plugin host and registry, domain types, config loading; no UI code
2. **wsl-tui** — Ratatui/crossterm event loop, App state, Action enum, view components (one file per screen), theme constants
3. **wsl-plugin-provision** — TOML pack parser, dependency resolver (`resolver.rs`), step executor (`executor.rs`), idempotency checker (`checker.rs`); the most complex plugin crate
4. **wsl-plugin-connect** — portable-pty embedded terminal (`pty.rs`), external terminal launchers, Termius SSH integration
5. **wsl-web** — Axum router, REST handlers mirroring TUI functionality, rust-embed SPA assets, WebSocket for real-time metrics
6. **Plugin trait** — `Plugin: Send + Sync` with `init(Arc<PluginContext>)`, `commands()`, `views()`, `shutdown()` — contract for both compile-time and Lua runtime plugins

See `.planning/research/ARCHITECTURE.md` for full data flow diagrams, pattern examples with code, and anti-patterns.

### Critical Pitfalls

Six critical pitfalls identified. All six are preventable in Phase 1 or the specific phase where they arise; none require architectural rethinking if caught early.

1. **libsql stack overflow on Windows** — Add `.cargo/config.toml` with `/STACK:8000000` linker flag before any database work; verify with a Windows smoke test (create table + insert + query exits cleanly). Recovery is a one-line fix if caught early; a refactor if discovered late.

2. **wsl.exe UTF-16LE / UTF-8 encoding ambiguity** — Detect `WSL_UTF8` env var before parsing any `wsl.exe` output; or force `WSL_UTF8=1` on the child process environment and always parse as UTF-8. Hard-coding UTF-16LE silently garbles output for users with the env var set (VS Code shipped this bug).

3. **Crossterm duplicate key events on Windows** — Filter all key events with `key.kind == KeyEventKind::Press` in the event loop skeleton. Windows fires both Press and Release; every action fires twice without this guard.

4. **No panic hook = corrupted terminal** — Use `ratatui::init()` and `ratatui::restore()` in `main()` before any other initialization. A panic without this leaves the terminal in raw mode and makes the shell unusable.

5. **Cargo workspace feature unification** — Set `resolver = "2"` in root `Cargo.toml` immediately. Resolver v1 enables features globally across binaries, causing phantom C-library build failures for the "wrong" binary.

6. **PTY/ConPTY architecture mismatch** — Use `portable-pty` for the embedded terminal; never `nix::pty` (Linux-only) or raw WinPTY (legacy). Allocate a dedicated spike for PTY before committing to the embedded terminal as a shipped feature.

See `.planning/research/PITFALLS.md` for integration gotchas, performance traps, security mistakes, and the full "Looks Done But Isn't" verification checklist.

---

## Implications for Roadmap

Based on the combined research, seven phases are recommended. The ordering respects the hard dependency graph from FEATURES.md (`Distro List required by everything`, `Storage Backend required by state features`, `Pack Engine required by provisioning features`) and the architectural build order from ARCHITECTURE.md.

### Phase 1: Foundation
**Rationale:** Everything else depends on a working scaffold. The four critical pitfalls (libsql stack, encoding, duplicate keys, panic hook) must be resolved before any feature work begins. Resolver v2, MSRV, workspace structure, and the storage/WSL/plugin abstractions must all be established first.
**Delivers:** Compilable workspace skeleton; libsql working on Windows; wsl.exe output parsing with encoding detection; TUI event loop with KeyEventKind filter and panic hook; all domain types; StorageBackend trait + libsql and JSON impls; Plugin trait + PluginRegistry; config.toml loading.
**Addresses (from FEATURES.md):** Configurable keybindings foundation; config.toml structure
**Avoids (from PITFALLS.md):** Pitfalls 1–5 (libsql overflow, encoding, duplicate keys, panic hook, resolver unification)
**Research flag:** Standard patterns — HIGH confidence. No additional research needed.

### Phase 2: Core Distro Management TUI
**Rationale:** Distro enumeration is the root dependency for everything. The first working version must let users see, manage, and connect to distros before any differentiators are added. This is the MVP that drives early GitHub traction.
**Delivers:** wsl-plugin-distro (list, install from online list, start, stop, terminate, remove, set default, export/import .tar); wsl-tui binary with Dashboard view, shell attach, vim nav, help overlay, fuzzy filter, Catppuccin Mocha theme, status bar, responsive layout.
**Uses (from STACK.md):** ratatui 0.30, crossterm 0.29, catppuccin 2.4, tokio::process::Command, nucleo for fuzzy filter
**Implements (from ARCHITECTURE.md):** TEA event loop, App state + Action enum, component-per-view structure, wsl-plugin-distro
**Avoids:** Blocking wsl.exe calls on render thread (use tokio::spawn + mpsc channel); KeyEventKind::Press filter
**Research flag:** Standard patterns — well-documented. No additional research needed.

### Phase 3: Provisioning Pack Engine
**Rationale:** The pack system is the core differentiator. It requires the storage layer (from Phase 1) and a working distro list (Phase 2). It is the feature that creates retention and community sharing — must be delivered before monitoring or connectivity to maximize differentiation impact.
**Delivers:** wsl-plugin-provision (TOML parser, dependency resolver, step executor, idempotency checker); interactive provisioning wizard; dry-run mode; all 9 built-in packs; custom TOML pack import; re-provision existing distros.
**Uses (from STACK.md):** toml 0.8, libsql for idempotency state, serde for pack deserialization
**Implements (from ARCHITECTURE.md):** wsl-plugin-provision with resolver.rs, executor.rs, checker.rs
**Avoids:** Pack engine must not block the render thread; all step execution is async and sends progress events back via mpsc
**Research flag:** Needs research-phase during planning. Idempotency design for the step executor (how to check "already done" for diverse step types like apt install, git clone, shell commands) is a design decision that benefits from targeted research.

### Phase 4: Monitoring and Backup
**Rationale:** Resource monitoring and named snapshots add operational depth for homelab and power users. Backup (export/import) is table stakes; named snapshots are a differentiator. Both are independent of the pack engine and can be built in parallel with provisioning if bandwidth allows, but should not block it.
**Delivers:** wsl-plugin-monitor (CPU/memory/disk per distro via /proc polling, sparklines, gauges); resource history logging to libsql; named snapshots with metadata; Monitor and Backup TUI views; WSL kernel update from TUI.
**Uses (from STACK.md):** sysinfo 0.37, libsql for history logging, chrono for timestamps, uuid for snapshot IDs
**Implements (from ARCHITECTURE.md):** wsl-plugin-monitor, wsl-plugin-backup; configurable tokio::time::interval for polling (not busy-loop)
**Avoids:** Do not poll on every render frame (60 calls/sec); use 1–5s configurable interval; debounce sparkline renders
**Research flag:** Standard patterns — monitoring polling and libsql inserts are well-documented. No additional research needed.

### Phase 5: Full Connectivity (Embedded PTY)
**Rationale:** Shell attach (shipped in Phase 2) satisfies the connection use case for most users. The embedded PTY, external terminal launch, and Termius integration complete the connection mode matrix but carry significantly more complexity. PTY in particular requires a dedicated spike on Windows before the feature is committed.
**Delivers:** wsl-plugin-connect with embedded ConPTY terminal pane (portable-pty), external terminal launch with configurable template (wt.exe, Alacritty, WezTerm), Termius connection mode with SSH provisioning pack.
**Uses (from STACK.md):** portable-pty 0.8 (ConPTY on Windows); tokio::process for external launch
**Implements (from ARCHITECTURE.md):** wsl-plugin-connect with pty.rs and external.rs
**Avoids:** WinPTY (legacy); nix::pty (Linux-only); test PTY against all four supported terminals before shipping
**Research flag:** Needs research-phase during planning. ConPTY + Ratatui embedded terminal integration is the most under-documented feature in the stack. Spike required before design.

### Phase 6: Extensibility (Lua Plugin Runtime)
**Rationale:** The Lua plugin system enables community adoption and long-tail customization, but depends on a stable plugin API (defined in Phases 1–5). It must be built after the API surface is known to avoid plugin API breakage before the community has built anything.
**Delivers:** mlua runtime loader in wsl-core/plugin/lua.rs; LuaPlugin impl of Plugin trait; sandboxed Lua environment (no os/io/package stdlib); plugin permission model and approval flow; plugin API versioning (API_VERSION constant).
**Uses (from STACK.md):** mlua 0.11.5 with lua54 + vendored + async features; lazy-load Lua VM only if plugins/ dir is non-empty
**Implements (from ARCHITECTURE.md):** PluginRegistry::load_lua_plugins(); Lua UserData types for Distro, Pack, BackupEntry
**Avoids:** Full Lua stdlib access (security risk); VM recreation per invocation (performance trap); missing API version constant (silent breakage on upgrade)
**Research flag:** Needs research-phase during planning. Lua sandbox design (which stdlib subsets are safe, how to expose wsl-core types as UserData) has limited Rust-specific documentation.

### Phase 7: Web UI
**Rationale:** The wsl-web binary is orthogonal to the TUI — it shares wsl-core but adds no value to TUI users. It is a force multiplier for CI/CD workflows and browser-based management. Defer until the TUI and wsl-core API are fully stable to avoid building REST endpoints that change under it.
**Delivers:** wsl-web binary (Axum REST API, embedded SPA via rust-embed, WebSocket for real-time metrics); REST endpoints mirroring all TUI functionality; SPA frontend with Catppuccin Mocha theme.
**Uses (from STACK.md):** axum 0.8, rust-embed 8.11, axum-embed 0.1, tower-http 0.6 (CORS + compression + trace), tokio-tungstenite 0.26
**Implements (from ARCHITECTURE.md):** Pattern 5 (Axum + embedded SPA); WebSocket broadcast channel for monitoring
**Avoids:** Binding to 0.0.0.0 (bind 127.0.0.1 only); permissive CORS (allowlist localhost origins only); libsql connection-per-request (single Arc<Connection>)
**Research flag:** Standard patterns — Axum + rust-embed SPA serving is well-documented. CORS config needs security review. No additional research needed beyond team familiarity with Axum 0.8 router API.

### Phase Ordering Rationale

- **Phase 1 before everything:** Five of six critical pitfalls are Phase 1 concerns. A broken storage layer, broken key handling, or broken terminal restore will corrupt every subsequent phase.
- **Phase 2 before Phase 3:** The pack engine runs commands against specific distros; a reliable distro list and shell attach are prerequisites.
- **Phase 3 before Phase 6:** The Lua plugin API must reflect the stable plugin command/view surface. Building the plugin runtime before the API stabilizes causes breakage.
- **Phase 5 after Phase 2:** Shell attach (Phase 2) is the simple connection mode. The embedded PTY is the rich version — it must be validated after the simple version is shipping and the team understands the connection lifecycle.
- **Phase 7 last:** wsl-web adds zero value to TUI users and should not influence wsl-core API decisions during active TUI development.

### Research Flags

Phases that benefit from `/gsd:research-phase` during planning:
- **Phase 3 (Provisioning Pack Engine):** Idempotency design for heterogeneous step types is a non-trivial design decision; research existing Ansible-lite and provisioning tool patterns for step state tracking.
- **Phase 5 (Connectivity / PTY):** ConPTY + Ratatui embedded terminal has sparse Rust-specific documentation; a spike is required before committing the feature design.
- **Phase 6 (Lua Plugin Runtime):** mlua sandbox design (safe stdlib subsets, UserData exposure, permission model) has limited authoritative documentation; research Neovim's Lua API design as a reference implementation.

Phases with standard patterns (research-phase can be skipped):
- **Phase 1 (Foundation):** Workspace setup, storage traits, event loop scaffolding — all patterns are well-documented in official Cargo, Ratatui, and tokio docs.
- **Phase 2 (Core TUI):** Distro lifecycle commands, TEA event loop, Ratatui component architecture — official ratatui templates cover all patterns.
- **Phase 4 (Monitoring/Backup):** Resource polling with tokio intervals and libsql storage — standard async patterns.
- **Phase 7 (Web UI):** Axum + rust-embed SPA serving — official Axum and rust-embed docs cover all patterns.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Core crates (ratatui 0.30, axum 0.8, crossterm 0.29, mlua 0.11, rust-embed 8.11) confirmed via official release notes and docs.rs. sysinfo and libsql versions at MEDIUM (search-derived, not direct crates.io fetch). wslapi crate at LOW — use only as fallback. |
| Features | HIGH | Backed by official WSL docs, direct competitor GitHub inspection, and authoritative TUI project sources (lazygit, k9s, btop). MVP scope is clearly bounded. |
| Architecture | HIGH (patterns) / MEDIUM (plugin system) | Ratatui TEA, Cargo workspace, Axum SPA patterns all sourced from official docs. Plugin system and PTY integration confidence is MEDIUM — well-reasoned but less battle-tested in this specific combination. |
| Pitfalls | MEDIUM-HIGH | libsql/Windows stack overflow: confirmed open GitHub issue. Encoding: confirmed VS Code production bug. Duplicate keys: multiple independent reproductions. PTY: MEDIUM — based on documented ConPTY vs WinPTY tradeoffs, limited production Rust data. |

**Overall confidence:** HIGH

### Gaps to Address

- **wslapi crate viability (LOW confidence):** Version 0.1.3 has not been actively updated. At Phase 2 implementation time, evaluate whether to use it, bind the `windows` crate directly, or rely entirely on `wsl.exe` shell-out. Document the decision in wsl-core/wsl/native.rs.
- **libsql version pin (MEDIUM confidence):** Version 0.9.29 was derived from search, not a direct crates.io fetch. Pin the version in Cargo.toml on Phase 1 day one and verify the Windows stack overflow workaround applies to that exact version.
- **sysinfo MSRV ceiling:** sysinfo 0.37.2 sets MSRV at 1.88. If a future sysinfo patch raises MSRV further, the workspace must track it. Watch sysinfo release notes.
- **Pack idempotency design:** The research identifies this as a design decision but does not resolve the schema for step state tracking. This is the primary open question for Phase 3 planning.
- **ConPTY + Ratatui integration:** No authoritative Rust example was found for embedding a ConPTY terminal as a Ratatui pane. The spike in Phase 5 is mandatory before design commitment.
- **Catppuccin in 256-color fallback (ConHost):** ConHost may not support truecolor. The `catppuccin` crate provides exact palette values; the app should detect `COLORTERM` and degrade gracefully or warn the user.

---

## Sources

### Primary (HIGH confidence)
- [Ratatui 0.30.0 highlights and release](https://ratatui.rs/highlights/v030/) — version, MSRV, crossterm compatibility, TEA and component patterns
- [Axum 0.8.0 announcement and CHANGELOG](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) — version, MSRV, WebSocket, macros
- [Official Cargo Workspaces docs](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) — workspace patterns, resolver v2
- [Ratatui async template structure](https://ratatui.github.io/async-template/02-structure.html) — event loop, mpsc channel pattern
- [Ratatui Panic Hooks recipe](https://ratatui.rs/recipes/apps/panic-hooks/) — panic hook requirement
- [Microsoft Learn: Basic WSL Commands](https://learn.microsoft.com/en-us/windows/wsl/basic-commands) — wsl.exe command surface
- [crossterm KeyEventKind Windows issue (ratatui/ratatui #347)](https://github.com/ratatui/ratatui/issues/347) — duplicate key event confirmation
- [VSCode WSL_UTF8 encoding bug (microsoft/vscode #276253)](https://github.com/microsoft/vscode/issues/276253) — encoding ambiguity confirmation
- [mlua repository](https://github.com/mlua-rs/mlua) — Lua 5.4 bindings, vendored feature
- [rust-embed 8.11.0](https://docs.rs/crate/rust-embed/latest) — release date, API
- [crossterm 0.29.0 on crates.io](https://crates.io/crates/crossterm/0.29.0) — version, download count
- [tokio releases](https://github.com/tokio-rs/tokio/releases) — LTS schedule
- [Cargo Workspaces resolver = "2" (The Cargo Book)](https://doc.rust-lang.org/cargo/reference/workspaces.html) — resolver v2 requirement

### Secondary (MEDIUM confidence)
- [wsl2-distro-manager GitHub](https://github.com/bostrot/wsl2-distro-manager) — competitor feature inventory
- [wsl-gui-tool GitHub](https://github.com/emeric-martineau/wsl-gui-tool) — competitor feature inventory
- [libsql crates.io](https://crates.io/crates/libsql) — version 0.9.29 (search-derived)
- [libsql + tokio Windows stack overflow (GitHub issue #1051)](https://github.com/tursodatabase/libsql/issues/1051) — Windows stack overflow workaround
- [sysinfo 0.37.2 on crates.io](https://crates.io/crates/sysinfo) — MSRV 1.88, download count
- [portable-pty docs.rs](https://docs.rs/portable-pty) — ConPTY support on Windows
- [Cargo Workspace Feature Unification Pitfall (nickb.dev)](https://nickb.dev/blog/cargo-workspace-and-the-feature-unification-pitfall/) — verified against official Cargo docs
- [catppuccin rust crate](https://github.com/catppuccin/rust) — official crate, ratatui feature
- [nucleo GitHub](https://github.com/helix-editor/nucleo) — Helix fuzzy matcher
- [Rust plugin system techniques](https://nullderef.com/blog/plugin-tech/) — plugin patterns
- [lazygit, k9s, btop](https://github.com/jesseduffield/lazygit) — TUI navigation patterns

### Tertiary (LOW confidence)
- [wslapi 0.1.3 docs.rs](https://docs.rs/wslapi/latest/wslapi/) — old version, limited maintenance signals; validate at Phase 2 implementation time
- [pseudoterminal crate](https://github.com/michaelvanstraten/pseudoterminal) — small crate, limited production use data; use portable-pty instead
- [Termius WSL documentation](https://www.sabbirz.com/blog/configure-wsl-inside-termius) — community blog; validate at Phase 5

---
*Research completed: 2026-02-21*
*Ready for roadmap: yes*
