# WSL TUI — Design Document

**Date:** 2026-02-21
**Author:** Mikkel Georgsen
**Status:** Approved

---

## 1. Overview

WSL TUI is a terminal user interface for managing WSL2 on Windows 11. It provides distro lifecycle management, a stackable provisioning pack system, resource monitoring, backup/restore, and multiple connection modes — all in a polished, lazygit-inspired TUI.

The project is a Cargo workspace monorepo producing two binaries:
- **wsl-tui** — Ratatui terminal interface
- **wsl-web** — Axum-powered web interface (sister project, same core)

## 2. Architecture

### 2.1 Workspace Structure

```
wsl-tui/
├── Cargo.toml                    # Workspace manifest
├── crates/
│   ├── wsl-core/                 # Shared library
│   ├── wsl-plugin-distro/        # Plugin: distro management
│   ├── wsl-plugin-provision/     # Plugin: stackable pack provisioning
│   ├── wsl-plugin-monitor/       # Plugin: resource monitoring
│   ├── wsl-plugin-backup/        # Plugin: export/import/snapshot
│   ├── wsl-plugin-connect/       # Plugin: connection modes
│   ├── wsl-tui/                  # Binary: terminal UI
│   └── wsl-web/                  # Binary: web UI
├── packs/                        # Built-in provisioning packs (.toml)
├── docs/
└── README.md
```

### 2.2 Plugin System (Dual: Compile-time + Runtime)

**Compile-time plugins:** Workspace crates implementing the `Plugin` trait. Ship with the binary. Zero overhead.

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&mut self, ctx: &PluginContext) -> Result<()>;
    fn shutdown(&self) -> Result<()>;
    fn commands(&self) -> Vec<Command>;
    fn views(&self) -> Vec<ViewDescriptor>;
    fn status_items(&self) -> Vec<StatusItem>;
}
```

**Runtime plugins (Phase 1 — Lua):** Lua scripts in `~/.wsl-tui/plugins/`. Loaded via `mlua`. Sandboxed. Community-extensible.

**Runtime plugins (Phase 2 — WASM):** `.wasm` modules loaded via `wasmtime`. Sandboxed, multi-language, portable. Adds ~10MB to binary but provides security guarantees.

**Plugin permissions model:** Runtime plugins declare required permissions (filesystem, WSL commands, network). User approves on first load.

### 2.3 WSL Interaction Layer

Primary interface: shell out to `wsl.exe` via `std::process::Command` with UTF-16 output parsing.

Supplementary: `wslapi` crate for native Windows API calls (registration checks, configuration queries).

Key commands wrapped:
- `wsl --list -v` / `--quiet` / `--running` / `--online`
- `wsl -d <distro>` / `wsl -d <distro> -- <command>`
- `wsl --install <distro>` / `--unregister` / `--terminate` / `--shutdown`
- `wsl --export` / `--import`
- `wsl --set-default` / `--set-version` / `--status` / `--update`

**UTF-16 handling:** `wsl.exe` outputs UTF-16LE. Parse with `widestring` crate or `String::from_utf16`.

## 3. Storage

### 3.1 Location

`~/.wsl-tui/` (`%USERPROFILE%\.wsl-tui\` on Windows)

```
~/.wsl-tui/
├── config.toml
├── data.db                  # libsql (embedded)
├── data/                    # JSON fallback
│   ├── distros.json
│   ├── recipes.json
│   ├── history.json
│   └── profiles.json
├── packs/                   # User-created packs
├── plugins/                 # Runtime plugins
├── logs/
└── backups/
```

### 3.2 Backend Detection

1. Check `config.toml` for explicit `storage_backend` override
2. If unset, initialize embedded libsql (via `libsql` crate with `core` feature — no external install needed)
3. If libsql init fails, fall back to JSON file storage
4. Optional: `wsl-tui setup install-libsql-cli` installs the external CLI via winget for DB inspection

### 3.3 Schema

```sql
CREATE TABLE distros (
    name TEXT PRIMARY KEY,
    wsl_version INTEGER,
    state TEXT,
    default_user TEXT,
    last_seen TIMESTAMP,
    metadata TEXT
);

CREATE TABLE packs (
    id TEXT PRIMARY KEY,
    name TEXT,
    description TEXT,
    category TEXT,
    steps TEXT,
    variables TEXT,
    depends_on TEXT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

CREATE TABLE pack_applications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    distro_name TEXT,
    pack_id TEXT,
    state TEXT,
    variables_used TEXT,
    applied_at TIMESTAMP,
    hash TEXT
);

CREATE TABLE history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    distro_name TEXT,
    action TEXT,
    plugin TEXT,
    status TEXT,
    output TEXT,
    started_at TIMESTAMP,
    completed_at TIMESTAMP
);

