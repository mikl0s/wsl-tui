# WSL TUI — Agent Reference

This document describes the actual architecture, coding conventions, and platform requirements for the `wsl-tui` workspace. Read this before making changes. Per-crate details are in the per-crate CLAUDE.md files linked at the bottom.

---

## Project Overview

**WSL TUI** is a Rust-based TUI + Web UI for managing WSL2 on Windows 11.

- **Workspace:** Cargo monorepo with three crates
  - `wsl-core` — shared library (business logic, config, storage, WSL execution, plugin system)
  - `wsl-tui` — interactive TUI binary (ratatui + crossterm)
  - `wsl-web` — web binary (Axum REST API + embedded SPA, Phase 7 — currently a stub)
- **License:** MIT
- **Author:** Mikkel Georgsen
- **Repository:** <https://github.com/mikl0s/wsl-tui>
- **Rust MSRV:** 1.88 (set by sysinfo 0.37.2; Rust 2024 edition required by ratatui 0.30)

---

## Architecture

### Workspace Layout

```
wsl-tui/                          (repo root)
├── Cargo.toml                    (workspace manifest; all dep versions pinned here)
├── .cargo/config.toml            (Windows stack size linker flags + GNU linker path)
├── wsl-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                (re-exports: Config, StorageMode, CoreError, WslExecutor, Plugin, PluginRegistry)
│       ├── config.rs             (Config, StorageMode, DEFAULT_CONFIG_TOML)
│       ├── error.rs              (CoreError enum with 8 variants)
│       ├── storage/
│       │   ├── mod.rs            (StorageBackend trait, StorageValue, StorageRow, BackendKind, StorageResult, open_storage)
│       │   ├── libsql.rs         (LibsqlBackend — primary embedded SQLite backend)
│       │   └── json.rs           (JsonBackend — JSON fallback with SQL mini-parser)
│       ├── wsl/
│       │   ├── mod.rs            (re-exports WslExecutor)
│       │   └── executor.rs       (WslExecutor: decode_output, run, list_verbose)
│       └── plugin/
│           ├── mod.rs            (Plugin trait)
│           └── registry.rs       (PluginRegistry: register, get, all, count)
├── wsl-tui/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               (entry point: Config::load, ratatui::init, run_app, ratatui::restore)
│       ├── app.rs                (App struct: running, first_run, show_welcome)
│       └── ui/
│           ├── mod.rs            (render dispatcher: welcome vs placeholder)
│           └── welcome.rs        (first-run welcome screen, Catppuccin colors)
└── wsl-web/
    ├── Cargo.toml
    └── src/
        └── main.rs               (stub: prints "not yet implemented")
```

### Dependency Flow

```
wsl-tui  ──depends on──>  wsl-core
wsl-web  ──depends on──>  wsl-core
```

`wsl-tui` and `wsl-web` never depend on each other. All shared logic lives in `wsl-core`.

---

## Coding Standards

### Error Handling

- **`wsl-core`:** Use `thiserror` — every public API returns `Result<T, CoreError>`. All error categories are variants of `CoreError`; do not define separate error types per module.
- **`wsl-tui` and `wsl-web`:** Use `anyhow::Result` — wrap `CoreError` via `?` operator for ergonomic propagation.
- **Never use `.unwrap()`** in production code. Use `.expect("descriptive reason")` only at initialization where failure is truly unrecoverable (e.g., the OS cannot allocate a thread). Use `?` everywhere else.

### Visibility

- Default to `pub(crate)` for items used only within a crate.
- Use `pub` only for items that are part of the cross-crate public API.
- Internal helpers within a module should be `fn` (private) unless tests in other files need them.

### Doc Comments

- Required on all `pub` types, traits, and functions (`///` style).
- Include an example in the doc comment for non-trivial functions.
- Module-level `//!` doc comments required on every file.

### Import Grouping

Separate import groups with a blank line:
1. `std` imports
2. External crate imports
3. Internal workspace imports (`crate::` or `wsl_core::`)

### Tests

- Unit tests: in the same file as the code, in a `#[cfg(test)] mod tests { ... }` block.
- Integration tests: in a `tests/` directory at the crate root, for cross-crate boundary verification.
- `cargo test --workspace` must pass with zero failures.
- `cargo clippy --workspace -- -D warnings` must pass with zero warnings.
- **No `.unwrap()` in tests** unless the test is specifically exercising the happy path where failure would indicate a test bug (and even then, add a descriptive `.expect()`).

---

## Testing Requirements

