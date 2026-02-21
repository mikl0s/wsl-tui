---
phase: 01-foundation
plan: 01
subsystem: infra
tags: [rust, cargo-workspace, libsql, ratatui, crossterm, tokio, thiserror, config, mingw-w64, msys2]

requires: []
provides:
  - "Compilable 3-crate Cargo workspace (wsl-core, wsl-tui, wsl-web)"
  - "Windows stack size linker flag for libsql stack overflow prevention"
  - "CoreError enum with 8 variants for all Phase 1 error categories"
  - "Config struct with TOML loading, env var overrides, and first-run detection"
  - "StorageMode enum (Auto/Libsql/Json) with FromStr and Display"
  - "DEFAULT_CONFIG_TOML: fully commented template written on first run"
  - "23 unit tests; all pass; zero clippy warnings"
affects:
  - "02-storage: consumes Config.storage and Config.config_dir"
  - "03-wsl-exec: consumes CoreError.WslExec / WslFailed"
  - "04-tui: consumes Config.first_run for welcome screen"

tech-stack:
  added:
    - "ratatui 0.30 (TUI framework)"
    - "crossterm 0.29 with event-stream (terminal backend)"
    - "tokio 1 full (async runtime)"
    - "libsql 0.9 core (embedded SQLite)"
    - "thiserror 2 (typed errors in wsl-core)"
    - "anyhow 1 (error propagation in binaries)"
    - "serde 1 + derive (serialization)"
    - "serde_json 1 (JSON fallback storage)"
    - "toml 0.8 (config file parsing)"
    - "dirs 6 (home dir via Windows Known Folder API)"
    - "encoding_rs 0.8 (UTF-16LE for wsl.exe output)"
    - "async-trait 0.1 (dyn StorageBackend trait object support)"
    - "futures 0.3 (EventStream support)"
    - "tempfile 3 (dev-dep for test isolation)"
    - "Rust toolchain: stable-x86_64-pc-windows-gnu (MinGW-w64 via MSYS2)"
  patterns:
    - "workspace.dependencies: all crate versions pinned at workspace root"
    - "resolver = 2: prevents Windows-specific feature bleed across crates"
    - "GNU toolchain with MinGW-w64: avoids MSVC PATH dependency"
    - "--stack linker flag: -Wl,--stack,8000000 for libsql SQL parser recursion"
    - "ENV_LOCK mutex pattern: serialize all tests that read WSL_TUI_* env vars"
    - "Config::load_from(PathBuf): testable variant that avoids ~/.wsl-tui/"

key-files:
  created:
    - "Cargo.toml (workspace manifest)"
    - ".cargo/config.toml (linker flags, MinGW-w64 linker path)"
    - "wsl-core/Cargo.toml"
    - "wsl-core/src/lib.rs (re-exports Config, StorageMode, CoreError)"
    - "wsl-core/src/error.rs (CoreError with 8 variants)"
    - "wsl-core/src/config.rs (Config, StorageMode, DEFAULT_CONFIG_TOML)"
    - "wsl-tui/Cargo.toml"
    - "wsl-tui/src/main.rs (stub)"
    - "wsl-web/Cargo.toml"
    - "wsl-web/src/main.rs (stub)"
  modified: []

key-decisions:
  - "GNU toolchain (x86_64-pc-windows-gnu) instead of MSVC: MSVC C++ workload installed but not added to PATH; MinGW-w64 GCC resolves the link.exe ambiguity without requiring Developer Command Prompt"
  - "Config::load_from(PathBuf) test helper: avoids touching ~/.wsl-tui/ in tests while keeping Config::load() clean for production use"
  - "ENV_LOCK serialization for ALL Config::load_from tests: Rust runs tests in the same process sharing env vars; any test calling load_from must hold the lock to prevent WSL_TUI_* leakage"
  - "tempfile 3 added as dev-dependency: provides clean temp dirs for config dir creation tests"
  - "StorageMode uses serde rename_all = lowercase: TOML values 'auto'/'libsql'/'json' match enum variant names"

patterns-established:
  - "Pattern 1: Windows stack size — use -Wl,--stack,8000000 in .cargo/config.toml GNU target; /STACK:8000000 for MSVC"
  - "Pattern 2: Config loading — load() for production, load_from(dir) for tests; both apply env overrides after file parse"
  - "Pattern 3: Test isolation for env vars — acquire ENV_LOCK and clear WSL_TUI_* at test start; guarantees clean state even after panics"
  - "Pattern 4: Workspace deps — all library versions declared once in [workspace.dependencies], members use workspace = true"

requirements-completed: [FOUND-01, FOUND-04, FOUND-07, FOUND-10, DX-03, DX-07]

duration: 24min
completed: 2026-02-21
---

# Phase 1 Plan 01: Workspace Scaffold and Config System Summary

**3-crate Cargo workspace with MinGW-w64 GNU toolchain, 8MB stack flag for libsql, CoreError enum, and TOML config with WSL_TUI_STORAGE env override — zero warnings, 23 tests green**

## Performance

- **Duration:** 24 min
- **Started:** 2026-02-21T20:42:31Z
- **Completed:** 2026-02-21T21:06:00Z
- **Tasks:** 2 of 2
- **Files modified:** 10

## Accomplishments

- Three-crate workspace (wsl-core/wsl-tui/wsl-web) compiles with zero errors using MinGW-w64 GNU toolchain via MSYS2
- Windows stack size linker flag (`-Wl,--stack,8000000`) configured in `.cargo/config.toml` for libsql SQL parser stack overflow prevention
- CoreError enum with 8 typed variants covers all Phase 1 error categories (config, storage, WSL exec, plugin)
- Config system loads `~/.wsl-tui/config.toml`, creates dir on first run, writes fully-commented default template, applies `WSL_TUI_*` env var overrides
- All 23 unit tests pass; `cargo clippy --workspace -- -D warnings` exits 0

