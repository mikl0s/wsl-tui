# Phase 1: Foundation - Research

**Researched:** 2026-02-21
**Domain:** Rust workspace scaffold, embedded storage (libsql/JSON), WSL command execution with encoding detection, TUI event loop (ratatui/crossterm), Windows platform quirks
**Confidence:** HIGH (core stack verified via official docs and confirmed GitHub issues; encoding behavior verified via multiple Microsoft WSL issues)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- First run shows a **welcome screen** with key info (config location, how to customize) before proceeding to the main TUI
- Config directory is `~/.wsl-tui/` — auto-created on first run
- Default `config.toml` is **fully commented** — all available options present but commented out with descriptions, like a typical dotfile
- **Environment variable overrides** supported: `WSL_TUI_*` env vars take precedence over config.toml values (useful for CI/scripting)
- **Unit tests required** for every public function, integration tests for cross-crate boundaries
- **Status bar indicator** shows current storage backend — visible in the TUI status bar
- When `storage = "auto"` and libsql fails, fall back transparently with the status bar reflecting the active backend
- When libsql becomes available after running on JSON fallback, **offer migration** — prompt user to migrate data, keep JSON as backup until confirmed

### Claude's Discretion

- Directory structure within `~/.wsl-tui/` — create subdirectories as needed by each phase
- Error handling pattern (thiserror/anyhow strategy)
- Rust conventions beyond clippy (unwrap policy, visibility defaults, doc comment requirements)
- Async runtime policy (tokio vs minimal async based on dependency needs)
- Whether status bar shows storage backend always or only on fallback
- Explicit storage = "libsql" failure behavior (refuse to start vs fall back with warning)
- JSON data file location relative to libsql

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| FOUND-01 | Cargo workspace compiles with all crate scaffolding and zero warnings | Resolver v2, workspace.dependencies, rust-version, per-crate Cargo.toml patterns documented |
| FOUND-02 | libsql embedded storage works on Windows with stack overflow workaround | Confirmed: `/STACK:8000000` linker flag in `.cargo/config.toml` for MSVC target; verified via GitHub issue #1051 |
| FOUND-03 | JSON fallback storage activates transparently when libsql fails | StorageBackend trait pattern; `auto` mode tries libsql first, falls back to serde_json file |
| FOUND-04 | Storage backend is configurable via `config.toml` (`auto` \| `libsql` \| `json`) | toml + serde pattern documented; `dirs` crate for config path resolution |
| FOUND-05 | WSL command execution handles both UTF-16LE and UTF-8 output encoding | `WSL_UTF8` env var check at runtime; `encoding_rs` for UTF-16LE decode; BOM-less detection strategy documented |
| FOUND-06 | Plugin trait and registry system supports compile-time plugin registration | `Box<dyn Plugin>` trait objects in a `Vec` inside `PluginRegistry`; no macro magic needed for Phase 1 |
| FOUND-07 | Configuration loaded from `~/.wsl-tui/config.toml` with sensible defaults | `dirs::home_dir()` + path join; `toml::from_str` with `#[derive(Deserialize, Default)]` |
| FOUND-08 | TUI event loop filters `KeyEventKind::Press` only (Windows crossterm fix) | Confirmed Windows crossterm issue; `key.kind == KeyEventKind::Press` guard documented; `is_key_press()` helper available |
| FOUND-09 | Panic hook restores terminal on crash via `ratatui::init()`/`ratatui::restore()` | `ratatui::init()` auto-installs panic hook; `ratatui::restore()` in custom hook via `take_hook()` pattern |
| FOUND-10 | Workspace uses `resolver = "2"` to prevent feature unification issues | Resolver 2 documented; required in `[workspace]` table; prevents build/dev/target-specific feature bleed |
| DX-01 | CLAUDE.md at repo root with coding standards, architecture patterns, and Rust conventions | CLAUDE.md structure researched; key sections identified; Rust-specific conventions established |
| DX-02 | Per-crate CLAUDE.md files for wsl-core, wsl-tui, and wsl-web with crate-specific context | Same format as root; scoped to crate's responsibilities and cross-crate API contracts |
| DX-03 | `cargo clippy --workspace` passes with zero warnings | Enforced via `deny(warnings)` or `#![warn(clippy::all)]` at crate level; zero-warning policy from day one |
| DX-04 | `cargo test --workspace` passes all tests | Unit tests in each module; integration tests in `tests/` directories at crate root |
| DX-05 | Startup time under 500ms to first render | Lazy initialization; avoid blocking I/O on startup critical path; defer storage opens |
| DX-06 | Idle memory usage under 50MB | No baseline concerns for Phase 1 skeleton; monitor with `sysinfo` in later phases |
| DX-07 | Binary size under 30MB (without WASM runtime) | Release profile: `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `strip = true`; libsql adds ~10-15MB |

</phase_requirements>

---

## Summary

Phase 1 builds a Rust workspace scaffold for a Windows-native TUI binary. The three hardest technical problems are all Windows platform quirks that must be solved at day one: (1) libsql's SQL parser triggers a stack overflow on Windows because the default 1MB stack is insufficient for deep recursion — the fix is a single linker flag in `.cargo/config.toml`; (2) `wsl.exe` outputs UTF-16LE by default but switches to UTF-8 when `WSL_UTF8=1` is set, requiring runtime detection rather than hardcoding; (3) crossterm on Windows emits both KeyPress and KeyRelease events, so every event handler must guard on `key.kind == KeyEventKind::Press`.

The library stack is straightforward and well-established: ratatui 0.30 with crossterm backend for the TUI, libsql 0.9.x for embedded SQLite storage, serde + toml for config loading, thiserror 2.0 for typed errors in wsl-core, anyhow 1.0 for application-level error propagation in wsl-tui and wsl-web, and tokio as the async runtime (required by libsql). The workspace uses Cargo resolver v2 and workspace dependency inheritance to prevent version drift between the three crates.

The plugin system for Phase 1 is intentionally minimal: a `Plugin` trait with a `name()` method and a `PluginRegistry` struct holding `Vec<Box<dyn Plugin>>`. This is the hook point that Phase 6 will grow into a Lua runtime; for now it only needs to exist and be exercisable in tests.

**Primary recommendation:** Scaffold the workspace first with all three crate stubs compiling, the linker flag in place, and clippy at zero warnings before implementing any logic — getting a clean build baseline is the gate everything else depends on.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ratatui | 0.30.0 | TUI rendering framework | De facto standard; 17M+ downloads; modular in 0.30 with stable ratatui-core |
| crossterm | 0.29.x | Cross-platform terminal backend | Bundled with ratatui; handles raw mode, alternate screen, event reading |
| libsql | 0.9.29 | Embedded SQLite-compatible DB | Turso's fork; async API; works with tokio; required feature: `core` (default) |
| tokio | 1.x | Async runtime | Required by libsql; also used for async event stream with crossterm |
| serde | 1.0 | Serialization framework | Universal; `features = ["derive"]` |
| serde_json | 1.0 | JSON file backend | JSON fallback storage implementation |
| toml | 0.8+ | TOML config deserialization | Parse `config.toml`; pairs with serde |
| thiserror | 2.0.18 | Typed error derive macro | Library errors in wsl-core (structured, matchable) |
| anyhow | 1.0 | Application error wrapping | Application errors in wsl-tui and wsl-web (ergonomic propagation) |
| dirs | 6.x | Platform home/config directories | `dirs::home_dir()` for `~/.wsl-tui/` path construction on Windows |
| encoding_rs | 0.8.x | Character encoding conversion | UTF-16LE decode of `wsl.exe` output |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio-macros | (via tokio) | `#[tokio::main]`, `#[tokio::test]` | Entry points and async test functions |
| crossterm event-stream feature | (via crossterm) | Async `EventStream` for tokio select! | When using tokio-based async event loop |
| futures | 0.3 | `StreamExt` for EventStream | Required when using crossterm EventStream |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| libsql | rusqlite + sqlx | rusqlite is sync-only; sqlx is more complex setup; libsql has async API and future remote-sync capability |
| libsql | sled (embedded KV) | sled lacks SQL; loses query expressiveness needed for Phase 3+ |
| thiserror | manual Error impls | thiserror 2.0 eliminates boilerplate with no runtime cost |
| dirs | std::env::home_dir() | `home_dir()` is deprecated in std and unreliable on Windows; `dirs` uses Known Folder API |
| encoding_rs | manual UTF-16 detection | encoding_rs is maintained by Mozilla, handles BOM behavior correctly |

