---
phase: 01-foundation
plan: 03
subsystem: core-tui
tags: [rust, wsl-executor, encoding, utf16le, utf8, plugin-system, ratatui, crossterm, tui-event-loop, welcome-screen, windows]

requires:
  - "01-01: CoreError enum (WslExec/WslFailed variants), Config with first_run flag"
  - "01-02: storage module compiles (json.rs was committed but not tracked in 01-02-SUMMARY)"
provides:
  - "WslExecutor with runtime WSL_UTF8 detection and UTF-16LE/UTF-8 decoding"
  - "Null-byte stripping from wsl.exe output"
  - "Plugin trait (name/version) and PluginRegistry (register/get/all/count)"
  - "Runnable wsl-tui binary with ratatui::init panic hook"
  - "KeyEventKind::Press filter preventing double-fire on Windows"
  - "App state struct (running/first_run/show_welcome) with unit tests"
  - "Welcome screen: centered block with config path, customization hint, any-key dismiss"
  - "Placeholder main screen: exits cleanly on q"
affects:
  - "04-tui: App struct is the foundation for Phase 2 distro list UI"
  - "05-connectivity: WslExecutor will execute wsl.exe subcommands for distro management"
  - "06-plugins: Plugin trait is the interface for Lua runtime plugins"

tech-stack:
  added:
    - "encoding_rs 0.8: UTF-16LE decoding for wsl.exe output (was in workspace deps, now used)"
    - "ratatui Layout/Constraint/Fill: vertical + horizontal centering for welcome screen"
    - "crossterm KeyEventKind: Press-only event filter (Windows double-fire fix)"
  patterns:
    - "WSL_UTF8_LOCK mutex: serialize all tests that read/write WSL_UTF8 env var"
    - "WslExecutor::decode_output: stateless fn, testable without spawning wsl.exe"
    - "ratatui::init() + ratatui::restore(): guaranteed terminal cleanup including panics"
    - "KeyEventKind::Press filter: prevents double-handling on Windows (critical fix)"
    - "App::first_run stored but #[allow(dead_code)]: Phase 2 consumer not yet present"

key-files:
  created:
    - "wsl-core/src/wsl/mod.rs (pub mod executor; re-export WslExecutor)"
    - "wsl-core/src/wsl/executor.rs (WslExecutor: is_utf8_mode, decode_output, run, list_verbose + 14 tests)"
    - "wsl-core/src/plugin/mod.rs (Plugin trait: name, version with Send+Sync bounds)"
    - "wsl-core/src/plugin/registry.rs (PluginRegistry: register/get/all/count + 9 tests)"
    - "wsl-tui/src/app.rs (App struct + 6 unit tests)"
    - "wsl-tui/src/ui/mod.rs (render dispatcher: welcome vs placeholder)"
    - "wsl-tui/src/ui/welcome.rs (centered welcome block with Catppuccin colors)"
  modified:
    - "wsl-core/src/lib.rs (added pub mod wsl, pub mod plugin; re-exports WslExecutor/Plugin/PluginRegistry)"
    - "wsl-tui/src/main.rs (full event loop: ratatui::init, Config::load, run_app, ratatui::restore)"

key-decisions:
  - "WSL_UTF8_LOCK mutex pattern: same env-lock approach as CONFIG tests — required because WSL_UTF8 is process-global and tests run in parallel threads"
  - "decode_output is a pub fn on WslExecutor (not private): enables direct testing without spawning wsl.exe — CI safety"
  - "Synchronous event::read() for Phase 1: simpler than EventStream; adequate before Phase 2 adds background async tasks"
  - "KeyCode::Char('Q') also quits: defensive — Shift+Q should also exit rather than silently ignoring"
  - "#[allow(dead_code)] on App::first_run: field is semantically correct and will be consumed in Phase 2 status bar; suppressing lint avoids false signal in -D warnings CI"
  - "clamp(44, 72) instead of .min(72).max(44): clippy::manual_clamp lint caught this — fixed for zero-warning compliance"

patterns-established:
  - "Pattern 5: WSL env serialization — same ENV_LOCK approach as config tests; apply to any test reading WSL_UTF8"
  - "Pattern 6: Stateless decode_pub_fn — expose decode logic as pub fn for test-driven verification without I/O"
  - "Pattern 7: Terminal lifecycle — ratatui::init() before loop entry, ratatui::restore() unconditionally after"

requirements-completed: [FOUND-05, FOUND-06, FOUND-08, FOUND-09, DX-04]

duration: 7min
completed: 2026-02-21
---

# Phase 1 Plan 03: WSL Executor, Plugin Registry, and TUI Event Loop Summary

**WslExecutor with runtime UTF-16LE/UTF-8 detection, compile-time PluginRegistry, and ratatui TUI event loop with KeyEventKind::Press filter and polished welcome screen**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-21T21:10:55Z
- **Completed:** 2026-02-21T21:17:58Z
- **Tasks:** 2 of 2
- **Files modified:** 9

## Accomplishments

- `WslExecutor` detects `WSL_UTF8` env at runtime; decodes UTF-16LE (default) or UTF-8 and strips null bytes — 14 unit tests, no `wsl.exe` calls in tests
- `Plugin` trait with `Send + Sync` bounds and `PluginRegistry` with `register`/`get`/`all`/`count` — 9 unit tests covering duplicates, empty registry, and default
- `wsl-tui` binary launches via `ratatui::init()` (auto-installs panic hook); event loop filters `KeyEventKind::Press` only (Windows double-fire fix); `ratatui::restore()` called unconditionally
- Welcome screen centered with Catppuccin colors: config path, customization note, any-key dismiss
- All 6 `App` state unit tests pass; binary compiles; zero clippy warnings across workspace
- Total wsl-core tests: 57 (all passing after adding wsl + plugin modules alongside existing storage/config/error tests)