## Task Commits

Each task was committed atomically:

1. **Task 1: Create Cargo workspace with all crate stubs and Windows linker flag** - `13b2a8c` (feat)
2. **Task 2: Create error types and config system with env overrides** - `e118605` (feat)

**Plan metadata:** (docs commit — recorded below)

## Files Created/Modified

- `Cargo.toml` — Workspace manifest: resolver=2, workspace.package, all deps, release profile
- `.cargo/config.toml` — GNU/MSVC linker flags, MinGW-w64 linker path
- `wsl-core/Cargo.toml` — Library crate manifest with all shared deps
- `wsl-core/src/lib.rs` — Crate root: declares modules, re-exports Config/StorageMode/CoreError
- `wsl-core/src/error.rs` — CoreError enum: NoHomeDir, ConfigParse, ConfigRead, TomlParse, StorageError, WslExec, WslFailed, PluginError
- `wsl-core/src/config.rs` — Config struct, StorageMode, DEFAULT_CONFIG_TOML const, Config::load/load_from, 15 unit tests
- `wsl-tui/Cargo.toml` — TUI binary manifest
- `wsl-tui/src/main.rs` — Stub: `#[tokio::main] async fn main() -> anyhow::Result<()>`
- `wsl-web/Cargo.toml` — Web binary manifest
- `wsl-web/src/main.rs` — Stub: prints "not yet implemented"

## Decisions Made

- **GNU toolchain over MSVC:** MSVC Build Tools 2022 is installed but only has the MSBuild workload — the C++ compiler (`cl.exe`) and MSVC `link.exe` are not in PATH. Git Bash's GNU `link` (coreutils) shadowed any MSVC `link.exe` lookup. Switched to `stable-x86_64-pc-windows-gnu` toolchain with MinGW-w64 GCC from MSYS2. MSVC target config preserved for environments where MSVC tools are in PATH.

- **Config::load_from(PathBuf) test helper:** Keeps `Config::load()` production API clean while giving tests full control over the config directory path without touching `~/.wsl-tui/`.

- **ENV_LOCK for all Config::load_from tests:** Discovered that `test_env_override_invalid_value` left `WSL_TUI_STORAGE=redis` in the process environment when it ran before other tests, causing those tests to fail with "invalid storage mode 'redis'". Fixed by adding `env_guard()` helper that acquires `ENV_LOCK` AND clears `WSL_TUI_STORAGE` at the start of every test that calls `load_from`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Switched from MSVC to GNU toolchain; installed MinGW-w64 via MSYS2**
- **Found during:** Task 1 (workspace scaffold)
- **Issue:** MSVC C++ workload not in PATH; Git Bash's GNU `link` (coreutils) shadows MSVC `link.exe`; both `rust-lld` and GNU `link` failed with different errors (lld: missing kernel32.lib; GNU link: wrong invocation format)
- **Fix:** Installed `stable-x86_64-pc-windows-gnu` Rust toolchain; installed MinGW-w64 GCC via MSYS2 pacman; configured `.cargo/config.toml` to use MinGW-w64 GCC as linker; attempted MSVC installation via winget and VS installer (both partially successful but C++ compiler still not in PATH)
- **Files modified:** `.cargo/config.toml`
- **Verification:** `cargo build --workspace` exits 0; all three crates compile
- **Committed in:** `13b2a8c` (part of Task 1 commit)

**2. [Rule 1 - Bug] Fixed test env-var leakage causing false failures**
- **Found during:** Task 2 (config tests)
- **Issue:** `test_env_override_invalid_value` left `WSL_TUI_STORAGE=redis` in process env when run before other `load_from` tests; caused `test_default_storage_mode_is_auto_from_commented_toml` and `test_storage_mode_from_toml_explicit` to fail
- **Fix:** Added `env_guard()` helper that acquires `ENV_LOCK` and clears `WSL_TUI_STORAGE`; applied to all tests that call `Config::load_from`
- **Files modified:** `wsl-core/src/config.rs`
- **Verification:** All 23 tests pass consistently across multiple runs
- **Committed in:** `e118605` (part of Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 blocking environment issue, 1 bug)
**Impact on plan:** Both auto-fixes necessary for correctness. No scope creep. MSVC target config preserved for future use.

## Issues Encountered

- MSVC C++ build tools not in PATH: VS Build Tools 2022 installed but only MSBuild workload present (no cl.exe/link.exe). Multiple fix attempts via winget and VS installer ran but didn't install the compiler. Resolved by switching to GNU toolchain.

## User Setup Required

**Note for other contributors:** Building this project requires MinGW-w64 GCC in PATH. The recommended setup is MSYS2 with the `mingw-w64-x86_64-gcc` package installed:

```
winget install MSYS2.MSYS2
pacman -S --noconfirm mingw-w64-x86_64-gcc
```

Add `C:\msys64\mingw64\bin` to PATH. Then `cargo build` works from any shell.

Alternatively, install Visual Studio with "Desktop development with C++" workload, ensure Developer Command Prompt sets PATH, and remove the `linker = ...` line from `.cargo/config.toml` for the MSVC target.

## Next Phase Readiness

- Phase 1 Plan 02 (storage backends) can start: `Config.storage` and `Config.config_dir` are available
- Phase 1 Plan 03 (WSL executor) can start: `CoreError.WslExec`/`WslFailed` variants defined
- Phase 1 Plan 04 (TUI skeleton) can start: `Config.first_run` flag available for welcome screen decision

---
*Phase: 01-foundation*
*Completed: 2026-02-21*