**Installation (workspace root Cargo.toml):**
```toml
[workspace]
members = ["wsl-core", "wsl-tui", "wsl-web"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.88"
authors = ["Mikkel Georgsen"]
license = "MIT"

[workspace.dependencies]
ratatui = "0.30"
crossterm = { version = "0.29", features = ["event-stream"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
thiserror = "2"
anyhow = "1"
dirs = "6"
encoding_rs = "0.8"
libsql = { version = "0.9", default-features = false, features = ["core"] }
futures = "0.3"
```

---

## Architecture Patterns

### Recommended Project Structure

```
wsl-tui/                         # workspace root
├── .cargo/
│   └── config.toml              # Windows stack size linker flag
├── Cargo.toml                   # workspace manifest, resolver = "2"
├── CLAUDE.md                    # root coding standards
├── wsl-core/                    # shared library crate
│   ├── Cargo.toml
│   ├── CLAUDE.md
│   └── src/
│       ├── lib.rs
│       ├── config.rs            # Config struct, TOML loading, env overrides
│       ├── error.rs             # CoreError (thiserror)
│       ├── storage/
│       │   ├── mod.rs           # StorageBackend trait
│       │   ├── libsql.rs        # LibsqlBackend impl
│       │   └── json.rs          # JsonBackend impl (serde_json)
│       ├── wsl/
│       │   └── executor.rs      # WslExecutor, encoding detection
│       └── plugin/
│           ├── mod.rs           # Plugin trait
│           └── registry.rs      # PluginRegistry
├── wsl-tui/                     # TUI binary crate
│   ├── Cargo.toml
│   ├── CLAUDE.md
│   └── src/
│       ├── main.rs              # #[tokio::main], panic hook, ratatui::init/restore
│       ├── app.rs               # App state struct
│       └── ui/
│           └── welcome.rs       # Welcome screen (first run)
└── wsl-web/                     # Web binary crate (stub only in Phase 1)
    ├── Cargo.toml
    ├── CLAUDE.md
    └── src/
        └── main.rs              # Empty stub, compiles cleanly
```