## Task Commits

Each task was committed atomically:

1. **Task 1: WSL executor with encoding detection and Plugin trait with registry** — `441b095` (feat)
2. **Task 2: TUI event loop skeleton with panic hook, KeyEventKind filter, and welcome screen** — `3baef84` (feat)

**Plan metadata:** (docs commit — recorded below)

## Files Created/Modified

- `wsl-core/src/wsl/mod.rs` — Module declaration, re-export of `WslExecutor`
- `wsl-core/src/wsl/executor.rs` — `WslExecutor`: `is_utf8_mode()`, `decode_output()`, `run()`, `list_verbose()`; 14 unit tests with `WSL_UTF8_LOCK` mutex pattern
- `wsl-core/src/plugin/mod.rs` — `Plugin` trait: `name() -> &str`, `version() -> &str`, `Send + Sync`
- `wsl-core/src/plugin/registry.rs` — `PluginRegistry`: `Vec<Box<dyn Plugin>>`, `register/get/all/count`; 9 unit tests
- `wsl-core/src/lib.rs` — Added `pub mod wsl`, `pub mod plugin`; re-exports `WslExecutor`, `Plugin`, `PluginRegistry`
- `wsl-tui/src/app.rs` — `App { running, first_run, show_welcome }`, `new()`, `quit()`, `dismiss_welcome()`; 6 unit tests
- `wsl-tui/src/ui/mod.rs` — `render()` dispatcher: welcome vs. placeholder; `render_placeholder()`
- `wsl-tui/src/ui/welcome.rs` — Centered welcome block (vertical + horizontal `Layout::Fill`), Catppuccin colors, any-key dismiss hint
- `wsl-tui/src/main.rs` — Full `#[tokio::main]`, `ratatui::init()`, `run_app()` with `KeyEventKind::Press` filter, `ratatui::restore()`

## Decisions Made

- **WSL_UTF8_LOCK mutex:** Same `ENV_LOCK` pattern established in Plan 01 — serializes tests reading/writing `WSL_UTF8` env var to prevent inter-test interference in parallel Rust test runs.

- **Synchronous `event::read()` for Phase 1:** The plan note confirms this is correct — Phase 1 has no background async tasks competing for the event loop. `EventStream` + `tokio::select!` is deferred to Phase 2.

- **`#[allow(dead_code)]` on `App::first_run`:** Field is structurally correct and will be consumed by the Phase 2 status bar. Suppressing the lint prevents false `-D warnings` failures before the consumer is added.

- **`clamp(44, 72)` vs `.min(72).max(44)`:** Clippy `manual_clamp` lint caught the pattern. Fixed inline.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing `storage/json.rs` blocked wsl-core compilation**
- **Found during:** Task 1 setup (running initial `cargo test -p wsl-core`)
- **Issue:** `storage/mod.rs` had `pub mod json;` and `pub use json::JsonBackend;` but `json.rs` was not present in the working tree. The file was committed (visible via `git show`) but the working tree differed. `git checkout wsl-core/src/storage/json.rs` restored the full implementation.
- **Fix:** Restored committed `json.rs` via `git checkout`; confirmed it was fully implemented (366 lines, `JsonData` struct, SQL mini-parser for CREATE/INSERT/SELECT/DELETE)
- **Files modified:** None — restoration only
- **Commit:** Part of environment setup; not a separate commit
- **Impact:** Zero. Committed code was complete; working tree was behind.

**2. [Rule 1 - Bug] Clippy `manual_clamp` warning in welcome.rs**
- **Found during:** Task 2 clippy verification
- **Issue:** `area.width.min(72).max(44)` triggers `clippy::manual_clamp` lint under `-D warnings`
- **Fix:** Changed to `area.width.clamp(44, 72)`
- **Files modified:** `wsl-tui/src/ui/welcome.rs`
- **Commit:** Included in Task 2 commit `3baef84`

---

**Total deviations:** 2 (1 environment/restoration, 1 auto-fixed clippy lint)
**Impact on plan:** None — both resolved inline before task commits.

## Verification Results

| Check | Result |
|-------|--------|
| `cargo test -p wsl-core -- wsl` | 14/14 PASS |
| `cargo test -p wsl-core -- plugin` | 9/9 PASS |
| `cargo test -p wsl-tui` | 6/6 PASS |
| `cargo build -p wsl-tui` | PASS (binary built) |
| `cargo clippy --workspace -- -D warnings` | 0 warnings |
| Total wsl-core tests (all suites) | 57/57 PASS |

## Next Phase Readiness

- Phase 1 Plan 04 (performance targets / DX-05 / DX-06): startup time and memory targets are trivially met by the Phase 1 skeleton; meaningful measurement deferred to Phase 2
- Phase 2 (Core TUI): `App` struct is the foundation for distro list UI; `WslExecutor` ready to call `wsl.exe --list --verbose`
- Phase 6 (plugins): `Plugin` trait and `PluginRegistry` are the compile-time interface for Lua runtime plugins

---
*Phase: 01-foundation*
*Completed: 2026-02-21*
