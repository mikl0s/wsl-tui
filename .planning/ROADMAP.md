# Roadmap: WSL TUI

## Overview

Seven phases build the project from a verified Rust scaffold to a fully-featured TUI plus optional web companion. Phase 1 plants all critical foundations (storage, encoding, event loop, plugin trait) that every subsequent phase depends on. Phases 2 through 4 deliver the core user value in dependency order: distro management first, then the provisioning pack engine (the primary differentiator), then monitoring and backup. Phase 5 completes the connection mode matrix. Phase 6 opens the plugin API to Lua at runtime. Phase 7 ships the web binary as a force multiplier for CI/CD and browser access. Each phase delivers a verifiable, independently runnable capability.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Foundation** - Compilable workspace scaffold with storage, WSL executor, plugin registry, and all Windows platform quirks resolved
- [ ] **Phase 2: Core Distro Management TUI** - Full distro lifecycle and shell attach in a polished, Catppuccin-themed TUI with vim navigation
- [ ] **Phase 3: Provisioning Pack Engine** - Stackable TOML packs with idempotency, dry-run, wizard, and all 9 built-in packs
- [ ] **Phase 4: Monitoring and Backup** - Real-time resource gauges, historical metrics, and named distro snapshots
- [ ] **Phase 5: Connectivity** - External terminal launch and Termius SSH integration completing the connection mode matrix
- [ ] **Phase 6: Extensibility** - Lua plugin runtime with sandboxing, permissions, and settings + command palette views
- [ ] **Phase 7: Web UI** - Axum REST API + embedded SPA binary with real-time WebSocket metrics

## Phase Details

### Phase 1: Foundation
**Goal**: The workspace compiles cleanly, all Windows platform quirks are resolved at the source, and every abstraction that downstream phases depend on is in place
**Depends on**: Nothing (first phase)
**Requirements**: FOUND-01, FOUND-02, FOUND-03, FOUND-04, FOUND-05, FOUND-06, FOUND-07, FOUND-08, FOUND-09, FOUND-10, DX-01, DX-02, DX-03, DX-04, DX-05, DX-06, DX-07
**Success Criteria** (what must be TRUE):
  1. `cargo build --workspace` and `cargo clippy --workspace` complete with zero errors and zero warnings on Windows 11
  2. A smoke test creates a libsql table, inserts a row, and queries it cleanly without a stack overflow on Windows
  3. A smoke test calls `wsl.exe --list --verbose` and parses the output correctly regardless of whether `WSL_UTF8` is set in the environment
  4. The TUI event loop launches, renders a placeholder frame, and exits cleanly on `q` without leaving the terminal in raw mode after both normal exit and a simulated panic
  5. CLAUDE.md exists at the repo root and in each crate with coding standards and architecture context
**Plans**: 4 plans

Plans:
- [x] 01-01-PLAN.md — Cargo workspace scaffold + config system (resolver v2, crate stubs, linker flag, error types, config loading with env overrides)
- [x] 01-02-PLAN.md — Storage backend (StorageBackend trait, LibsqlBackend with smoke test, JsonBackend, open_storage factory with auto-fallback)
- [ ] 01-03-PLAN.md — WSL executor + Plugin registry + TUI skeleton (encoding detection, Plugin trait, PluginRegistry, event loop with KeyEventKind filter, panic hook, welcome screen)
- [ ] 01-04-PLAN.md — CLAUDE.md living documents (root + wsl-core, wsl-tui, wsl-web crate docs, coding standards)

### Phase 2: Core Distro Management TUI
**Goal**: Users can see all their WSL distros, manage their full lifecycle, connect via shell attach, and navigate a polished themed TUI — this is the first shippable version
**Depends on**: Phase 1
**Requirements**: DIST-01, DIST-02, DIST-03, DIST-04, DIST-05, DIST-06, DIST-07, DIST-08, DIST-09, DIST-10, CONN-01, TUI-01, TUI-07, TUI-08, TUI-09, TUI-10, TUI-12, TUI-13, TUI-14, TUI-15
**Success Criteria** (what must be TRUE):
  1. User launches `wsl-tui` and sees all installed distros with Running/Stopped state, WSL version, and default indicator within 500ms
  2. User can install a new distro from the online list and watch per-step progress feedback without the UI freezing
  3. User can start, stop, terminate, set default, and remove (with confirmation) any distro using keyboard actions
  4. User can export a distro to a `.tar` file and import a `.tar` as a new distro from within the TUI
  5. User presses `Enter` (or configured key) on a running distro and drops into a shell; closing the shell returns them to the TUI with layout restored
  6. Pressing `?` shows context-aware help, `/` opens fuzzy filter, number keys 1-5 switch views, and all actions work via vim-style hjkl navigation