### Pattern 1: Windows Stack Size Linker Flag

**What:** Sets the Windows PE executable stack size to 8MB instead of the default 1MB, required for libsql's SQL parser to avoid stack overflow during recursive descent.

**When to use:** Always — set once in `.cargo/config.toml`, applies to all workspace binaries.

**Example:**
```toml
# .cargo/config.toml
# Source: https://github.com/tursodatabase/libsql/issues/1051 (confirmed fix)

[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "link-arg=/STACK:8000000"]

[target.x86_64-pc-windows-gnu]
rustflags = ["-C", "link-arg=-Wl,--stack,8000000"]
```

### Pattern 2: StorageBackend Trait

**What:** A `dyn`-safe async-compatible trait that both LibsqlBackend and JsonBackend implement, with factory function that tries libsql first.

**When to use:** All storage access goes through this trait so calling code never knows which backend is active.

**Example:**
```rust
// Source: project pattern; based on trait object best practices

#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    async fn execute(&self, sql: &str, params: Vec<String>) -> Result<(), CoreError>;
    async fn query(&self, sql: &str, params: Vec<String>) -> Result<Vec<Row>, CoreError>;
    fn backend_name(&self) -> &'static str;
}

pub enum BackendKind {
    Auto,
    Libsql,
    Json,
}

/// Tries libsql first when Auto; returns (backend, active_name)
pub async fn open_storage(
    config_dir: &Path,
    kind: BackendKind,
) -> Result<(Box<dyn StorageBackend>, &'static str), CoreError> {
    match kind {
        BackendKind::Libsql => {
            let b = LibsqlBackend::open(config_dir).await?;
            Ok((Box::new(b), "libsql"))
        }
        BackendKind::Json => {
            let b = JsonBackend::open(config_dir)?;
            Ok((Box::new(b), "json"))
        }
        BackendKind::Auto => {
            match LibsqlBackend::open(config_dir).await {
                Ok(b) => Ok((Box::new(b), "libsql")),
                Err(_) => {
                    let b = JsonBackend::open(config_dir)?;
                    Ok((Box::new(b), "json"))  // TUI reads backend_name() for status bar
                }
            }
        }
    }
}
```

