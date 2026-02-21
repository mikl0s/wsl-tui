# wsl-core — Agent Reference

`wsl-core` is the shared library crate for the WSL TUI workspace. All business logic, storage, WSL command execution, configuration, error types, and the plugin system live here. The two binaries (`wsl-tui` and `wsl-web`) depend on this crate and never depend on each other.

---

## Crate Purpose

- Owns all shared data types and error types
- Provides the configuration system (`Config`, `StorageMode`)
- Implements both storage backends (`LibsqlBackend`, `JsonBackend`) behind a trait abstraction
- Executes `wsl.exe` subcommands with runtime encoding detection
- Defines the compile-time plugin interface and registry

---

## Module Structure

```
wsl-core/src/
├── lib.rs           — crate root; re-exports the full public API
├── error.rs         — CoreError enum (all error categories as variants)
├── config.rs        — Config struct, StorageMode enum, DEFAULT_CONFIG_TOML
├── storage/
│   ├── mod.rs       — StorageBackend trait, StorageValue, StorageRow, BackendKind,
│   │                  StorageResult, open_storage factory
│   ├── libsql.rs    — LibsqlBackend (primary embedded SQLite backend)
│   └── json.rs      — JsonBackend (JSON fallback with SQL mini-parser)
├── wsl/
│   ├── mod.rs       — re-exports WslExecutor
│   └── executor.rs  — WslExecutor: decode_output, run, list_verbose
└── plugin/
    ├── mod.rs       — Plugin trait (Send + Sync)
    └── registry.rs  — PluginRegistry: register, get, all, count
```

---

## Public API

Everything re-exported from `lib.rs`:

```rust
pub use config::{Config, StorageMode};
pub use error::CoreError;
pub use plugin::{Plugin, PluginRegistry};
pub use wsl::WslExecutor;
```

The `storage` module is `pub mod` but consumers typically use `open_storage` and the trait.

### `config` module

| Item | Description |
|---|---|
| `Config` | Application configuration loaded from `~/.wsl-tui/config.toml` |
| `Config::load()` | Load from the canonical location (`~/.wsl-tui/config.toml`); creates dir + default file on first run |
| `Config::load_from(PathBuf)` | Load from an explicit directory — used in tests to avoid touching `~/.wsl-tui/` |
| `Config.storage` | Which storage backend to use (`StorageMode`) |
| `Config.config_dir` | Resolved path to `~/.wsl-tui/` (guaranteed to exist after `load`) |
| `Config.first_run` | `true` when `config.toml` did not exist before this run |
| `StorageMode` | `Auto` \| `Libsql` \| `Json`; impl `FromStr` + `Display` |
| `DEFAULT_CONFIG_TOML` | `&'static str` with fully-commented default config — written to disk on first run |

### `error` module

| Variant | When used |
|---|---|
| `CoreError::NoHomeDir` | `dirs::home_dir()` returned `None` |
| `CoreError::ConfigParse(String)` | Config value rejected (e.g., invalid `StorageMode` string) |
| `CoreError::ConfigRead(io::Error)` | I/O error reading the config file |
| `CoreError::TomlParse(toml::de::Error)` | Config file is not valid TOML |
| `CoreError::StorageError(String)` | Storage backend init or operation failed |
| `CoreError::WslExec(String)` | `wsl.exe` could not be spawned |
| `CoreError::WslFailed(String)` | `wsl.exe` exited with a non-zero status |
| `CoreError::PluginError(String)` | A plugin reported an error |

**Rule:** All public functions in `wsl-core` return `Result<T, CoreError>`. Never define a separate per-module error type.

### `storage` module

| Item | Description |
|---|---|
| `StorageValue` | Single cell value: `Null`, `Integer(i64)`, `Real(f64)`, `Text(String)`, `Blob(Vec<u8>)` |
| `StorageRow` | `Vec<StorageValue>` — one row from a query |
| `StorageBackend` | Async trait: `execute`, `query`, `backend_name` |
| `BackendKind` | `Auto` \| `Libsql` \| `Json`; `From<StorageMode>` impl |
| `StorageResult` | Output of `open_storage`: `backend`, `backend_name`, `migration_available` |
| `open_storage(dir, kind)` | Factory: opens the requested backend; `Auto` falls back transparently |
| `LibsqlBackend` | libsql embedded SQLite; db file at `<config_dir>/wsl-tui.db` |
| `LibsqlBackend::open(dir)` | Async: open or create the file-based DB |
| `LibsqlBackend::open_memory()` | Test-only: open an in-memory DB (no filesystem) |
| `JsonBackend` | JSON fallback; persists to `<config_dir>/data.json` |
| `JsonBackend::open(dir)` | Sync: open or create the JSON file |

