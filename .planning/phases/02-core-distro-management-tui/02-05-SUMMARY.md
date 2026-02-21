---
phase: 02-core-distro-management-tui
plan: 05
subsystem: ui
tags: [rust, ratatui, crossterm, catppuccin, modal, gauge, mpsc, tokio, spawn-blocking, export, import, install]

# Dependency graph
requires:
  - phase: 02-core-distro-management-tui/02-03
    provides: App struct with ModalState, action routing, execute_action, run_app with tokio::select!
  - phase: 02-core-distro-management-tui/02-04
    provides: popup.rs popup_area() helper, confirm modal pattern, ModalState::Confirm

provides:
  - ModalState::InstallPicker with online distro list using ratatui List widget
  - ModalState::InstallProgress with Gauge widget and tokio::sync::mpsc progress channel
  - ModalState::UpdateProgress for wsl --update with completion feedback
  - ModalState::ExportInput single-field text input modal with cursor rendering
  - ModalState::ImportInput three-field form with Tab field cycling and cursor
  - install_modal.rs: render_install_picker, render_install_progress, render_update_progress
  - input_modal.rs: render_export_input, render_import_input with render_with_cursor helper
  - App::install_rx: Option<mpsc::Receiver<(String, u16, bool)>> for background task progress
  - Action variants: ModalInputChar, ModalInputBackspace, ModalInputLeft, ModalInputRight, ModalInputTab
  - Background install/update via tokio::task::spawn_blocking with polling loop and progress channel
  - default_export_path() helper using dirs::download_dir() for sensible default paths
  - Manual PartialEq for ModalState (ListState is Copy but not PartialEq)

affects:
  - Phase 3+ plans using modal overlays (can reuse input_modal.rs pattern)
  - Any future modal needing background task progress (use install_rx channel pattern)

# Tech tracking
tech-stack:
  added:
    - dirs = { workspace = true } added to wsl-tui/Cargo.toml
  patterns:
    - "Background task progress pattern: spawn_blocking + mpsc::channel<(String, u16, bool)> + tokio::select! branch"
    - "Manual PartialEq for structs containing non-PartialEq fields (ListState) — compare only semantic fields"
    - "Cursor rendering via render_with_cursor(): insert '_' at char index into Vec<char>"
    - "Tab field cycling: active_field = (active_field + 1) % 3 with cursor reset to field length"
    - "Gauge widget with use_unicode(true) for smooth install progress bar"
    - "Modal text input: char-level editing via Vec<char> collect/insert/remove, cursor tracks char index"
    - "Async progress branch in tokio::select! uses std::future::pending() when install_rx is None"

key-files:
  created:
    - wsl-tui/src/ui/install_modal.rs
    - wsl-tui/src/ui/input_modal.rs
  modified:
    - wsl-tui/src/app.rs
    - wsl-tui/src/action.rs
    - wsl-tui/src/main.rs
    - wsl-tui/src/ui/mod.rs
    - wsl-tui/Cargo.toml

key-decisions:
  - "Manual PartialEq for ModalState — ListState is Copy not PartialEq; derive(PartialEq) conflicts with manual impl; removed PartialEq from derive, kept manual impl only"
  - "Both tasks committed together — input_modal.rs was created alongside mod.rs which references it; splitting would have left an uncompilable intermediate state"
  - "run_install_with_progress uses polling loop with try_wait() at 500ms intervals — wsl.exe --install has no machine-readable progress; time-based estimation capped at 90% until process exits"
  - "std::future::pending() in tokio::select! when install_rx is None — avoids Option<Receiver> branch complexity while keeping the select! clean"
  - "Capital I for InstallDistro, lowercase u for UpdateWsl — avoids conflict with lowercase i (Import keybinding)"
  - "dirs::download_dir() with home_dir() fallback for default export path — provides sensible Windows path without hardcoding"

patterns-established:
  - "Pattern: install_rx channel pattern — spawn_blocking sends (step, percent, completed) tuples; main loop receives and updates ModalState fields; completed=true drops receiver"
  - "Pattern: multi-field modal Tab cycling — active_field: u8, % field_count, cursor resets to new field length"
  - "Pattern: cursor rendering with Vec<char> — convert to Vec<char> for O(n) char-indexed ops, insert '_' at cursor pos"

requirements-completed: [DIST-02, DIST-08, DIST-09, DIST-10]

# Metrics
duration: 8min
completed: 2026-02-22
---

# Phase 2 Plan 05: Install Flow, Progress Modal, and Export/Import Text Input Modals Summary

**Online distro picker with j/k navigation + Gauge progress modal for wsl --install/update, plus single-field export and three-field import text input modals with char-level editing via mpsc progress channel**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-02-22T10:06:38Z
- **Completed:** 2026-02-22T10:14:38Z
- **Tasks:** 2 (combined into one commit)
- **Files modified:** 7 (2 created, 5 modified)

## Accomplishments