### Pattern 3: WSL Encoding Detection

**What:** Check `WSL_UTF8` environment variable at runtime to pick the correct decoder for `wsl.exe` stdout.

**When to use:** Every call to `wsl.exe` that reads stdout.

**Example:**
```rust
// Source: VSCode WSL extension issue #276253; WSL GitHub issue #4607

use encoding_rs::UTF_16LE;
use std::process::Command;

pub fn run_wsl(args: &[&str]) -> Result<String, CoreError> {
    let output = Command::new("wsl.exe")
        .args(args)
        .output()
        .map_err(CoreError::WslExec)?;

    let stdout = if std::env::var("WSL_UTF8").as_deref() == Ok("1") {
        // WSL_UTF8=1: output is UTF-8
        String::from_utf8_lossy(&output.stdout).into_owned()
    } else {
        // Default: wsl.exe outputs UTF-16LE without BOM
        let (decoded, _, _) = UTF_16LE.decode(&output.stdout);
        decoded.into_owned()
    };

    Ok(stdout.trim().to_string())
}
```

### Pattern 4: TUI Event Loop with KeyEventKind Filter

**What:** crossterm on Windows fires both Press and Release events for each keystroke. Filter to Press only.

**When to use:** All keyboard event handlers in the TUI.

**Example:**
```rust
// Source: ratatui async-template; crossterm docs; ratatui FAQ
// https://ratatui.rs/tutorials/counter-async-app/async-event-stream/

use crossterm::event::{EventStream, KeyEventKind};
use futures::StreamExt;
use tokio::select;

async fn event_loop(tx: mpsc::UnboundedSender<AppEvent>) {
    let mut reader = EventStream::new();
    loop {
        select! {
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(crossterm::event::Event::Key(key))) => {
                        // CRITICAL: Windows fires KeyRelease too — only handle Press
                        if key.kind == KeyEventKind::Press {
                            tx.send(AppEvent::Key(key)).ok();
                        }
                    }
                    None | Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }
}
```

### Pattern 5: Panic Hook + ratatui::init/restore

**What:** `ratatui::init()` auto-installs a panic hook that calls `ratatui::restore()` before panicking. For the TUI skeleton, this is sufficient.

**When to use:** Entry point of wsl-tui/src/main.rs.

**Example:**
```rust
// Source: https://docs.rs/ratatui/latest/ratatui/fn.init.html
// Source: https://ratatui.rs/recipes/apps/panic-hooks/

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ratatui::init() installs panic hook, enables raw mode, enters alternate screen
    let mut terminal = ratatui::init();

    let result = run_app(&mut terminal).await;

    // Always restore, even if run_app returned Err
    ratatui::restore();

    result
}

async fn run_app(terminal: &mut ratatui::DefaultTerminal) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| {
            // placeholder render
            frame.render_widget("WSL TUI - Loading...", frame.area());
        })?;

        // Event handling with KeyEventKind::Press filter
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind == KeyEventKind::Press
                && key.code == crossterm::event::KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}
```

### Pattern 6: Config Loading with Env Override

**What:** Load TOML config from `~/.wsl-tui/config.toml`, fall back to `Default::default()` if missing, then apply `WSL_TUI_*` environment variable overrides.

**Example:**
```rust
// Source: dirs crate docs; toml crate docs; serde docs

use dirs::home_dir;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default = "default_storage")]
    pub storage: StorageMode,
    // ... other fields
}

fn default_storage() -> StorageMode { StorageMode::Auto }

impl Config {
    pub fn load() -> Result<Self, CoreError> {
        let config_dir = home_dir()
            .ok_or(CoreError::NoHomeDir)?
            .join(".wsl-tui");

        std::fs::create_dir_all(&config_dir).ok();

        let config_path = config_dir.join("config.toml");
        let mut config = if config_path.exists() {
            let text = std::fs::read_to_string(&config_path)?;
            toml::from_str(&text)?
        } else {
            Config::default()
        };

        // WSL_TUI_* env overrides (locked decision)
        if let Ok(val) = std::env::var("WSL_TUI_STORAGE") {
            config.storage = val.parse()?;
        }

        Ok(config)
    }
}
```