**Plans**: TBD

Plans:
- [ ] 02-01: wsl-plugin-distro (list with encoding-safe parsing, install with progress stream, start/stop/terminate/set-default/remove)
- [ ] 02-02: Export/import .tar, WSL kernel update, shell attach (TUI suspend + wsl.exe + restore)
- [ ] 02-03: Dashboard view, status bar, Catppuccin Mocha theme, responsive layout, vim navigation
- [ ] 02-04: Help overlay, fuzzy filter, number-key view switching, configurable keybindings

### Phase 3: Provisioning Pack Engine
**Goal**: Users can go from a bare distro to a fully provisioned dev environment by selecting packs from a wizard, with full visibility into what will change before it happens
**Depends on**: Phase 2
**Requirements**: PROV-01, PROV-02, PROV-03, PROV-04, PROV-05, PROV-06, PROV-07, PROV-08, PROV-09, PROV-10, PROV-11, PROV-12, TUI-02
**Success Criteria** (what must be TRUE):
  1. User opens the Provision view, selects multiple packs, and the wizard prompts for per-pack variables (shell choice, Node version, etc.) before executing
  2. User runs dry-run and sees a readable list of exactly which steps would execute and which would be skipped (already applied), with no changes made to the distro
  3. Provisioning runs all selected packs in correct dependency order; if a step fails the user can retry from that step without re-running prior steps
  4. Re-running provisioning on a distro where packs are already applied skips those steps and reports them as already done
  5. User can load a custom TOML pack from `~/.wsl-tui/packs/` and provision with it alongside built-in packs
**Plans**: TBD

Plans:
- [ ] 03-01: TOML pack parser, dependency resolver (topological sort), conflict detection
- [ ] 03-02: Step executor with idempotency checker, retry from failed step, progress events via mpsc
- [ ] 03-03: Interactive provisioning wizard (variable prompts, dry-run mode), pack state persistence in libsql
- [ ] 03-04: All 9 built-in packs (home-setup, claude-code, nvm-node, python-dev, rust-dev, docker, ai-stack, gui-desktop, server-base) + custom pack import
- [ ] 03-05: Provision view (modal overlay with pack selection, wizard, execution progress)

### Phase 4: Monitoring and Backup
**Goal**: Users can observe real-time resource consumption for running distros and create, browse, and restore named snapshots
**Depends on**: Phase 2
**Requirements**: MON-01, MON-02, MON-03, MON-04, MON-05, MON-06, BACK-01, BACK-02, BACK-03, TUI-03, TUI-04, TUI-05
**Success Criteria** (what must be TRUE):
  1. Monitor view shows CPU and memory gauges updating every 5 seconds (configurable) for each running distro without blocking navigation
  2. Full-screen monitor view renders sparklines showing metric history for the current session; user can see disk usage per distro
  3. User can create a named snapshot with a description and see it appear in the snapshot list with timestamp and file size
  4. User can restore any named snapshot to a running distro and the operation completes with status feedback
  5. Logs view shows scrollable execution history that the user can filter by distro or action type
**Plans**: TBD

Plans:
- [ ] 04-01: wsl-plugin-monitor (sysinfo polling via tokio interval, CPU/memory/disk per distro, sparkline data accumulation, history logging to libsql)
- [ ] 04-02: wsl-plugin-backup (named snapshots with metadata, snapshot list, restore, uuid + chrono integration)
- [ ] 04-03: Monitor view (gauges, sparklines, per-distro breakdown), Backup view (snapshot manager), Logs view (scrollable + filterable)

### Phase 5: Connectivity
**Goal**: Users have the full connection mode matrix — external terminal launch and Termius SSH — covering every workflow beyond shell attach
**Depends on**: Phase 2
**Requirements**: CONN-02, CONN-03, CONN-04, CONN-05, CONN-06
**Success Criteria** (what must be TRUE):
  1. User can launch a distro in Windows Terminal, Alacritty, or WezTerm with a single keypress; the external terminal command uses `{distro_name}` substitution
  2. User can connect to a distro via Termius; the SSH server is automatically provisioned on first use with a per-distro port derived from base_port + offset
  3. User can set a default connection mode globally in settings and override it per distro; the configured mode is used when pressing the connect key
