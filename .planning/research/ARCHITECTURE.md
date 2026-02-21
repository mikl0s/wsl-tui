# Architecture Research

**Domain:** Rust TUI + Web UI tool for WSL2 management (dual binary, shared core, plugin system)
**Researched:** 2026-02-21
**Confidence:** HIGH (Ratatui patterns, Cargo workspace, Axum SPA) / MEDIUM (plugin system, PTY integration)

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Frontend Layer                              │
│  ┌──────────────────────┐          ┌──────────────────────────────┐ │
│  │       wsl-tui        │          │           wsl-web            │ │
│  │  (Ratatui + crossterm│          │   (Axum + embedded SPA)      │ │
│  │   binary)            │          │   binary)                    │ │
│  └──────────┬───────────┘          └──────────────┬───────────────┘ │
└─────────────┼────────────────────────────────────┼─────────────────┘
              │                                     │
              │  path dependency                    │  path dependency
              │                                     │
┌─────────────▼─────────────────────────────────────▼─────────────────┐
│                        wsl-core (shared library)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
│  │  Plugin Host │  │  WSL Layer   │  │ Storage Layer│               │
│  │  (trait +    │  │  (wsl.exe    │  │  (libsql +   │               │
│  │   registry)  │  │   + wslapi)  │  │   JSON trait)│               │
│  └──────┬───────┘  └──────┬───────┘  └──────────────┘               │
│         │                 │                                           │
│  ┌──────▼───────────────────────────────────────────────────────┐    │
│  │                  Compile-Time Plugin Crates                   │    │
│  │  wsl-plugin-distro  wsl-plugin-provision  wsl-plugin-monitor  │    │
│  │  wsl-plugin-backup  wsl-plugin-connect                        │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │               Runtime Plugin Layer                            │    │
│  │  Lua scripts (mlua, Phase 1)   WASM modules (wasmtime, Phase 2│   │
│  └──────────────────────────────────────────────────────────────┘    │
└───────────────────────────────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      External / System Layer                          │
│   wsl.exe  │  wslapi.dll  │  ~\.wsl-tui\ (config, DB, packs)        │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| `wsl-tui` (binary) | TUI event loop, render cycle, keybinding dispatch, view routing | `main.rs` spawns tokio runtime; Ratatui terminal, crossterm backend |
| `wsl-web` (binary) | HTTP server, REST API routes, embedded SPA assets, WebSocket (optional) | Axum router, `rust-embed` for SPA, tower-http |
| `wsl-core` (lib) | All shared business logic: WSL commands, storage, plugin host, domain types | Workspace lib crate; no UI code, no binary entrypoints |
| `Plugin trait` | Contract between core and all plugins (compile-time and runtime) | `pub trait Plugin: Send + Sync` with init/shutdown/commands/views |
| `wsl-plugin-distro` | Distro lifecycle: list, install, start, stop, terminate, remove, default | Wraps `wsl.exe` commands + `wslapi` crate |
| `wsl-plugin-provision` | Pack loading, dependency resolution, idempotency checks, step execution | TOML parser, step executor, state checker |
| `wsl-plugin-monitor` | CPU/memory/disk metrics per running distro | Polls `/proc/stat`, `/proc/meminfo` via `wsl -d <distro> -- cat ...` |
| `wsl-plugin-backup` | Export/import `.tar`/`.vhdx`, named snapshots | Wraps `wsl --export` / `wsl --import` |
| `wsl-plugin-connect` | Shell attach, embedded PTY, external terminal, Termius/SSH | `portable-pty` for embedded mode, `tokio::process` for attach |
| `StorageBackend trait` | Abstraction over libsql embedded and JSON fallback | `#[async_trait]` with libsql `Connection` and serde_json impl |
| `PluginContext` | Shared context passed to plugins on init: storage handle, WSL layer ref, config | `Arc<PluginContext>` passed at startup |
| `Lua runtime` | Phase 1 runtime plugins: sandboxed mlua instance per script | mlua `Lua::new_with(StdLib::BASE | StdLib::STRING, ...)` |

## Recommended Project Structure

```
wsl-tui/                              # Cargo workspace root
├── Cargo.toml                        # [workspace] manifest + [workspace.dependencies]
├── Cargo.lock                        # Shared lock file (commit this)
├── crates/
│   ├── wsl-core/                     # Shared library — no UI code
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                # Public API surface
│   │       ├── wsl/                  # WSL interaction layer
│   │       │   ├── mod.rs
│   │       │   ├── executor.rs       # tokio::process wsl.exe wrapper
│   │       │   ├── parser.rs         # UTF-16LE output parser
│   │       │   └── native.rs         # wslapi crate bindings
│   │       ├── storage/              # Storage backend
│   │       │   ├── mod.rs            # StorageBackend trait
│   │       │   ├── libsql.rs         # libsql embedded impl
│   │       │   └── json.rs           # JSON fallback impl
│   │       ├── plugin/               # Plugin host
│   │       │   ├── mod.rs            # Plugin trait, registry
│   │       │   ├── context.rs        # PluginContext struct
│   │       │   └── lua.rs            # mlua runtime loader
│   │       ├── config.rs             # Config struct + TOML loading
│   │       ├── types.rs              # Domain types (Distro, Pack, etc.)
│   │       └── error.rs              # Error types (thiserror)
│   │
│   ├── wsl-plugin-distro/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                # Implements Plugin trait
│   │
│   ├── wsl-plugin-provision/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── resolver.rs           # Dependency graph resolution
│   │       ├── executor.rs           # Step execution engine
│   │       └── checker.rs            # Idempotency state checks
│   │
│   ├── wsl-plugin-monitor/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   │
│   ├── wsl-plugin-backup/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   │
│   ├── wsl-plugin-connect/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pty.rs                # portable-pty embedded terminal
│   │       └── external.rs           # External terminal launchers
│   │
│   ├── wsl-tui/                      # TUI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs               # Tokio runtime entry, terminal init
│   │       ├── app.rs                # App state (model), top-level update fn
│   │       ├── tui.rs                # Terminal lifecycle (enter/exit raw mode)
│   │       ├── event.rs              # Async event handler → mpsc channel
│   │       ├── action.rs             # Action enum (typed messages)
│   │       ├── components/           # View components
│   │       │   ├── mod.rs            # Component trait
│   │       │   ├── dashboard.rs      # Dashboard view
│   │       │   ├── provision.rs      # Provision view + wizard
│   │       │   ├── monitor.rs        # Monitor view + gauges
│   │       │   ├── backup.rs         # Backup view
│   │       │   ├── logs.rs           # Logs view
│   │       │   ├── settings.rs       # Settings view
│   │       │   ├── help.rs           # Help overlay
│   │       │   └── palette.rs        # Command palette
│   │       ├── theme.rs              # Catppuccin Mocha colors
│   │       └── config.rs             # TUI-specific config (keybindings)
│   │
│   └── wsl-web/                      # Web binary
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs               # Tokio + Axum startup
│           ├── router.rs             # Route registration
│           ├── handlers/             # HTTP handlers (one per resource)
│           │   ├── distros.rs
│           │   ├── packs.rs
│           │   ├── monitor.rs
│           │   └── backup.rs
│           ├── assets.rs             # rust-embed SPA assets
│           └── error.rs              # HTTP error mapping
│
├── packs/                            # Built-in provisioning packs
│   ├── home-setup.toml
│   ├── claude-code.toml
│   ├── nvm-node.toml
│   └── ...
│
├── frontend/                         # SPA source (build output embedded in wsl-web)
│   ├── package.json
│   └── src/
│
└── docs/
    └── plans/
```

### Structure Rationale

- **`crates/` prefix:** Groups all workspace members under one folder, avoiding root-level clutter. Standard practice for large workspaces.
- **`wsl-core` is library-only:** No `main.rs`, no TUI imports, no HTTP imports. Both binaries depend on it via path. This ensures the core can be tested independently.
- **Plugin crates are separate:** Each plugin crate compiles independently. This enforces the compile-time plugin boundary and allows future removal of individual plugins without touching core.
- **`wsl-tui/src/components/`:** One file per view — matches Ratatui's component architecture template pattern. Clean separation of rendering logic per screen.
- **`[workspace.dependencies]`:** All shared dependency versions declared once in root `Cargo.toml`. Plugin crates reference with `workspace = true` — prevents version drift.

## Architectural Patterns

### Pattern 1: The Elm Architecture (TEA) for TUI

**What:** Model-Update-View cycle. `App` struct holds all state. `Action` enum represents every possible state change. Main loop receives actions, calls `update()`, then calls `view()` to render.

**When to use:** Ratatui recommends TEA as primary pattern. Best for predictable state, easy to test (pure functions), and clear data flow. Ideal for complex multi-view TUIs.

**Trade-offs:** Requires discipline to keep `view()` pure. Mutable state during Ratatui rendering requires pragmatic adaptation (some widgets need `&mut self`).

**Example:**
```rust
// action.rs
#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Render,
    Quit,
    SelectDistro(String),
    InstallDistro(String),
    SwitchView(View),
    ProvisionPack { distro: String, pack_id: String },
}

// app.rs
pub struct App {
    pub running: bool,
    pub current_view: View,
    pub distros: Vec<Distro>,
    pub selected_distro: Option<usize>,
    // ... all app state
}

impl App {
    pub fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::SwitchView(v) => self.current_view = v,
            Action::SelectDistro(name) => { /* update selection */ }
            Action::Quit => self.running = false,
            _ => {}
        }
        Ok(None)
    }

    pub fn view(&mut self, frame: &mut Frame) {
        match self.current_view {
            View::Dashboard => render_dashboard(frame, self),
            View::Provision => render_provision(frame, self),
            // ...
        }
    }
}
```

### Pattern 2: Async Event Loop with tokio mpsc Channels

**What:** Event capture runs in a spawned tokio task. Events (key presses, ticks, resize) are sent over an unbounded channel. Main loop `select!`s between events and render ticks. All long-running operations (WSL commands) are spawned as separate tokio tasks — they send results back as Actions.

**When to use:** Required for non-blocking I/O (WSL command execution, monitoring polls) alongside TUI rendering. Standard pattern in async Ratatui templates.

**Trade-offs:** Adds channel overhead, but prevents UI freeze during WSL commands (which can take 1-5s for installs). Complexity is justified.

**Example:**
```rust
// event.rs — async event producer
pub async fn event_loop(tx: UnboundedSender<Event>) {
    loop {
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                CEvent::Key(key) if key.kind == KeyEventKind::Press => {
                    tx.send(Event::Key(key)).unwrap();
                }
                CEvent::Resize(w, h) => tx.send(Event::Resize(w, h)).unwrap(),
                _ => {}
            }
        }
        tx.send(Event::Tick).unwrap();
    }
}

// main.rs — main loop
loop {
    select! {
        Some(event) = rx.recv() => {
            if let Some(action) = handle_event(event, &app) {
                if let Some(followup) = app.update(action)? {
                    app.update(followup)?;
                }
            }
        }
    }
    terminal.draw(|f| app.view(f))?;
    if !app.running { break; }
}
```

**Critical Windows note:** Filter crossterm events to `KeyEventKind::Press` only. Windows sends both `Press` and `Release` events, causing double-firing of actions. Use `key.kind == KeyEventKind::Press` guard or `as_key_press_event()` helper.

### Pattern 3: Trait-Based Plugin Host

**What:** `Plugin` trait defines the contract. A `PluginRegistry` holds `Vec<Box<dyn Plugin>>`. Compile-time plugins are registered at startup. Runtime Lua plugins are loaded from `~/.wsl-tui/plugins/*.lua` via mlua.

**When to use:** When you need both static (performant, type-safe) and dynamic (user-extensible) plugin support.

**Trade-offs:** `dyn Plugin` requires heap allocation. Lua adds ~1.5MB to binary. Worth it for extensibility.

**Example:**
```rust
// plugin/mod.rs
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn init(&mut self, ctx: Arc<PluginContext>) -> Result<()>;
    fn shutdown(&self) -> Result<()>;
    fn commands(&self) -> Vec<PluginCommand>;
    fn views(&self) -> Vec<ViewDescriptor>;
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn register_builtin<P: Plugin + 'static>(&mut self, p: P) {
        self.plugins.push(Box::new(p));
    }

    pub fn load_lua_plugins(&mut self, dir: &Path, ctx: Arc<PluginContext>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension() == Some("lua".as_ref()) {
                let plugin = LuaPlugin::load(&path, ctx.clone())?;
                self.plugins.push(Box::new(plugin));
            }
        }
        Ok(())
    }
}
```

### Pattern 4: Cargo Workspace with Centralized Dependencies

**What:** Root `Cargo.toml` declares `[workspace]` members and `[workspace.dependencies]` with all shared crate versions. Member crates reference with `{ workspace = true }`.

**When to use:** Any multi-crate project. Prevents version conflicts, centralizes upgrades.

**Trade-offs:** Slightly more setup. Massive win for maintainability.

**Example:**
```toml
# Root Cargo.toml
[workspace]
members = [
    "crates/wsl-core",
    "crates/wsl-plugin-distro",
    "crates/wsl-plugin-provision",
    "crates/wsl-plugin-monitor",
    "crates/wsl-plugin-backup",
    "crates/wsl-plugin-connect",
    "crates/wsl-tui",
    "crates/wsl-web",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
authors = ["Mikkel Georgsen"]

[workspace.dependencies]
# TUI
ratatui = "0.30"
crossterm = "0.28"
# Web
axum = { version = "0.8", features = ["macros"] }
tower-http = { version = "0.6", features = ["fs", "cors", "trace"] }
tokio = { version = "1", features = ["full"] }
# Data
libsql = { version = "0.6", features = ["core"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
# Error handling
thiserror = "2"
anyhow = "1"
# Plugins
mlua = { version = "0.10", features = ["lua54", "vendored"] }
# WSL
wslapi = "0.2"
# PTY
portable-pty = "0.8"
# Embed assets
rust-embed = "8"
```

### Pattern 5: Axum with Embedded SPA

**What:** `wsl-web` binary embeds the compiled SPA bundle at compile time using `rust-embed`. Axum serves the REST API under `/api/` and falls back to `index.html` for all other routes (enabling client-side routing).

**When to use:** Single deployable binary requirement. Eliminates CORS configuration.

**Trade-offs:** Binary size increases by SPA size (typically 1-5MB compressed). No hot reload in production. Use `cargo run --features dev` with static file serving for development.

**Example:**
```rust
// assets.rs
#[derive(Embed)]
#[folder = "../../frontend/dist/"]
struct Assets;

// router.rs
pub fn build_router(core: Arc<Core>) -> Router {
    Router::new()
        .nest("/api", api_routes(core))
        .fallback(spa_handler)  // SPA catch-all
}

async fn spa_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            // SPA client-side routing: serve index.html
            let index = Assets::get("index.html").unwrap();
            Html(String::from_utf8(index.data.into_owned()).unwrap()).into_response()
        }
    }
}
```

## Data Flow

### TUI Event Flow

```
[Terminal Input]
    │
    ▼ (crossterm EventStream)
[Event Task]  ─── tokio::spawn ───►  [event::poll() loop]
    │                                      │
    │ UnboundedSender<Event>               │ KeyEventKind::Press filter
    ▼                                      ▼
[Main Loop] ◄──── UnboundedReceiver<Event>
    │
    │ handle_event() → Action
    ▼
[App::update(action)]  ─────────────► state mutation
    │
    │ optional spawn for async work (WSL commands)
    │
    ▼ tokio::spawn ────────────────► [WslExecutor::run()]
                                           │
                                           │ result → Action::CommandResult(...)
                                           ▼
                                    [tx.send(Action)] ──► [App::update()]
    │
    ▼ (every frame)
[terminal.draw(|f| app.view(f))]
```

### Web API Flow

```
[Browser Request]
    │
    ▼
[Axum Router]
    │ /api/distros → handlers::distros::list()
    ▼
[Handler] → Arc<Core> → plugin.list_distros()
    │                         │
    │                         ▼
    │                   [WslExecutor] → wsl.exe
    │                         │
    │                         ▼
    │                   [StorageBackend] → upsert_distro()
    │
    ▼ JSON response
[Browser]
```

### Plugin Data Flow

```
[App startup]
    │
    ▼
[PluginRegistry::register_builtin(DistroPlugin)]
[PluginRegistry::register_builtin(ProvisionPlugin)]
[PluginRegistry::load_lua_plugins("~/.wsl-tui/plugins/")]
    │
    ▼
[Plugin::init(Arc<PluginContext>)] ── each plugin gets shared context
    │
    ├── PluginContext contains:
    │       - Arc<dyn StorageBackend>
    │       - Arc<WslExecutor>
    │       - AppConfig
    │
    ▼
[Plugin::commands()] ── merged into command palette
[Plugin::views()]   ── registered as navigable views
```

### WSL Command Execution Flow

```
[App Action: InstallDistro("Ubuntu-24.04")]
    │
    ▼
[tokio::spawn async block]
    │
    ▼
[WslExecutor::run("wsl", ["--install", "Ubuntu-24.04"])]
    │
    ▼
[tokio::process::Command::new("wsl.exe")]
    │ .args(["--install", "Ubuntu-24.04"])
    │ .output().await
    │
    ▼
[UTF-16LE decode: String::from_utf16(&bytes_as_u16)]
    │
    ▼
[Result<WslOutput>] ──► tx.send(Action::DistroInstalled(...))
```

### Storage Initialization Flow

```
[App::new()]
    │
    ▼
[config.toml] → storage_backend field?
    │
    ├── "json" → JsonBackend::new("~/.wsl-tui/data/")
    │
    └── "libsql" (default) → libsql::Database::open("~/.wsl-tui/data.db")
            │
            ├── success → LibsqlBackend
            │
            └── failure → warn + fallback to JsonBackend
```

## Scaling Considerations

This is a local desktop tool, not a server application. Scaling means feature complexity, not concurrent users.

| Scale | Architecture Adjustments |
|-------|--------------------------|
| Single user, local | Current architecture — monolith with plugins is correct |
| Many plugins / views | Plugin registry + view routing already handles this — no changes needed |
| Remote WSL management (future) | Abstract `WslExecutor` trait; add `RemoteWslExecutor` via SSH; currently out of scope |
| WASM plugins (Phase 2) | Add `WasmPlugin` impl of `Plugin` trait; wasmtime adds ~10MB; host-side capability grants |

### Scaling Priorities

1. **First complexity driver:** Plugin count and Lua script loading time. Mitigation: lazy-load Lua runtime only if `~/.wsl-tui/plugins/` is non-empty.
2. **Second complexity driver:** Monitoring poll frequency. Mitigation: configurable poll interval; use `tokio::time::interval` not busy-loop; debounce renders.

## Anti-Patterns

### Anti-Pattern 1: Blocking in the Async Event Loop

**What people do:** Call `std::process::Command::new("wsl.exe").output()` (blocking) directly inside the main async loop.
**Why it's wrong:** Blocks the tokio executor thread. Freezes the TUI for the entire duration of the WSL command (can be 5-30s for installs).
**Do this instead:** Use `tokio::process::Command` with `.await`. Spawn long operations with `tokio::spawn`. Send results back as Actions over the mpsc channel.

### Anti-Pattern 2: Ignoring KeyEventKind on Windows

**What people do:** Handle all `Event::Key(e)` events without checking `e.kind`.
**Why it's wrong:** On Windows, crossterm fires both `KeyEventKind::Press` and `KeyEventKind::Release`. Every key action fires twice, causing double navigation, double commands.
**Do this instead:** Guard every key handler with `if key.kind != KeyEventKind::Press { return; }` or use `as_key_press_event()`.

### Anti-Pattern 3: Circular Crate Dependencies

**What people do:** Have `wsl-plugin-distro` depend on `wsl-tui` for shared types, while `wsl-tui` depends on `wsl-plugin-distro`.
**Why it's wrong:** Cargo circular dependency — compile error, impossible to build.
**Do this instead:** All shared types live in `wsl-core`. Both the plugin crate and the TUI crate depend on `wsl-core`. They never depend on each other.

### Anti-Pattern 4: UI Code in wsl-core

**What people do:** Import `ratatui` in `wsl-core` to share widget helpers.
**Why it's wrong:** Forces `wsl-web` to compile ratatui even though it never renders TUI. Bloats the web binary. Violates separation of concerns.
**Do this instead:** Keep `wsl-core` free of all UI crates. TUI-specific widgets go in `wsl-tui`. Web-specific middleware goes in `wsl-web`.

### Anti-Pattern 5: Synchronous Storage in the Render Loop

**What people do:** Query storage (libsql) synchronously inside `app.view()` or `app.update()`.
**Why it's wrong:** Storage I/O blocks the render loop, causes frame drops and stuttery UI.
**Do this instead:** Pre-fetch data into App state on app start and after mutations. Use `tokio::spawn` for refreshes. Never call `StorageBackend` methods inside render path.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| `wsl.exe` | `tokio::process::Command` shell-out | UTF-16LE output requires `String::from_utf16` decode; always async |
| `wslapi.dll` | `wslapi` crate (Windows bindings) | Platform-specific; only for registration checks and config queries |
| `~/.wsl-tui/data.db` | `libsql` crate, embedded mode | `libsql::Database::open_local(path)` — no external process |
| Lua scripts | `mlua` crate, standalone mode | Sandboxed: restrict `StdLib` to safe subset; load from `plugins/` dir |
| ConPTY / PTY | `portable-pty` crate | Use `native_pty_system()` — wraps ConPTY on Windows automatically |
| External terminals | `std::process::Command` | Launch `wt.exe`, `alacritty`, `wezterm` with `wsl -d <distro>` args |
| SPA frontend | `rust-embed` at compile time | `#[folder = "../../frontend/dist/"]` bakes in compiled SPA |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `wsl-tui` ↔ `wsl-core` | Rust function calls (direct) | Linked via `path = "../wsl-core"` dependency |
| `wsl-web` ↔ `wsl-core` | Rust function calls (direct) | Same; both binaries compile wsl-core into their binary |
| `wsl-core` ↔ plugins (compile-time) | Rust function calls via `Plugin` trait | `Box<dyn Plugin>` in registry |
| `wsl-core` ↔ Lua plugins (runtime) | `mlua` API: Rust exposes functions to Lua tables | `lua.globals().set("wsl", wsl_table)?` |
| TUI ↔ async tasks | `tokio::sync::mpsc::unbounded_channel` | Events in, Actions back out |
| Web handlers ↔ core | `Arc<Core>` passed via Axum state (`State<Arc<Core>>`) | Standard Axum dependency injection |

## Build Order for Phases

Dependencies between components dictate which must be built first:

```
Phase 1 — Foundation
  wsl-core skeleton (types, error, config)
    └── StorageBackend trait + libsql impl
    └── WslExecutor (basic wsl.exe shell-out)
    └── Plugin trait + PluginRegistry

Phase 2 — Core TUI (depends on Phase 1)
  wsl-plugin-distro (implements Plugin, uses WslExecutor)
  wsl-tui binary (event loop, basic views)
    └── Dashboard view (list distros)

Phase 3 — Provisioning (depends on Phase 2)
  wsl-plugin-provision (pack TOML loading, executor, idempotency)
  wsl-tui Provision view

Phase 4 — Monitoring + Backup (depends on Phase 2)
  wsl-plugin-monitor
  wsl-plugin-backup
  wsl-tui Monitor + Backup views

Phase 5 — Connectivity (depends on Phase 2)
  wsl-plugin-connect (PTY, external terminal, Termius)

Phase 6 — Runtime Plugins (depends on Phase 1)
  wsl-core/plugin/lua.rs (mlua integration)
  LuaPlugin impl of Plugin trait

Phase 7 — Web UI (depends on Phase 1, any phase)
  wsl-web binary (Axum router, handlers)
  SPA frontend (build → embed)
```

## Sources

- [Ratatui Component Architecture](https://ratatui.rs/concepts/application-patterns/component-architecture/) — HIGH confidence (official docs)
- [Ratatui Elm Architecture](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/) — HIGH confidence (official docs)
- [Ratatui Flux Architecture](https://ratatui.rs/concepts/application-patterns/flux-architecture/) — HIGH confidence (official docs)
- [Ratatui async template structure](https://ratatui.github.io/async-template/02-structure.html) — HIGH confidence (official template)
- [Ratatui templates repository](https://github.com/ratatui/templates) — HIGH confidence (official)
- [Cargo Workspace Best Practices](https://reintech.io/blog/cargo-workspace-best-practices-large-rust-projects) — MEDIUM confidence (verified with official Cargo docs)
- [Official Cargo Workspaces docs](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) — HIGH confidence (official)
- [Rust plugin system techniques](https://nullderef.com/blog/plugin-tech/) — MEDIUM confidence (technical blog, multiple sources agree)
- [mlua repository](https://github.com/mlua-rs/mlua) — HIGH confidence (official)
- [wslapi crate](https://docs.rs/wslapi/latest/wslapi/) — HIGH confidence (docs.rs)
- [wsl-rust crate](https://github.com/mmastrac/wsl-rust) — MEDIUM confidence (GitHub, active)
- [portable-pty crate](https://docs.rs/portable-pty) — HIGH confidence (docs.rs)
- [rust-embed for SPA serving](https://nguyenhuythanh.com/posts/rust-backend-spa/) — MEDIUM confidence (verified with rust-embed docs)
- [crossterm KeyEventKind Windows issue](https://github.com/ratatui/ratatui/issues/347) — HIGH confidence (official ratatui issue tracker)
- [tokio::process::Command](https://docs.rs/tokio/latest/tokio/process/struct.Command.html) — HIGH confidence (official docs)
- [Actors with Tokio](https://ryhl.io/blog/actors-with-tokio/) — MEDIUM confidence (widely cited, author is tokio contributor)

---
*Architecture research for: Rust TUI + Web UI WSL2 management tool (wsl-tui)*
*Researched: 2026-02-21*