### Pattern 7: Plugin Trait (Compile-Time Registration)

**What:** Minimal `Plugin` trait for Phase 1. The registry holds boxed trait objects inserted at startup in `main.rs`. Runtime plugin loading (Lua) is Phase 6.

**Example:**
```rust
// Phase 1 minimal design — expands in Phase 6

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.iter().find(|p| p.name() == name).map(|p| p.as_ref())
    }

    pub fn all(&self) -> &[Box<dyn Plugin>] {
        &self.plugins
    }
}
```

### Anti-Patterns to Avoid

- **Calling `ratatui::restore()` only on success:** Always call it — use a `defer`-style pattern or wrap in a function called before `?` propagation. `ratatui::init()` handles this in the simple case, but custom code must not skip restore.
- **Hardcoding UTF-16LE for wsl.exe output:** Must check `WSL_UTF8` at runtime — VSCode got burned by this exact assumption.
- **Using `unwrap()` on libsql operations in production code:** libsql operations return `Result`; always propagate.
- **Using `crossterm::event::read()` inside async context without EventStream:** `read()` blocks the thread; use `EventStream` with `tokio::select!` in async contexts.
- **Setting `resolver = "1"` or omitting it:** The default before Rust 2021 edition is resolver v1, which unifies features across the workspace and can cause platform-specific features to bleed across crates (e.g., Windows-only tokio features enabling in wsl-core tests on Linux).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Terminal state restore on panic | Custom signal handler or atexit hook | `ratatui::init()` (auto-installs panic hook) | Handles alternate screen, raw mode, cursor state atomically |
| UTF-16LE decode | Manual byte-pair iteration | `encoding_rs::UTF_16LE.decode()` | Handles BOM variants, replacement characters, edge cases |
| Home directory resolution on Windows | `std::env::var("USERPROFILE")` | `dirs::home_dir()` | Uses Windows Known Folder API; handles edge cases like OneDrive redirection |
| TOML deserialization | Manual string parsing | `toml::from_str` with `#[derive(Deserialize)]` | Handles nested tables, arrays, type coercion |
| Typed errors | `Box<dyn Error>` everywhere | `thiserror` in libraries, `anyhow` in binaries | thiserror generates matchable variants; anyhow adds context chaining |
| Async key event polling | `thread::sleep` polling loop | `crossterm::EventStream` with `futures::StreamExt` | Non-blocking, composable with `tokio::select!`, no busy-wait |

**Key insight:** The Windows platform quirks (stack size, encoding, key events) all have known, verified fixes. The trap is discovering them mid-implementation when tests start failing; implementing them from day one prevents downstream rework.

---

## Common Pitfalls

### Pitfall 1: libsql Stack Overflow on Windows

**What goes wrong:** `cargo test` or the binary panics with a stack overflow during the first SQL operation involving the parser (e.g., `CREATE TABLE`, `SELECT`).

**Why it happens:** libsql's SQL parser (lemon-rs) uses deep recursion. Windows default thread stack is 1MB; the parser needs ~8MB for some query shapes.

**How to avoid:** Add to `.cargo/config.toml` before writing any libsql code:
```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "link-arg=/STACK:8000000"]
```

**Warning signs:** Stack overflow panic in any libsql test. The stack trace will show `libsql::parser` frames.

