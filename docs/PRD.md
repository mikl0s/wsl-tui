# Product Requirements Document (PRD)

**Project:** WSL TUI
**Date:** 2026-02-21
**Author:** Mikkel Georgsen
**Version:** 1.0

---

## 1. Product Overview

WSL TUI is a Rust-based terminal user interface for managing WSL2 on Windows 11. It combines distro lifecycle management, a stackable provisioning pack system, resource monitoring, backup/restore, and multi-mode connectivity into a polished, modern TUI.

## 2. User Stories

### Distro Management
- **US-1:** As a user, I can see all installed WSL distros with their state (Running/Stopped), WSL version, and resource usage at a glance.
- **US-2:** As a user, I can install a new distro from the available online list with a single action.
- **US-3:** As a user, I can start, stop, restart, and terminate individual distros.
- **US-4:** As a user, I can set a default distro.
- **US-5:** As a user, I can remove (unregister) a distro with a confirmation prompt.
- **US-6:** As a user, I can update the WSL kernel from within the TUI.

### Provisioning (Stackable Packs)
- **US-10:** As a user, after installing a distro, I am presented with a pack selection screen where I can choose multiple provisioning packs.
- **US-11:** As a user, I can select packs like `home-setup`, `claude-code`, `nvm-node` and have them all applied automatically.
- **US-12:** As a user, each pack prompts me for variables (which shell? which Node version?) through an interactive wizard modal.
- **US-13:** As a user, packs are applied idempotently — re-running a pack skips already-applied steps.
- **US-14:** As a user, I can dry-run a pack to see what would change before applying.
- **US-15:** As a user, I can create and save custom packs as TOML files.
- **US-16:** As a user, I can share packs with others by sharing TOML files.
- **US-17:** As a user, I can re-provision an existing distro by selecting additional packs or re-running existing ones.

### Connection
- **US-20:** As a user, I can connect to a distro via shell attach (suspends TUI, drops into shell).
- **US-21:** As a user, I can connect via an embedded terminal pane within the TUI.
- **US-22:** As a user, I can launch a distro in Windows Terminal (or my configured external terminal).
- **US-23:** As a user, I can launch a distro in Termius with automatic SSH setup.
- **US-24:** As a user, I can configure my preferred default connection mode per distro or globally.

### Monitoring
- **US-30:** As a user, I can see real-time CPU, memory, and disk usage for each running distro.
- **US-31:** As a user, I can view a full-screen monitoring dashboard with charts and gauges.
- **US-32:** As a user, resource usage history is logged and queryable.

### Backup & Restore
- **US-40:** As a user, I can export a distro to a `.tar` or `.vhdx` file.
- **US-41:** As a user, I can import a distro from a `.tar` or `.vhdx` file.
- **US-42:** As a user, I can create named snapshots of distros.
- **US-43:** As a user, I can see backup history with timestamps and file sizes.

### Storage
- **US-50:** As a user, the app automatically uses libsql embedded storage (no install needed).
- **US-51:** As a user, if libsql fails, the app falls back to JSON file storage transparently.
- **US-52:** As a user, I can override the storage backend in `config.toml`.
- **US-53:** As a user, I can install the libsql CLI tool via `wsl-tui setup install-libsql-cli` for external DB inspection.

### Plugins
- **US-60:** As a user, built-in plugins provide all core functionality (distro, provision, monitor, backup, connect).
- **US-61:** As a user, I can install Lua plugins in `~/.wsl-tui/plugins/` to extend functionality.
- **US-62:** As a user, runtime plugins request permissions on first load and I can approve/deny.
- **US-63:** As a user, a failing plugin never crashes the application.

### Settings & Config
- **US-70:** As a user, all configuration is in `~/.wsl-tui/config.toml`.
- **US-71:** As a user, I can edit settings from within the TUI settings view.
- **US-72:** As a user, keybindings are configurable.

## 3. Functional Requirements

### FR-1: WSL Command Execution
- Execute `wsl.exe` commands via `std::process::Command`
- Handle UTF-16LE output encoding
- Supplement with `wslapi` crate for native API access
- Timeout handling for long-running operations (install, export)

### FR-2: Storage Abstraction
- `StorageBackend` trait with `LibsqlBackend` and `JsonBackend` implementations
- Auto-detection at startup with configurable override
- All operations wrapped in transactions (libsql) or atomic file writes (JSON)
- Migration support for schema changes across versions

### FR-3: Pack Engine
- Parse TOML pack definitions
- Resolve dependency graph (topological sort)
- Detect conflicts before execution
- Execute steps sequentially within a pack, packs in dependency order
- Per-step idempotency checking via `StepState`
- Per-step retry on failure
- Resume from last failed step
- Real-time progress reporting via callback/channel
- Dry-run mode producing a diff/plan output

### FR-4: Plugin System
- Plugin trait with lifecycle hooks (init, shutdown)
- Plugin registry with ordered initialization
- Lua runtime (mlua) for Phase 1 runtime plugins
- Sandboxed execution environment for runtime plugins
- Permission model: declare + approve

### FR-5: TUI Rendering
- Ratatui with crossterm backend
- Immediate-mode rendering
- Responsive layout (adapts to terminal size)
- Filter for `KeyEventKind::Press` only (Windows crossterm quirk)
- Modal/overlay support for provisioning wizard
- Status bar with storage indicator, active distro, clock

### FR-6: Connection Manager
- Shell attach: spawn `wsl -d <distro>` as child process, restore TUI on exit
- Embedded terminal: PTY via `portable-pty`/`conpty`, split pane rendering
- External launch: configurable command template with `{distro_name}` substitution
- Termius: SSH server provisioning, port management, Termius launch

## 4. Non-Functional Requirements

| Requirement | Target |
|-------------|--------|
| Startup time | < 500ms to first render |
| Memory usage | < 50MB idle |
| Binary size | < 30MB (without WASM runtime) |
| Distro list refresh | < 1 second |
| Pack application | Real-time progress feedback |
| Platform | Windows 11 (build 22000+) |
| Terminal compatibility | Windows Terminal, ConHost, Alacritty, WezTerm |

## 5. Built-in Packs (v1)

| Pack | Steps | Variables |
|------|-------|-----------|
| `home-setup` | Shell install, prompt theme, editor, .ssh sync, dotfiles | shell, prompt, editor |
| `claude-code` | Install Claude Code CLI, configure | API key (optional) |
| `nvm-node` | Install nvm, Node LTS, global packages | Node version, packages |
| `python-dev` | Install pyenv, Python, poetry/uv | Python version, tool |
| `rust-dev` | Install rustup, stable toolchain, tools | Toolchain, components |
| `docker` | Install Docker Engine, compose, daemon config | Storage driver |
| `ai-stack` | Install ollama, open-webui, GPU config | Models to pull |
| `gui-desktop` | X11/Wayland setup, GUI apps | Display server, apps |
| `server-base` | nginx/caddy, certbot, firewall | Web server choice |

## 6. Release Strategy

| Phase | Scope | Target |
|-------|-------|--------|
| **v0.1** | Core management: list/start/stop/install/remove + dashboard UI | MVP |
| **v0.2** | Pack provisioning system + built-in packs | Core differentiator |
| **v0.3** | All connection modes (shell, embedded, external, Termius) | Full connectivity |
| **v0.4** | Resource monitoring + backup/restore | Operational features |
| **v0.5** | Lua plugin system + settings UI | Extensibility |
| **v1.0** | Polish, performance, documentation, community packs | Stable release |
| **v1.x** | wsl-web binary, WASM plugins, pack marketplace | Future |
