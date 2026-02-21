# Statement of Work (SOW)

**Project:** WSL TUI
**Date:** 2026-02-21
**Author:** Mikkel Georgsen
**Type:** Open Source (MIT License)

---

## 1. Project Description

Development of a Rust-based terminal user interface (TUI) and web interface for managing WSL2 on Windows 11. Delivered as a Cargo workspace monorepo producing two binaries (`wsl-tui` and `wsl-web`) sharing a common core library.

## 2. Deliverables

### D1: wsl-core Library Crate
- Plugin trait and registry system
- WSL command execution layer (wsl.exe + wslapi)
- Storage abstraction (libsql + JSON backends)
- Pack provisioning engine with idempotency
- Configuration management
- Shared data models

### D2: Built-in Plugin Crates
- `wsl-plugin-distro` — Distro lifecycle management
- `wsl-plugin-provision` — Stackable pack provisioning system
- `wsl-plugin-monitor` — Resource monitoring
- `wsl-plugin-backup` — Export/import/snapshot
- `wsl-plugin-connect` — Shell attach, embedded terminal, external launch, Termius

### D3: wsl-tui Binary Crate
- Ratatui-based terminal interface
- Dashboard, provisioning modal, monitor, backup, logs, settings views
- Vim-style keybindings
- Responsive layout
- Catppuccin Mocha theme

### D4: wsl-web Binary Crate
- Axum-based web server
- REST API exposing core functionality
- Embedded SPA frontend
- Shared authentication (local only, no remote auth for v1)

### D5: Built-in Provisioning Packs
- 9 built-in packs: home-setup, claude-code, nvm-node, python-dev, rust-dev, docker, ai-stack, gui-desktop, server-base
- TOML format, documented schema
- Variable prompts and dependency declarations

### D6: Runtime Plugin System
- Phase 1: Lua plugin runtime (mlua)
- Phase 2: WASM plugin runtime (wasmtime)
- Permission model and sandbox

### D7: Documentation
- README.md (GitHub-optimized)
- BRD, PRD, SOW
- Theme guidelines
- Architecture documentation
- Pack authoring guide
- Plugin development guide
- TUI kickoff prompt
- WebUI kickoff prompt

### D8: CI/CD & Distribution
- GitHub Actions CI (build, test, lint, clippy)
- Release workflow producing Windows binaries
- winget package manifest
- Scoop bucket manifest (optional)

## 3. Technical Specifications

### Language & Toolchain
- Rust (stable toolchain, latest stable)
- Cargo workspace
- Edition 2024

### Key Dependencies
| Crate | Purpose | Version |
|-------|---------|---------|
| ratatui | TUI framework | 0.30+ |
| crossterm | Terminal backend | Latest |
| axum | Web server | 0.8+ |
| tokio | Async runtime | 1.x |
| libsql | Embedded database | 0.9+ |
| serde / serde_json / toml | Serialization | Latest |
| clap | CLI parsing | 4.x |
| mlua | Lua scripting | Latest |
| wslapi | Windows WSL API | Latest |
| portable-pty | PTY for embedded terminal | Latest |

### Target Platform
- Windows 11 (build 22000+)
- x86_64 architecture (primary)
- ARM64 (secondary, if CI supports it)

## 4. Work Breakdown

### Phase 1: Foundation
1. Initialize Cargo workspace with all crate scaffolding
2. Implement `wsl-core`: models, config, storage trait
3. Implement libsql and JSON storage backends
4. Implement WSL command execution layer
5. Implement plugin trait and registry
6. Basic TUI shell (app loop, event handling, layout)

### Phase 2: Core Features
7. `wsl-plugin-distro`: list, start, stop, install, remove, set default
8. Dashboard view: distro list, details panel, status bar
9. Pack provisioning engine in `wsl-core`
10. `wsl-plugin-provision`: pack loading, wizard modal, execution
11. Built-in packs (all 9)

### Phase 3: Connectivity & Operations
12. `wsl-plugin-connect`: shell attach, external launch
13. Embedded terminal (PTY integration)
14. Termius integration (SSH provisioning, launch)
15. `wsl-plugin-monitor`: resource collection, gauges, charts
16. `wsl-plugin-backup`: export, import, snapshot management

### Phase 4: Extensibility & Polish
17. Lua plugin runtime
18. Settings view and config editor
19. Help overlay and command palette
20. Fuzzy search/filter
21. Keyboard shortcut customization
22. Error handling polish and recovery flows

### Phase 5: Web UI
23. `wsl-web`: Axum server setup, REST API
24. SPA frontend (framework TBD — likely Leptos or static HTML+JS)
25. API endpoints mirroring plugin commands
26. Embedded static assets

### Phase 6: Release
27. GitHub Actions CI/CD
28. Release binary builds
29. winget manifest
30. Documentation finalization
31. Community launch

## 5. Quality Standards

- **Testing:** Unit tests for core logic, integration tests for WSL commands (mocked), UI snapshot tests
- **Linting:** `clippy` with pedantic warnings, `rustfmt` enforced
- **Documentation:** Rustdoc on all public APIs, user-facing docs in `docs/`
- **Performance:** Benchmarks for startup time and rendering
- **Security:** Plugin sandboxing verified, no arbitrary code execution from untrusted input

## 6. Acceptance Criteria

- [ ] `cargo build --workspace` succeeds with zero warnings
- [ ] `cargo test --workspace` passes all tests
- [ ] `cargo clippy --workspace` passes with no warnings
- [ ] TUI launches on Windows 11, displays distro list within 500ms
- [ ] Can install, start, stop, remove a distro entirely from the TUI
- [ ] Can apply 3+ packs to a fresh distro and have it ready to use
- [ ] All 4 connection modes functional
- [ ] Storage backend auto-detection works (libsql primary, JSON fallback)
- [ ] Lua plugin loads and executes without crashing host
- [ ] wsl-web serves API and responds to all endpoints
- [ ] Binary size under 30MB