**Source:** [GitHub issue #1051](https://github.com/tursodatabase/libsql/issues/1051) — confirmed fix

---

### Pitfall 2: Double Key Events on Windows (crossterm)

**What goes wrong:** Every keypress triggers the action twice — `q` quits twice, or menu items navigate two steps.

**Why it happens:** Windows terminal infrastructure (Console API) reports both key-down and key-up events. crossterm surfaces both as `Event::Key`. Linux/macOS only emit key-down.

**How to avoid:** Guard every key handler:
```rust
if key.kind == KeyEventKind::Press { /* handle */ }
// OR use the convenience method:
if let Some(key) = event.as_key_press_event() { /* handle */ }
```

**Warning signs:** Actions firing twice on Windows but once on Linux in tests.

**Source:** [ratatui issue #347](https://github.com/ratatui/ratatui/issues/347); [crossterm docs](https://docs.rs/crossterm/latest/crossterm/event/struct.KeyEvent.html)

---

### Pitfall 3: wsl.exe Encoding Hardcoding

**What goes wrong:** Parsing fails on machines with `WSL_UTF8=1` set (common in developer setups, CI environments with explicit UTF-8 locale configs).

**Why it happens:** `wsl.exe` switches from UTF-16LE to UTF-8 when the env var is set. Code that always decodes as UTF-16LE gets garbage characters.

**How to avoid:** Check `std::env::var("WSL_UTF8")` before each `wsl.exe` invocation and pick the decoder accordingly.

**Warning signs:** Distro names appear as garbled characters or empty strings on some machines but not others.

**Source:** [VSCode issue #276253](https://github.com/microsoft/vscode/issues/276253); [WSL issue #4607](https://github.com/microsoft/WSL/issues/4607)

---

### Pitfall 4: Terminal Not Restored After Panic

**What goes wrong:** After a panic during development, the terminal is stuck in raw mode — no echo, cursor hidden, and the alternate screen may still be active.

**Why it happens:** `ratatui::init()` enables raw mode and enters the alternate screen; without a panic hook these are not reversed on panic.

**How to avoid:** Always use `ratatui::init()` (which auto-installs the panic hook). Do NOT call `Terminal::new(CrosstermBackend::new(stdout()))` directly without also installing the hook separately.

**Warning signs:** Terminal appears broken after a crash; `reset` command needed to recover.

**Source:** [ratatui panic hooks recipe](https://ratatui.rs/recipes/apps/panic-hooks/)

---

### Pitfall 5: Resolver v1 Feature Bleed

**What goes wrong:** A Windows-specific feature (e.g., `tokio/io-driver` on Windows) gets enabled in a dev dependency on Linux because workspace feature unification merges all feature sets.

**Why it happens:** Resolver v1 (default before edition 2021) merges features globally across the workspace regardless of target, build vs. dev context.

**How to avoid:** Set `resolver = "2"` in `[workspace]`. With resolver v2, build deps, dev deps, and platform-specific deps are isolated.

**Warning signs:** CI failing on Linux with Windows-specific code paths compiled in.

**Source:** [Rust Edition Guide - Default Cargo resolver](https://doc.rust-lang.org/edition-guide/rust-2021/default-cargo-resolver.html)

---

### Pitfall 6: async_trait overhead (if used)

**What goes wrong:** Using `async_trait` macro for `StorageBackend` adds a heap allocation per async call via `Box::pin(async move {...})`.

**Why it happens:** Rust's async trait support (without the macro) requires nightly or RPITIT syntax that stabilized in Rust 1.75.

**How to avoid:** Since MSRV is Rust 1.88, prefer native async fn in traits (stabilized in 1.75 with RPIT, full support in 1.75+). Use `async_trait` only if object-safety is required and RPITIT syntax is awkward. For `dyn StorageBackend` you still need manual boxing or `async_trait`.

**Recommendation:** Use `async_trait = "0.1"` for Phase 1 since `dyn StorageBackend` requires it; revisit if performance profiling shows hot-path overhead.

---

## Code Examples

Verified patterns from official sources:

### libsql Local Database Initialization

```rust
// Source: https://docs.turso.tech/sdk/rust/reference
use libsql::Builder;

pub async fn open_local_db(path: &Path) -> Result<libsql::Connection, libsql::Error> {
    let db = Builder::new_local(path).build().await?;
    let conn = db.connect()?;
    Ok(conn)
}

// Smoke test pattern (verifies no stack overflow):
#[tokio::test]
async fn test_libsql_smoke() {
    let db = Builder::new_local(":memory:").build().await.unwrap();
    let conn = db.connect().unwrap();
    conn.execute("CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, val TEXT)", ())
        .await
        .unwrap();
    conn.execute("INSERT INTO test (val) VALUES (?1)", ["hello"])
        .await
        .unwrap();
    let mut rows = conn.query("SELECT val FROM test", ()).await.unwrap();
    let row = rows.next().await.unwrap().unwrap();
    assert_eq!(row.get::<String>(0).unwrap(), "hello");
}
```

### ratatui::init + panic hook + event loop skeleton

```rust
// Source: https://docs.rs/ratatui/latest/ratatui/fn.init.html

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();  // installs panic hook, raw mode, alt screen
    let result = run_app(&mut terminal).await;
    ratatui::restore();                 // always restore regardless of result
    result
}

async fn run_app(terminal: &mut ratatui::DefaultTerminal) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(
                ratatui::widgets::Paragraph::new("WSL TUI - Press q to quit"),
                area,
            );
        })?;

        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind == crossterm::event::KeyEventKind::Press {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
    }
}
```

### WSL Executor with encoding detection

```rust
// Source: derived from VSCode issue #276253 and WSL issue #4607
use encoding_rs::UTF_16LE;
use std::process::Command;

pub fn wsl_command_output(args: &[&str]) -> Result<String, CoreError> {
    let output = Command::new("wsl.exe")
        .args(args)
        .output()
        .map_err(|e| CoreError::WslExec(e.to_string()))?;

    if !output.status.success() {
        return Err(CoreError::WslFailed(
            String::from_utf8_lossy(&output.stderr).into_owned()
        ));
    }

    let text = if std::env::var_os("WSL_UTF8").is_some() {
        String::from_utf8_lossy(&output.stdout).into_owned()
    } else {
        let (decoded, _, _) = UTF_16LE.decode(&output.stdout);
        decoded.into_owned()
    };

    Ok(text.trim_end_matches('\0').trim().to_string())
}
```

### Workspace Cargo.toml structure

```toml
# Source: https://doc.rust-lang.org/cargo/reference/workspaces.html

[workspace]
members = ["wsl-core", "wsl-tui", "wsl-web"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.88"
authors = ["Mikkel Georgsen"]
license = "MIT"
repository = "https://github.com/mikl0s/wsl-tui"

[workspace.dependencies]
# Each member declares: libsql = { workspace = true }
libsql = "0.9"
tokio = { version = "1", features = ["full"] }
ratatui = "0.30"
crossterm = { version = "0.29", features = ["event-stream"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
thiserror = "2"
anyhow = "1"
dirs = "6"
encoding_rs = "0.8"
futures = "0.3"

[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
strip = true
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `tui` crate (fork) | `ratatui` 0.30 | 2023 (tui archived) | ratatui is the maintained fork; all docs and community on ratatui |
| `Terminal::new(CrosstermBackend::new(stdout()))` | `ratatui::init()` | ratatui 0.28.1 | init() auto-installs panic hook; old pattern needs manual hook setup |
| Resolver v1 (implicit) | Resolver v2 (`resolver = "2"`) | Rust 2021 edition | v2 prevents cross-target feature bleed; mandatory for Windows-targeting workspaces |
| `std::env::home_dir()` | `dirs::home_dir()` | Deprecated in Rust 1.29 | std version is unreliable on Windows; `dirs` uses Windows Known Folder API |
| `async_trait` for all async traits | Native async fn in traits (RPITIT) | Rust 1.75 | Native support eliminates macro overhead; still need `async_trait` for `dyn` object safety |
| thiserror 1.x | thiserror 2.0.x | 2024 | v2 has better proc-macro hygiene; same API surface |

**Deprecated/outdated:**
- `tui` crate: archived; use `ratatui`
- `std::env::home_dir()`: deprecated; use `dirs::home_dir()`
- Resolver v1: avoid for new workspace projects on Windows

---

## Open Questions

1. **async_trait vs. native async fn in trait for StorageBackend**
   - What we know: `async fn in trait` stabilized in Rust 1.75; MSRV is 1.88, so it's available. But `dyn StorageBackend` with async methods still requires boxing. `async_trait` macro handles this transparently.
   - What's unclear: Whether the `async_trait` allocation overhead matters for storage calls that are inherently I/O-bound anyway.
   - Recommendation: Use `async_trait = "0.1"` for Phase 1 simplicity; revisit only if profiling shows storage call overhead.

2. **libsql connection pooling strategy**
   - What we know: `db.connect()` creates a connection; libsql is async.
   - What's unclear: Whether a single `Arc<Connection>` is sufficient for Phase 1's single-threaded TUI usage, or whether a connection pool is needed.
   - Recommendation: Single `Arc<Connection>` wrapped in the storage backend is sufficient for Phase 1; the Web UI phase will need a pool (documented in Phase 7 plans).

3. **JSON fallback data file location**
   - What we know: This is marked as Claude's discretion in CONTEXT.md.
   - Recommendation: Use `~/.wsl-tui/data.json` for the JSON backend alongside `~/.wsl-tui/wsl-tui.db` for libsql. Simple, predictable, no subdirectories needed in Phase 1.

4. **wsl.exe output contains null bytes**
   - What we know: UTF-16LE decode of wsl.exe output sometimes includes trailing null bytes (`\0`) that must be stripped.
   - What's unclear: Whether this is consistent across all WSL versions or platform-specific.
   - Recommendation: Always call `.trim_end_matches('\0').trim()` after decoding wsl.exe output.

---

## Sources

### Primary (HIGH confidence)

- [ratatui::init() official docs](https://docs.rs/ratatui/latest/ratatui/fn.init.html) — init behavior, panic hook, ratatui 0.30 API
- [ratatui v0.30 highlights](https://ratatui.rs/highlights/v030/) — modularization, `ratatui::run()`, new APIs
- [ratatui panic hooks recipe](https://ratatui.rs/recipes/apps/panic-hooks/) — `take_hook()` + `restore_tui()` pattern
- [ratatui async event stream tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) — `EventStream` + `tokio::select!` + `KeyEventKind::Press` filter
- [libsql Turso Rust reference](https://docs.turso.tech/sdk/rust/reference) — Builder API, local database, connection, execute, query patterns; libsql 0.9.29
- [libsql docs.rs](https://docs.rs/libsql/latest/libsql/) — confirmed version 0.9.29
- [thiserror docs.rs](https://docs.rs/thiserror/latest/thiserror/) — version 2.0.18, derive macro usage
- [Cargo resolver documentation](https://doc.rust-lang.org/edition-guide/rust-2021/default-cargo-resolver.html) — resolver v2 behavior
- [crossterm KeyEvent docs](https://docs.rs/crossterm/latest/crossterm/event/struct.KeyEvent.html) — `kind` field, `KeyEventKind::Press`
- [encoding_rs UTF_16LE docs](https://docs.rs/encoding_rs/latest/encoding_rs/static.UTF_16LE.html) — BOM handling, decode methods
- [Cargo workspaces book](https://doc.rust-lang.org/cargo/reference/workspaces.html) — workspace.dependencies, resolver, rust-version

### Secondary (MEDIUM confidence)

- [libsql Windows stack overflow issue #1051](https://github.com/tursodatabase/libsql/issues/1051) — `/STACK:8000000` fix confirmed working by reporter
- [VSCode WSL encoding issue #276253](https://github.com/microsoft/vscode/issues/276253) — WSL_UTF8 behavior, UTF-16LE default confirmed
- [WSL issue #4607](https://github.com/microsoft/WSL/issues/4607) — wsl.exe UTF-16 output without BOM confirmed
- [ratatui issue #347](https://github.com/ratatui/ratatui/issues/347) — Windows duplicate key events confirmed
- [dirs crate docs](https://crates.io/crates/dirs) — Windows Known Folder API usage
- [Rust 1.88 release announcement](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/) — confirmed release date June 26, 2025; let chains, naked functions

### Tertiary (LOW confidence)

- Various blog posts on thiserror/anyhow patterns — consistent with official docs, used for pattern confirmation only

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified via official docs/crates.io
- Architecture: HIGH — patterns derived from official ratatui and libsql docs
- Windows quirks (stack, encoding, key events): HIGH — confirmed via official GitHub issues with verified fixes
- Pitfalls: HIGH for documented issues; MEDIUM for async_trait overhead concern
- CLAUDE.md structure: MEDIUM — based on community patterns and gist analysis

**Research date:** 2026-02-21
**Valid until:** 2026-04-21 (60 days; ratatui and libsql both actively developed but core APIs are stable in 0.30/0.9 series)
