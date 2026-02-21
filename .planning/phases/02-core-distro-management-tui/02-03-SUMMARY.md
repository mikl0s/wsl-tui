---
phase: 02-core-distro-management-tui
plan: 03
subsystem: ui
tags: [rust, ratatui, crossterm, tokio, async, eventstream, catppuccin, chrono]

# Dependency graph
requires:
  - phase: 02-core-distro-management-tui/02-01
    provides: DistroInfo, DistroState, WslExecutor lifecycle methods
  - phase: 02-core-distro-management-tui/02-02
    provides: Catppuccin theme constants, KeyBindings, KeyAction

provides:
  - Action enum covering all Phase 2 user operations (27 variants)
  - View enum (Dashboard, Provision, Monitor, Backup, Logs) with display_name()
  - FocusPanel enum (DistroList, Details)
  - ModalState enum (None, Confirm, Help)
  - Expanded App struct with distro list, list state, filter, executor, storage_backend
  - App::refresh_distros() for live wsl.exe polling
  - App::visible_distros(), selected_distro(), move_selection_up/down(), switch_focus(), set_view()
  - EventStream + tokio::select! async event loop with 5-second ticker
  - resolve_action() routing: welcome / modal / filter / normal keybinding dispatch
  - execute_action() with spawn_blocking for wsl.exe calls
  - dashboard.rs: 40/60 split-pane (distro list + details) with responsive layout
  - status_bar.rs: left/centre/right bar with distro state, view name, storage, HH:MM clock

affects:
  - 02-04 (shell attach — uses App, Action, execute_action pattern)
  - 02-05 (modal overlays — uses ModalState::Confirm, execute_action modal routing)

# Tech tracking
tech-stack:
  added:
    - chrono = "0.4" (clock in status bar; added to workspace deps + wsl-tui)
  patterns:
    - "Action enum: input → Action → App mutation via resolve_action + execute_action"
    - "spawn_blocking for synchronous wsl.exe calls inside async event loop"
    - "Responsive layout guard: width < 40 → message, < 60 → single-column, >= 60 → split-pane"
    - "Filter bar injected as top row inside distro list area when active"
    - "Status bar: three Paragraph widgets with Left/Center/Right alignment over same Rect"

key-files:
  created:
    - wsl-tui/src/action.rs
    - wsl-tui/src/ui/dashboard.rs
    - wsl-tui/src/ui/status_bar.rs
  modified:
    - wsl-tui/src/app.rs
    - wsl-tui/src/main.rs
    - wsl-tui/src/ui/mod.rs
    - Cargo.toml
    - wsl-tui/Cargo.toml

key-decisions:
  - "execute_action is async, resolve_action is sync — clean separation lets resolve_action be pure"
  - "Action::None doubles as welcome-screen dismiss sentinel — avoids separate code path for any-key dismiss"
  - "display_name() on View instead of a global match — keeps view-related logic co-located"
  - "Three overlapping Paragraph widgets for status bar sections — simpler than manual string padding"
  - "chrono::Local::now() called inline in status_bar::render — no need to store clock in App"

patterns-established:
  - "Pattern 1: resolve_action returns Action; execute_action takes ownership and mutates App — clean event-loop separation"
  - "Pattern 2: visible_distros() is the single source of truth for what's shown; all render + selection code uses it"
  - "Pattern 3: #[allow(dead_code)] on App fields consumed by later plans — avoids false -D warnings until consumer exists"

requirements-completed: [DIST-01, DIST-03, DIST-04, DIST-05, DIST-06, TUI-01, TUI-07, TUI-08, TUI-12, TUI-15]

# Metrics
duration: 6min
completed: 2026-02-21
---

# Phase 2 Plan 03: Async Event Loop and Dashboard Split-Pane View Summary

**EventStream + tokio::select! event loop with 5-second distro refresh, Action/View/FocusPanel/ModalState enums, 40/60 split-pane dashboard showing live DistroInfo with Catppuccin styling and a HH:MM status bar**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-02-21T22:48:49Z
- **Completed:** 2026-02-21T22:55:01Z
- **Tasks:** 2
- **Files modified:** 8 (3 created, 5 modified)

## Accomplishments

- Upgraded the TUI event loop from synchronous `crossterm::event::read()` to `EventStream` + `tokio::select!` with a 5-second background ticker for automatic distro refresh without blocking the UI
- Expanded `App` with 10+ new fields covering View, FocusPanel, ModalState, distro list, ListState, filter state, WslExecutor, and storage_backend; all methods documented and tested
- Created `action.rs` with 27-variant `Action` enum and `resolve_action`/`execute_action` separation; blocking wsl.exe calls wrapped in `tokio::task::spawn_blocking`
- Built `dashboard.rs` split-pane view: distro list (40%) with state indicators + details panel (60%) with action hints; responsive: single-column at <60 cols, narrow-terminal message at <40 cols
- Built `status_bar.rs` with left/centre/right sections (distro state, view name, storage + chrono clock)
- Added `chrono = "0.4"` to workspace; 99 unit tests + 6 doc tests pass; zero clippy warnings workspace-wide