CREATE TABLE profiles (
    id TEXT PRIMARY KEY,
    name TEXT,
    config TEXT,
    created_at TIMESTAMP
);
```

### 3.4 StorageBackend Trait

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn list_distros(&self) -> Result<Vec<Distro>>;
    async fn upsert_distro(&self, distro: &Distro) -> Result<()>;
    async fn list_packs(&self) -> Result<Vec<Pack>>;
    async fn save_pack(&self, pack: &Pack) -> Result<()>;
    async fn delete_pack(&self, id: &str) -> Result<()>;
    async fn log_pack_application(&self, app: &PackApplication) -> Result<()>;
    async fn get_pack_state(&self, distro: &str, pack_id: &str) -> Result<Option<PackApplication>>;
    async fn log_action(&self, entry: &HistoryEntry) -> Result<()>;
    async fn query_history(&self, filter: &HistoryFilter) -> Result<Vec<HistoryEntry>>;
    async fn list_profiles(&self) -> Result<Vec<Profile>>;
    async fn save_profile(&self, profile: &Profile) -> Result<()>;
}
```

## 4. Provisioning System — Stackable Packs

### 4.1 Core Concept

Provisioning uses a **stackable pack system**. Each pack is an independent, idempotent unit of configuration. Users select multiple packs to apply after distro installation. Packs declare dependencies and conflicts.

### 4.2 Pack Model

```rust
pub struct Pack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: PackCategory,
    pub depends_on: Vec<String>,
    pub conflicts_with: Vec<String>,
    pub variables: Vec<Variable>,
    pub steps: Vec<ProvisionStep>,
    pub check: StateCheck,
}

pub enum PackCategory { Home, Dev, Services, AI, GUI, Custom }

pub enum ProvisionStep {
    PackageInstall { manager: PackageManager, packages: Vec<String> },
    ShellSetup { shell: Shell, config: ShellConfig },
    ThemeInstall { theme: ThemeSpec },
    DotfilesClone { repo: String, target: PathBuf },
    ScriptRun { script: String, elevated: bool },
    FileWrite { path: PathBuf, content: String, template: bool },
    ServiceEnable { services: Vec<String> },
    UserCreate { username: String, groups: Vec<String> },
    SshSync { source: SshSource },
    Custom { command: String, description: String },
}
```

### 4.3 Idempotency & State Management

Each step implements state checking:

```rust
pub enum StepState {
    NotApplied,
    Applied { at: DateTime, hash: String },
    Drifted { expected: String, actual: String },
    Failed { at: DateTime, error: String },
}
```

- `PackageInstall` → check `dpkg -l` / `rpm -q`
- `ShellSetup` → check `$SHELL`, framework directory existence
- `FileWrite` → hash comparison
- `ServiceEnable` → `systemctl is-enabled`
- `ScriptRun` → user-defined check command or always re-run

**Dry-run mode:** Show what would change without applying. Renders as a preview diff in TUI.

### 4.4 Built-in Packs

| Pack ID | Name | Category | Description |
|---------|------|----------|-------------|
| `home-setup` | Home Setup | Home | Shell, prompt (starship/p10k), editor, .ssh sync, dotfiles |
| `claude-code` | Claude Code | Dev | Install Claude Code CLI + configuration |
| `nvm-node` | NVM + Node | Dev | nvm + LTS Node.js + configurable global packages |
| `python-dev` | Python Dev | Dev | pyenv + latest Python + poetry/uv |
| `rust-dev` | Rust Dev | Dev | rustup + stable toolchain + common tools |
| `docker` | Docker Engine | Services | Docker Engine + compose + daemon config |
| `ai-stack` | AI Stack | AI | ollama + open-webui + GPU passthrough config |
| `gui-desktop` | GUI Desktop | GUI | X11/Wayland forwarding + common GUI apps |
| `server-base` | Server Base | Services | nginx/caddy + certbot + firewall basics |

### 4.5 Installation Flow

```
1. User selects "Install New Distro"
2. Pick distro from available list (wsl --list --online)
3. WSL installs distro
4. Pack selection screen (multi-select with categories):
   ☑ home-setup      ☐ python-dev
   ☑ claude-code     ☐ rust-dev
   ☑ nvm-node        ☐ docker
   ☐ ai-stack        ☐ gui-desktop
5. Dependency resolution (auto-add deps, warn on conflicts)
6. Variable prompts for selected packs (shell? prompt? Node version?)
7. Execution with real-time progress + log pane
8. Summary: ✓ 3 packs applied, 0 failed. Ubuntu ready.
```

## 5. TUI Interface

### 5.1 Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│  WSL TUI v0.1.0                          ▸ Ubuntu (Running)   ◆ 14:32 │
├──────────────────┬──────────────────────────────────────────────────────┤
│  DISTROS         │  DETAILS                                            │
│  ▸ Ubuntu 24.04  │  Name / State / WSL Version / User / IP             │
│    Debian 12     │  Uptime / Memory / Disk                             │
│    Fedora 41     │                                                      │
│                  │  RESOURCE MONITOR                                    │
│  QUICK ACTIONS   │  CPU / MEM / DISK gauges                            │
│  [s] Start       │                                                      │
│  [x] Stop        │                                                      │
│  [c] Connect     │                                                      │
│  [d] Set Default │                                                      │
│  [p] Provision   │                                                      │
│  [b] Backup      │                                                      │
│  [i] Install New │                                                      │
├──────────────────┴──────────────────────────────────────────────────────┤
│  [Tab] Panel  [/] Search  [?] Help  [:] Command  [q] Quit  ◉ libsql   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Navigation