| Requirement | Rule |
|---|---|
| Public functions | Unit test required for every public function |
| Cross-crate boundaries | Integration test required |
| `cargo test --workspace` | Must pass, zero failures |
| `cargo clippy --workspace -- -D warnings` | Must pass, zero warnings |
| Async tests | Use `#[tokio::test]` (not `#[test]` on async fns) |

---

## Windows Platform Requirements

### Stack Size (Critical — Do Not Remove)

`.cargo/config.toml` sets an 8MB thread stack for both GNU and MSVC toolchains:

```toml
[target.x86_64-pc-windows-gnu]
linker = "C:\\msys64\\mingw64\\bin\\gcc.exe"
rustflags = ["-C", "link-arg=-Wl,--stack,8000000"]

[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "link-arg=/STACK:8000000"]
```

**Why:** libsql's SQL parser (lemon-rs) uses deep recursion that overflows the default 1MB Windows thread stack. Removing this flag causes a stack overflow on first SQL query.

### Toolchain

The project uses `stable-x86_64-pc-windows-gnu` with MinGW-w64 from MSYS2. Ensure `C:\msys64\mingw64\bin` is in PATH. MSVC target config is preserved for environments where MSVC tools are in PATH.

### KeyEventKind Filter (Critical — Do Not Remove)

```rust
if key.kind != KeyEventKind::Press {
    continue;
}
```

On Windows, crossterm fires **two events per key press** (Press + Release). Without this filter, every key is processed twice. This filter must be applied in ALL key event handlers.

### WSL Output Encoding

`wsl.exe` outputs UTF-16LE by default. When `WSL_UTF8=1` is set, it outputs UTF-8. Use `WslExecutor::decode_output()` to handle both cases. Always strip trailing null bytes from `wsl.exe` output.

---

## Performance Targets

| Target | Value |
|---|---|
| Startup time to first render | < 500ms |
| Idle memory usage | < 50MB |
| Release binary size | < 30MB |

Release profile is configured in `Cargo.toml` root with `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `strip = true`.

---

## Async Runtime

- **Runtime:** tokio (required by libsql)
- **Entry points:** Use `#[tokio::main]` on `main()` in `wsl-tui` and `wsl-web`
- **Async tests:** Use `#[tokio::test]`
- **Trait objects:** Use `async_trait` for `dyn` trait objects requiring async methods (e.g., `StorageBackend`)

---

## Configuration System

- **Config file location:** `~/.wsl-tui/config.toml`
- **Auto-created:** On first run, `Config::load()` creates the directory and writes a fully-commented default config
- **`first_run` flag:** Set to `true` when `config.toml` did not exist before this run; used by TUI to show the welcome screen
- **Env var overrides:** `WSL_TUI_*` prefix; override file values (e.g., `WSL_TUI_STORAGE=json`)
- **Config dir:** `~/.wsl-tui/` resolved via `dirs::home_dir()`
- **Test helper:** `Config::load_from(PathBuf)` for tests — avoids touching `~/.wsl-tui/`

### ENV_LOCK Pattern (Required for Tests)

Any test that reads or writes `WSL_TUI_*` environment variables must acquire a static mutex and clear the variable:

```rust
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_guard() -> std::sync::MutexGuard<'static, ()> {
    let guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("WSL_TUI_STORAGE");
    guard
}
```

The same pattern applies to `WSL_UTF8` using a `WSL_UTF8_LOCK`.

---

## Workspace Dependencies

All crate versions are pinned once in `[workspace.dependencies]` in the root `Cargo.toml`. Member crates use `{ workspace = true }` to reference them. Never add a direct version to a member crate's `Cargo.toml` for a dependency already in the workspace.

---

## Per-Crate Documentation

- [wsl-core/CLAUDE.md](wsl-core/CLAUDE.md) — module structure, public API, storage backend guide, plugin guide
- [wsl-tui/CLAUDE.md](wsl-tui/CLAUDE.md) — entry point, event loop, UI rendering, adding views
- [wsl-web/CLAUDE.md](wsl-web/CLAUDE.md) — stub status, Phase 7 roadmap context

---

## Phase Roadmap (Summary)

| Phase | Focus | Status |
|---|---|---|
| 1 — Foundation | Workspace, config, storage, WSL executor, TUI skeleton | Complete |
| 2 — Core TUI | Distro list, shell attach, status bar, keybindings | Planned |
| 3 — Provisioning | Pack system, Ansible-lite engine, dry-run | Planned |
| 4 — Distro Management | Create, delete, import/export, clone | Planned |
| 5 — Connectivity | External terminal, Termius SSH integration | Planned |
| 6 — Extensibility | Lua runtime plugins, WASM Phase 2 | Planned |
| 7 — Web UI | Axum REST API, embedded SPA | Planned |
