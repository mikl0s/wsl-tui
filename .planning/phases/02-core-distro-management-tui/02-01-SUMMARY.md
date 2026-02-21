---
phase: 02-core-distro-management-tui
plan: 01
subsystem: wsl-core
tags: [rust, wsl, distro, parsing, tdd]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: WslExecutor, CoreError, run/list_verbose methods
provides:
  - DistroInfo struct (name, state, version, is_default)
  - DistroState enum (Running, Stopped)
  - OnlineDistro struct (name, friendly_name)
  - parse_list_verbose() — parses wsl --list --verbose output
  - parse_list_online() — parses wsl --list --online output
  - WslExecutor::list_distros() — executes + parses installed distro list
  - WslExecutor::list_online() — executes + parses online catalog
  - WslExecutor::start_distro() — wsl -d <name> -- true
  - WslExecutor::terminate_distro() — wsl --terminate <name>
  - WslExecutor::set_default() — wsl --set-default <name>
  - WslExecutor::unregister() — wsl --unregister <name>
  - WslExecutor::export_distro() — wsl --export <name> <path>
  - WslExecutor::import_distro() — wsl --import <name> <dir> <path>
  - WslExecutor::update_wsl() — wsl --update
affects:
  - 02-02 (theme)
  - 02-03 (dashboard TUI)
  - 02-04 (shell attach)
  - 02-05 (install/export/import modals)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Parse pattern: skip header line, check * for default, whitespace-split remaining fields"
    - "Column separator for online list: 2+ consecutive spaces as delimiter"
    - "Thin wrapper pattern: executor methods delegate to self.run() + parse function"
    - "CI-safe test pattern: parse functions tested with static strings; wsl.exe not called"

key-files:
  created:
    - wsl-core/src/wsl/distro.rs
  modified:
    - wsl-core/src/wsl/executor.rs
    - wsl-core/src/wsl/mod.rs
    - wsl-core/src/lib.rs

key-decisions:
  - "parse_list_verbose uses whitespace split after stripping * prefix — handles variable column widths reliably"
  - "parse_list_online uses splitn(2, 2 spaces) as column separator — matches fixed-width table format"
  - "Executor lifecycle methods are thin 2-line wrappers delegating to self.run() — no extra logic or state"
  - "list_distros/list_online compose executor + parser — single public entry point per operation"

patterns-established:
  - "Pattern 1: distro.rs holds pure data types + parse functions; executor.rs holds wsl.exe dispatch"
  - "Pattern 2: All parse functions return Result<Vec<T>, CoreError::WslExec> on bad input"
  - "Pattern 3: Tests use static sample strings matching exact wsl.exe output format"

requirements-completed: [DIST-01, DIST-02, DIST-03, DIST-04, DIST-05, DIST-06, DIST-08, DIST-09, DIST-10]

# Metrics
duration: 4min
completed: 2026-02-21
---

# Phase 2 Plan 01: Distro Data Types and WslExecutor Lifecycle Methods Summary

**DistroInfo/DistroState/OnlineDistro types with 6 parse tests and 9 WslExecutor lifecycle methods (list_distros through update_wsl) — complete wsl-core API surface for Phase 2 TUI**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-02-21T22:39:49Z
- **Completed:** 2026-02-21T22:43:54Z
- **Tasks:** 2
- **Files modified:** 4 (1 created, 3 modified)

## Accomplishments
- Created `wsl-core/src/wsl/distro.rs` with DistroInfo, DistroState, OnlineDistro types and parse functions for both `wsl --list --verbose` and `wsl --list --online` output formats
- Extended `WslExecutor` with 9 new public methods covering the complete distro lifecycle: list_distros, list_online, start_distro, terminate_distro, set_default, unregister, export_distro, import_distro, update_wsl
- All types re-exported through `wsl::mod` and `lib.rs` making them accessible as `wsl_core::DistroInfo` etc.
- 67 unit tests pass (60 pre-existing + 7 new), plus 5 doc tests; zero clippy warnings; clean workspace build

## Task Commits

Each task was committed atomically:

1. **Task 1: DistroInfo types and parse_list_verbose (TDD)** - `90f24a8` (feat)
2. **Task 2: WslExecutor distro lifecycle methods (TDD)** - `ec22aac` (feat)

**Plan metadata:** TBD (docs: complete plan)

_Note: TDD tasks — tests and implementation written together in single commits as plan allowed_

## Files Created/Modified
- `wsl-core/src/wsl/distro.rs` — DistroInfo struct, DistroState enum, OnlineDistro struct, parse_list_verbose(), parse_list_online() with 6 unit tests + 4 doc tests
- `wsl-core/src/wsl/executor.rs` — Extended with 9 new WslExecutor methods + 2 integration-style unit tests
- `wsl-core/src/wsl/mod.rs` — Added `pub mod distro` and re-exports for DistroInfo, DistroState, OnlineDistro
- `wsl-core/src/lib.rs` — Added public re-exports `pub use wsl::{DistroInfo, DistroState, OnlineDistro, WslExecutor}`

## Decisions Made
- `parse_list_verbose` uses whitespace split after stripping `*` prefix — reliable across variable column widths in wsl.exe output
- `parse_list_online` uses `splitn(2, "  ")` (two spaces) as column separator — matches the fixed-width table format from wsl.exe
- Executor lifecycle methods are thin 2-line wrappers around `self.run()` — no extra logic needed; parse functions handle output transformation
- `list_distros()` and `list_online()` compose the executor + parser as a single public entry point

## Deviations from Plan

None — plan executed exactly as written. The intermittent `test_config_default_keybindings` failure observed during first test run was caused by a pre-existing env var race condition in the config test suite (unrelated to this plan's changes); confirmed by running isolated and full-suite tests in sequence.

## Issues Encountered
- Pre-existing intermittent test failure in `config::tests::test_config_default_keybindings` during first test run. The failure is caused by a parallel test setting `WSL_TUI_STORAGE` env var that leaks into `Config::default()` before the lock can be acquired. This is out-of-scope per deviation rules (pre-existing issue in unrelated module). The test passes consistently when run in isolation or on second run. Logged to deferred items.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness
- All 9 DIST requirements (DIST-01 through DIST-10) have executor methods and data types
- `DistroInfo`, `DistroState`, `OnlineDistro` available as `wsl_core::DistroInfo` etc. for TUI plans 02-03 through 02-05
- `WslExecutor::list_distros()` is the primary method for the dashboard distro list (02-03)
- `WslExecutor::start_distro()` and `terminate_distro()` ready for shell attach (02-04)
- Export/import methods ready for the modal UX (02-05)

## Self-Check: PASSED

- FOUND: wsl-core/src/wsl/distro.rs
- FOUND: wsl-core/src/wsl/executor.rs
- FOUND: .planning/phases/02-core-distro-management-tui/02-01-SUMMARY.md
- FOUND commit 90f24a8: feat(02-01): add DistroInfo types and parse functions
- FOUND commit ec22aac: feat(02-01): add WslExecutor distro lifecycle methods
- 67 unit tests pass, 5 doc tests pass, zero clippy warnings, clean workspace build

---
*Phase: 02-core-distro-management-tui*
*Completed: 2026-02-21*