## Task Commits

Each task was committed atomically:

1. **Task 1: Action enum + App state expansion + async event loop** - `f911764` (feat)
2. **Task 2: Dashboard split-pane layout with distro list and details panel** - `a6c9b66` (feat)

**Plan metadata:** TBD (docs: complete plan)

## Files Created/Modified

- `wsl-tui/src/action.rs` — Action enum (27 variants) covering all Phase 2 user operations
- `wsl-tui/src/app.rs` — Expanded App: View/FocusPanel/ModalState enums, 10+ new fields, 8 new methods, 17 new unit tests (32 total)
- `wsl-tui/src/main.rs` — EventStream + tokio::select! loop, resolve_action/execute_action dispatch, KeyBindings wiring
- `wsl-tui/src/ui/dashboard.rs` — Split-pane dashboard with distro list, details panel, responsive layout
- `wsl-tui/src/ui/status_bar.rs` — Three-section status bar with chrono clock
- `wsl-tui/src/ui/mod.rs` — Updated render dispatcher; pub mod dashboard/status_bar; view placeholder
- `Cargo.toml` — Added chrono = "0.4" to [workspace.dependencies]
- `wsl-tui/Cargo.toml` — Added chrono = { workspace = true }

## Decisions Made

- **execute_action is async, resolve_action is sync:** Clean separation — `resolve_action` is a pure mapping function (no .await needed), `execute_action` owns the async context for spawn_blocking and refresh calls.
- **Action::None doubles as welcome-screen dismiss sentinel:** When `show_welcome` is true, `resolve_action` always returns `Action::None`; `execute_action` handles the None case by calling `dismiss_welcome()`. Avoids a separate any-key code path.
- **Three overlapping Paragraphs for status bar:** Left, centre, and right sections are separate `Paragraph` widgets rendered over the same `Rect` — simpler than manual string padding arithmetic.
- **chrono::Local::now() inline in render:** No need to cache the clock in `App` state; called fresh each frame.
- **display_name() on View impl:** View-related string logic lives with the type rather than in a global match in the renderer.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added DistroState import to app.rs test module**
- **Found during:** Task 1 (running cargo test -p wsl-tui)
- **Issue:** Tests use `DistroState::Running` and `DistroState::Stopped`, but `DistroState` was removed from the top-level `use wsl_core::{...}` import because clippy flagged it as unused in production code. The test module needed its own import.
- **Fix:** Added `use wsl_core::DistroState;` inside `#[cfg(test)] mod tests { ... }`
- **Files modified:** `wsl-tui/src/app.rs`
- **Verification:** `cargo test -p wsl-tui` passes 32 tests
- **Committed in:** `f911764` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 3 — blocking compile error)
**Impact on plan:** Trivial fix; moving the import into the test module is the correct Rust pattern.

## Issues Encountered

- `dlltool.exe not found` when building chrono without `C:\msys64\mingw64\bin` in PATH. Chrono needs the MinGW toolchain for Windows time zone support (same requirement as the rest of the project). Build succeeds with the standard PATH set in `.cargo/config.toml` context. No code change needed — this is a dev environment PATH concern, not a code issue.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `App::refresh_distros()` and `execute_action` ready for Phase 2 Plan 04 shell attach
- `ModalState::Confirm { distro_name, action_label }` ready for Plan 05 remove/export/import flows
- `Action::AttachShell`, `ExportDistro`, `ImportDistro`, `InstallDistro` are defined and dispatched as no-ops; Plan 04-05 implement the bodies
- Dashboard provides the full UI surface that Plan 04-05 overlays will build on top of

## Self-Check: PASSED

- FOUND: `wsl-tui/src/action.rs` (85 lines >= 20 min)
- FOUND: `wsl-tui/src/app.rs` (611 lines >= 100 min)
- FOUND: `wsl-tui/src/main.rs` (315 lines >= 60 min)
- FOUND: `wsl-tui/src/ui/dashboard.rs` (258 lines >= 80 min)
- FOUND: `wsl-tui/src/ui/status_bar.rs` (100 lines >= 30 min)
- FOUND commit f911764: feat(02-03): async EventStream event loop with App state expansion and Action enum
- FOUND commit a6c9b66: feat(02-03): dashboard split-pane view with distro list, details panel, and status bar
- 99 unit tests pass, 6 doc tests pass, zero clippy warnings workspace-wide

---
*Phase: 02-core-distro-management-tui*
*Completed: 2026-02-21*