#### StorageBackend trait

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn execute(&self, sql: &str, params: Vec<StorageValue>) -> Result<u64, CoreError>;
    async fn query(&self, sql: &str, params: Vec<StorageValue>) -> Result<Vec<StorageRow>, CoreError>;
    fn backend_name(&self) -> &'static str;   // "libsql" or "json"
}
```

All call sites use `Box<dyn StorageBackend>` — never couple to a specific backend type.

### `wsl` module

| Item | Description |
|---|---|
| `WslExecutor` | Stateless struct; `Default` + `Clone` |
| `WslExecutor::new()` | Construct a new executor |
| `WslExecutor::decode_output(raw: &[u8])` | Pub fn: decode UTF-16LE or UTF-8 (reads `WSL_UTF8` env), strip null bytes, trim whitespace |
| `WslExecutor::run(&[&str])` | Run `wsl.exe` with the given args; returns decoded stdout |
| `WslExecutor::list_verbose()` | Run `wsl.exe --list --verbose`; returns decoded output |

**CI pattern:** `decode_output` is `pub` so tests can verify encoding logic without spawning `wsl.exe` (WSL may not be available in CI).

### `plugin` module

| Item | Description |
|---|---|
| `Plugin` | Trait: `name() -> &str`, `version() -> &str`, `Send + Sync` |
| `PluginRegistry` | Holds `Vec<Box<dyn Plugin>>`; registration order preserved |
| `PluginRegistry::new()` | Create empty registry |
| `PluginRegistry::register(plugin)` | Append a plugin (duplicates not checked) |
| `PluginRegistry::get(name)` | Find by name; returns first match; `None` if absent |
| `PluginRegistry::all()` | `&[Box<dyn Plugin>]` — all plugins in insertion order |
| `PluginRegistry::count()` | Number of registered plugins |

---

## Cross-Crate Contracts

**What `wsl-tui` uses from `wsl-core`:**
- `Config::load()` at startup
- `Config.first_run` to control welcome screen
- `StorageBackend` (via `open_storage`) for distro state persistence
- `StorageResult.backend_name` for the TUI status bar
- `StorageResult.migration_available` for the Phase 2 migration prompt
- `WslExecutor` for listing and managing WSL distros
- `Plugin` + `PluginRegistry` for compile-time plugins

**What `wsl-web` will use from `wsl-core` (Phase 7):**
- `Config::load()` for configuration
- `open_storage` for the web backend's data store
- `WslExecutor` for REST API distro operations

---

## Error Handling Conventions

```rust
// In wsl-core: return CoreError
pub fn my_function() -> Result<String, CoreError> {
    let home = dirs::home_dir().ok_or(CoreError::NoHomeDir)?;
    let content = std::fs::read_to_string(home.join("file"))?;  // ConfigRead via From<io::Error>
    Ok(content)
}

// In wsl-tui / wsl-web: use anyhow
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;  // CoreError auto-wraps via anyhow
    Ok(())
}
```

---

## Adding a New Module to wsl-core

1. Create `wsl-core/src/<name>.rs` (or `<name>/mod.rs` for a submodule).
2. Add `pub mod <name>;` in `wsl-core/src/lib.rs`.
3. Re-export the public API with `pub use <name>::{TypeA, TypeB};` in `lib.rs`.
4. Implement the module — all errors must be `CoreError` variants (add new variants to `error.rs` if needed).
5. Add `#[cfg(test)] mod tests { ... }` with unit tests for every public function.
6. Run `cargo test -p wsl-core` — all tests must pass.
7. Run `cargo clippy -p wsl-core -- -D warnings` — zero warnings.

---

## Adding a New Storage Backend

1. Create `wsl-core/src/storage/<name>.rs`.
2. Implement `StorageBackend` for the new backend struct using `#[async_trait]`.
3. Add a new variant to `BackendKind` in `storage/mod.rs`.
4. Update `open_storage` to handle the new variant.
5. Add `pub mod <name>;` and `pub use <name>::<BackendType>;` in `storage/mod.rs`.
6. Add tests: at minimum a smoke test (create table, insert, query, assert).

---

## Storage Notes

- **Auto mode:** `open_storage(dir, BackendKind::Auto)` tries libsql first; falls back to JSON silently on any error. The caller never knows which backend is active — this is intentional per the locked design decision.
- **migration_available:** Set to `true` when libsql is active AND `data.json` exists in `config_dir`. This flag enables the Phase 2 migration prompt UI. No data is moved in Phase 1 — detection only.
- **JSON mini-parser:** `JsonBackend` supports `CREATE TABLE`, `INSERT`, `SELECT`, and `DELETE` via string prefix matching. It is not a general SQL parser — only the fixed set of statements the application issues.
- **LibsqlBackend lifetime:** The `_db: libsql::Database` field keeps the connection alive. Do not remove it — it is not dead code.

---

## Test Count (Phase 1)

| Module | Tests |
|---|---|
| `error` | 8 |
| `config` | 15 |
| `storage/libsql` | 4 |
| `storage/json` | 5 |
| `storage/mod` | 5 |
| `wsl/executor` | 14 |
| `plugin/registry` | 9 |
| **Total** | **60** |

Run all with: `cargo test -p wsl-core`