- Created `install_modal.rs` with three renderers: `render_install_picker` (List widget with MAUVE highlight), `render_install_progress` (Gauge widget with `use_unicode(true)`, step label + percent), and `render_update_progress` (step text with completion hint)
- Created `input_modal.rs` with `render_export_input` (single field with cursor) and `render_import_input` (three stacked fields with Tab cycling, active field highlighted in MAUVE), plus `render_with_cursor()` helper using `Vec<char>` indexing
- Extended `ModalState` with `InstallPicker`, `InstallProgress`, `UpdateProgress`, `ExportInput`, `ImportInput` variants; implemented manual `PartialEq` to handle `ListState` (Copy but not PartialEq)
- Added `install_rx: Option<mpsc::Receiver<(String, u16, bool)>>` to `App`; background install/update spawn blocking tasks send `(step, percent, completed)` tuples; `tokio::select!` receives them and updates modal state inline
- Added 5 new `Action` variants (`ModalInputChar`, `ModalInputBackspace`, `ModalInputLeft`, `ModalInputRight`, `ModalInputTab`) with full char-level editing in `execute_action`
- Wired all four operations: `InstallDistro` (I) → picker → progress → refresh; `UpdateWsl` (u) → update progress; `ExportDistro` (e) → export modal; `ImportDistro` (i) → import modal
- 19 new unit tests: `test_install_rx_none_initially`, `test_install_rx_some_after_assign`, `test_install_picker_modal_equality`, `test_install_progress_dismiss_on_completed`, `test_export_input_modal_equality`, `test_import_input_modal_equality`, `test_export_modal_triggers`, `test_import_modal_triggers`, `test_export_input_char_handling`, `test_import_field_cycling`, `test_install_picker_to_progress_transition`, `test_install_rx_some_after_trigger`, plus 5 cursor rendering tests in `input_modal`

## Task Commits

Both tasks were committed together (input_modal.rs was created alongside mod.rs which references it):

1. **Task 1 + Task 2: Install flow, progress modal, export/import modals** - `b8345b9` (feat)

**Plan metadata:** TBD (docs: complete plan)

## Files Created/Modified

- `wsl-tui/src/ui/install_modal.rs` — Online distro picker with List widget; Gauge-based install progress; update progress modal
- `wsl-tui/src/ui/input_modal.rs` — Export single-field modal; import 3-field modal with Tab cycling; `render_with_cursor()` helper
- `wsl-tui/src/app.rs` — 5 new ModalState variants; manual PartialEq; `install_rx` field; 9 new unit tests
- `wsl-tui/src/action.rs` — 5 new Action variants for modal text input
- `wsl-tui/src/main.rs` — Full modal routing in `resolve_action`; `execute_action` handles all new actions; `tokio::select!` branch for progress channel; `run_install_with_progress` and `run_update_with_progress` background task functions; `default_export_path()` helper; 10 new unit tests
- `wsl-tui/src/ui/mod.rs` — Declares `install_modal` and `input_modal` modules; routes all new ModalState variants in render dispatcher
- `wsl-tui/Cargo.toml` — Added `dirs = { workspace = true }` dependency

## Decisions Made

- **Manual PartialEq for ModalState:** `ListState` is `Copy` (not `PartialEq`), so `derive(PartialEq)` would require `ListState: PartialEq`. Removed `PartialEq` from derive, kept only manual `impl PartialEq for ModalState` that compares `online_distros` and ignores scroll position.
- **Tasks committed together:** `input_modal.rs` was created at the same time as `ui/mod.rs` which declares `pub mod input_modal`. Splitting into separate commits would have left an uncompilable state between tasks.
- **std::future::pending() in tokio::select!:** When `install_rx` is `None`, the progress channel branch needs to yield without blocking. `std::future::pending()` creates a never-resolving future — the branch is effectively skipped while other branches can run.
- **Capital I for InstallDistro:** Default keybindings already use lowercase `i` for Import. Capital `I` avoids the conflict without requiring a keybinding config change.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed PartialEq derive conflict with manual impl**
- **Found during:** Task 1 (first cargo build)
- **Issue:** `#[derive(PartialEq)]` on `ModalState` conflicted with `impl PartialEq for ModalState` — Rust E0119 "conflicting implementations"
- **Fix:** Removed `PartialEq` from `#[derive(Debug, Clone, PartialEq)]` — kept only `Debug` and `Clone`; manual `impl PartialEq` handles all variants
- **Files modified:** `wsl-tui/src/app.rs`
- **Verification:** `cargo build -p wsl-tui` passes
- **Committed in:** `b8345b9`

**2. [Rule 1 - Bug] Fixed clone_on_copy clippy warning in ui/mod.rs**
- **Found during:** Task 1 (cargo clippy -D warnings)
- **Issue:** `list_state.clone()` on `ListState` which implements `Copy` — clippy::clone_on_copy
- **Fix:** Changed to `*list_state` (dereference copy)
- **Files modified:** `wsl-tui/src/ui/mod.rs`
- **Verification:** `cargo clippy --workspace -- -D warnings` passes with zero warnings
- **Committed in:** `b8345b9`

---

**Total deviations:** 2 auto-fixed (both Rule 1 — compiler/clippy errors caught during first build)
**Impact on plan:** Zero impact on delivered functionality. Both fixes were necessary for compilation and lint compliance.

## Issues Encountered

None — install flow, modal routing, text input editing, and background task channel all compiled cleanly after the two auto-fixes above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All DIST requirements (DIST-01 through DIST-10) are now wired end-to-end in the TUI
- Phase 2 is complete — the full distro lifecycle (list, attach, start, stop, set default, remove, export, import, install, update) is functional
- The background task progress pattern (`install_rx` channel + `tokio::select!` branch) is ready for Phase 3 provisioning operations that need progress feedback
- `input_modal.rs` and `install_modal.rs` are available for reuse by any future flow needing text input or progress display

## Self-Check: PASSED

- FOUND: `wsl-tui/src/ui/install_modal.rs` (created, 80+ lines)
- FOUND: `wsl-tui/src/ui/input_modal.rs` (created, 130+ lines)
- FOUND commit b8345b9: feat(02-05): install flow, progress modal, export/import text input modals
- 67 wsl-tui tests pass, 67 wsl-core tests pass, 5 doc-tests pass, zero clippy warnings workspace-wide

---
*Phase: 02-core-distro-management-tui*
*Completed: 2026-02-22*