- **Tab/Shift+Tab**: Cycle panels
- **j/k or Up/Down**: Navigate lists
- **Enter**: Select/expand
- **h/j/k/l**: Vim-style movement
- **`/`**: Fuzzy search/filter
- **`?`**: Help overlay
- **`:`**: Command palette
- **1-5**: Jump to views (Dashboard, Provision, Monitor, Backup, Logs)

### 5.3 Key Views

1. **Dashboard** — Distro list + details + resource monitor
2. **Provision** — Modal overlay: pack selection, variable prompts, execution progress
3. **Monitor** — Full-screen resource charts, per-distro breakdown
4. **Backup** — Export/import/snapshot manager
5. **Logs** — Scrollable execution history with filtering
6. **Settings** — Config editor, plugin manager, connection preferences

### 5.4 Provisioning Modal

```
┌──────────────── PROVISION: Ubuntu 24.04 ─────────────────────┐
│  Step 1 of 6: Shell Setup                                     │
│                                                                │
│  Which shell do you want as default?                          │
│    ● Zsh + Oh My Zsh                                          │
│    ○ Zsh + Starship                                           │
│    ○ Fish + Fisher                                            │
│    ○ Bash (keep default)                                      │
│    ○ Nushell                                                  │
│                                                                │
│  ┌─ Preview ─────────────────────────────────────────────┐    │
│  │  Installs: zsh, oh-my-zsh, powerlevel10k theme        │    │
│  │  Sets default shell to /usr/bin/zsh                    │    │
│  └───────────────────────────────────────────────────────┘    │
│  [Enter] Select  [←/→] Steps  [Esc] Cancel                   │
└────────────────────────────────────────────────────────────────┘
```

## 6. Connection System

### 6.1 Four Connection Modes

| Mode | Mechanism | Use Case |
|------|-----------|----------|
| **Shell Attach** | Suspend TUI, run `wsl -d <distro>` | Default, simplest, full terminal |
| **Embedded Terminal** | Split pane with PTY (`portable-pty`/`conpty`) | Power users, multi-session |
| **External Launch** | Run `wt.exe -p <distro>` or configured terminal | Users with preferred terminal |
| **Termius** | Launch Termius with SSH connection to distro | Termius users, rich SSH features |

### 6.2 Termius Integration

- Auto-detect Termius installation
- Ensure SSH server running in target distro (auto-provision if `auto_setup_ssh = true`)
- Per-distro SSH port mapping (base_port + offset): Ubuntu=2222, Debian=2223, etc.
- Launch Termius with connection parameters
- If not installed, offer `winget install Termius.Termius`

### 6.3 Configuration

```toml
[connection]
default_mode = "shell_attach"
external_terminal = "wt.exe"
external_args = "-p {distro_name}"

[connection.termius]
auto_setup_ssh = true
base_port = 2222
```

## 7. Error Handling & Resilience

- **WSL failures:** Structured errors with actionable recovery suggestions
- **Storage failures:** Auto-detect corruption, offer rebuild from backup format
- **Plugin failures:** Sandboxed, never crash host. Error in status bar + disable option
- **Provisioning failures:** Per-step retry, resume from last failed step
- **Graceful degradation:** Missing dependencies disable features with helpful messages

## 8. Technology Stack

| Component | Crate | Notes |
|-----------|-------|-------|
| TUI Framework | `ratatui` + `crossterm` | v0.30+, Windows-native |
| Web Server | `axum` + `tower-http` | For wsl-web binary |
| Async Runtime | `tokio` | Shared across both binaries |
| Local Database | `libsql` (core feature) | Embedded, no external deps |
| JSON Storage | `serde_json` + `serde` | Fallback backend |
| WSL API | `wslapi` crate + `wsl.exe` | Dual approach |
| Config | `toml` + `serde` | TOML-based configuration |
| Lua Plugins | `mlua` | Phase 1 runtime plugins |
| WASM Plugins | `wasmtime` | Phase 2 runtime plugins |
| Terminal PTY | `portable-pty` or `conpty` | Embedded terminal mode |
| CLI | `clap` | Command-line argument parsing |

## 9. Future Considerations

- **wsl-web:** Axum binary serving an SPA. Same `wsl-core` library. REST API mirrors plugin commands.
- **Pack marketplace:** Community pack sharing via Git repos or a central registry.
- **WASM plugins:** Phase 2 after Lua plugin API stabilizes.
- **Remote management:** Manage WSL on remote Windows machines via SSH/API.