**Plans**: TBD

Plans:
- [ ] 05-01: External terminal launcher (configurable command template, Windows Terminal/Alacritty/WezTerm support)
- [ ] 05-02: Termius connection mode (SSH server auto-provisioning pack, per-distro port mapping, per-distro and global default configuration)

### Phase 6: Extensibility
**Goal**: Community Lua plugins load safely, run sandboxed, and can read distro state and trigger pack operations — and the TUI gains settings and command palette views
**Depends on**: Phase 3 (stable plugin API surface)
**Requirements**: EXT-01, EXT-02, EXT-03, EXT-04, EXT-05, TUI-06, TUI-11
**Success Criteria** (what must be TRUE):
  1. A Lua file dropped in `~/.wsl-tui/plugins/` loads on next startup and its declared commands appear in the command palette
  2. A Lua plugin that calls a forbidden stdlib function (os, io, package) is refused at load time with a clear error; the host TUI continues running
  3. User approves or denies a plugin's declared permissions on first load; approval is persisted so the prompt does not reappear on subsequent startups
  4. A Lua plugin that panics or throws a runtime error is caught and logged; all other plugins and the host TUI continue running normally
  5. Pressing `:` opens the command palette with fuzzy-matched commands from both built-in actions and loaded plugins; the Settings view exposes all config.toml options as an in-TUI editor
**Plans**: TBD

Plans:
- [ ] 06-01: mlua runtime loader (lua54 + vendored + async, lazy-load only when plugins/ is non-empty, LuaPlugin impl of Plugin trait)
- [ ] 06-02: Lua sandbox (stdlib allowlist, permission declaration + approval flow, permission persistence in libsql)
- [ ] 06-03: Plugin API (Distro, Pack, BackupEntry UserData types, distro list access, pack operations, notification system)
- [ ] 06-04: Settings view (TUI config editor for config.toml), command palette (`:` with nucleo fuzzy matching, plugin + built-in commands)

### Phase 7: Web UI
**Goal**: Users who prefer a browser or need headless/CI access get a full REST API and embedded SPA that mirrors all TUI functionality
**Depends on**: Phase 6 (stable wsl-core API)
**Requirements**: WEB-01, WEB-02, WEB-03, WEB-04, WEB-05, WEB-06
**Success Criteria** (what must be TRUE):
  1. `wsl-web` starts and serves the SPA at `http://127.0.0.1:3000` with no external files required (all assets embedded in the binary)
  2. Every distro lifecycle action (list, start, stop, terminate, set default, remove, export, import) is available via documented REST endpoints returning consistent JSON
  3. The browser dashboard shows live CPU and memory gauges that update automatically via WebSocket without requiring page refresh
  4. The web UI renders in the Catppuccin Mocha theme matching the TUI visual identity
  5. All API error responses use the `{ "error": "message" }` format; CORS is restricted to localhost origins only
**Plans**: TBD

Plans:
- [ ] 07-01: Axum server scaffold (127.0.0.1:3000 binding, tower-http CORS/compression/tracing, single Arc<Connection> for libsql)
- [ ] 07-02: REST API handlers (distros, packs, monitor, backup, connect, config endpoints mirroring TUI functionality)
- [ ] 07-03: WebSocket/SSE real-time metrics streaming
- [ ] 07-04: SPA frontend (rust-embed embedded assets, Catppuccin Mocha CSS theme, wiring to all REST + WebSocket endpoints)

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7

Note: Phases 4 and 5 both depend on Phase 2 (not on each other) and can be parallelized if bandwidth allows, but are planned to execute sequentially.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 2/4 | In progress | - |
| 2. Core Distro Management TUI | 0/4 | Not started | - |
| 3. Provisioning Pack Engine | 0/5 | Not started | - |
| 4. Monitoring and Backup | 0/3 | Not started | - |
| 5. Connectivity | 0/2 | Not started | - |
| 6. Extensibility | 0/4 | Not started | - |
| 7. Web UI | 0/4 | Not started | - |
