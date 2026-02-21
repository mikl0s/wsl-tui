---
phase: 02-core-distro-management-tui
plan: 04
subsystem: ui
tags: [rust, ratatui, crossterm, catppuccin, modal, overlay, shell-attach, filter]

# Dependency graph
requires:
  - phase: 02-core-distro-management-tui/02-03
    provides: App struct with filter_active/filter_text/ModalState fields, dashboard split-pane, action routing

provides:
  - popup.rs shared popup_area() helper using Flex::Center (used by both modals)
  - help_overlay.rs with context-aware keybinding overlay (double-border, 70%x80%, Catppuccin styled)
  - confirm_modal.rs with y/N confirmation popup (double-border, RED border, warning text)
  - App::activate_filter(), deactivate_filter(), filter_push_char(), filter_pop_char() methods
  - Shell attach: instant TUI swap via ratatui::restore() + wsl.exe + ratatui::init() restore
  - Auto-start stopped distros before shell attach
  - ConfirmYes executes executor.unregister() via spawn_blocking
  - Up/Down/j/k navigation while filter bar is active
  - 16 new unit tests across app.rs, main.rs, help_overlay.rs, confirm_modal.rs

affects:
  - 02-05 (import/export flows — uses the same ModalState::Confirm pattern)
  - All future views that need popups (reuse popup.rs popup_area utility)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "popup_area(area, %, %) using Layout::vertical/horizontal with Flex::Center — reusable centered modal utility"
    - "Shell attach pattern: ratatui::restore() → wsl.exe Command::new().status() → ratatui::init()"
    - "AttachShell handled in run_app not execute_action — gives access to &mut terminal"
    - "ConfirmYes clones modal state before clearing — avoids borrow conflict when running async executor call"
    - "vec![] macro instead of push for static keybinding lists — avoids clippy::vec_init_then_push"

key-files:
  created:
    - wsl-tui/src/ui/popup.rs
    - wsl-tui/src/ui/help_overlay.rs
    - wsl-tui/src/ui/confirm_modal.rs
  modified:
    - wsl-tui/src/app.rs
    - wsl-tui/src/main.rs
    - wsl-tui/src/ui/mod.rs

key-decisions:
  - "Shell attach lives in run_app (not execute_action) — needs &mut terminal for ratatui::restore/init"
  - "popup.rs shared utility — both modals reuse popup_area() rather than duplicating Flex::Center layout"
  - "deactivate_filter() resets selection to index 0 — predictable UX when exiting filter mode"
  - "Up/Down/j/k routed in filter mode — user can navigate filtered results without leaving filter"
  - "ConfirmYes clones ModalState before clearing — avoids Rust borrow checker conflict on app.modal"

patterns-established:
  - "Pattern: popup_area() as shared utility for all modal overlays — import from ui::popup"
  - "Pattern: modal rendering order — dashboard first, modals last (rendered on top)"
  - "Pattern: destructive action flow — RemoveDistro sets ModalState::Confirm, ConfirmYes executes via spawn_blocking"

requirements-completed: [CONN-01, DIST-07, TUI-09, TUI-10]

# Metrics
duration: 5min
completed: 2026-02-21
---

# Phase 2 Plan 04: Help Overlay, Fuzzy Filter, Confirmation Modal, and Shell Attach Summary

**Help overlay with context-aware keybindings, y/N confirm modal with double-border, inline filter with navigation support, and instant TUI-swap shell attach with auto-start and full restore**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-02-21T22:58:43Z
- **Completed:** 2026-02-21T23:03:07Z
- **Tasks:** 2
- **Files modified:** 6 (3 created, 3 modified)

## Accomplishments

- Created `popup.rs` with a reusable `popup_area()` helper using `Flex::Center` via `Layout::vertical/horizontal` — shared by both modal overlays with no duplication
- Built `help_overlay.rs`: centered 70%x80% modal with double-border, MAUVE border, LAVENDER title, context-aware keybinding sections (Navigation, Distro Actions, Search & UI) in Catppuccin Mocha palette
- Built `confirm_modal.rs`: 60%x30% centered modal with double RED border, distro name in MAUVE bold, WARNING in YELLOW, "[y]" in RED bold — matches "cannot be undone" spec exactly
- Added `activate_filter()`, `deactivate_filter()`, `filter_push_char()`, `filter_pop_char()` to `App` with full doc comments + examples + unit tests (16 new tests total)
- Implemented shell attach in `attach_shell()` in `run_app`: instant swap via `ratatui::restore()`, `wsl.exe -d <name>` with inherited stdio, `ratatui::init()` full restore, auto-start stopped distros, refresh list on return
- Wired `ConfirmYes` to call `executor.unregister()` via `spawn_blocking` — the confirmation modal now actually removes distros
- Added Up/Down/j/k navigation routing while filter is active

## Task Commits

Each task was committed atomically:

1. **Task 1: Help overlay and fuzzy filter** - `1db82a5` (feat)
2. **Task 2: Confirmation modal and shell attach** - `2038d37` (feat)

**Plan metadata:** TBD (docs: complete plan)

## Files Created/Modified

- `wsl-tui/src/ui/popup.rs` — Shared `popup_area(area, %x, %y)` helper using `Flex::Center`
- `wsl-tui/src/ui/help_overlay.rs` — Context-aware help overlay with section headers and key bindings
- `wsl-tui/src/ui/confirm_modal.rs` — y/N confirmation modal with "cannot be undone" warning
- `wsl-tui/src/app.rs` — 4 new filter methods (activate/deactivate/push_char/pop_char) + 5 new unit tests
- `wsl-tui/src/main.rs` — Shell attach in `run_app`, `ConfirmYes` with `unregister`, filter routing with Up/Down/j/k
- `wsl-tui/src/ui/mod.rs` — Declares popup/help_overlay/confirm_modal modules, routes modal rendering after dashboard

## Decisions Made

- **Shell attach in run_app:** `execute_action` doesn't have `&mut terminal`, so `AttachShell` is intercepted before calling `execute_action` and handled by `attach_shell()` which has full terminal access.
- **Shared popup.rs utility:** Both `help_overlay` and `confirm_modal` need identical `popup_area()` logic — extracted to `ui::popup` to avoid duplication.
- **deactivate_filter() resets to index 0:** Rather than try to restore the "previous" selection (complex state), reset to 0 for a predictable UX after clearing the filter.
- **Up/Down/j/k in filter mode:** Routing these keys in filter mode lets users navigate the filtered results without deactivating the filter — better UX than forcing Esc first.
- **ConfirmYes clones modal state before clearing:** `app.modal.clone()` captures `distro_name` before `app.modal = ModalState::None` — avoids Rust borrow conflict between reading modal fields and writing to `app`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed clippy::vec_init_then_push in help_overlay.rs**
- **Found during:** Task 1 (running cargo clippy -p wsl-tui -- -D warnings)
- **Issue:** Clippy flagged Vec::new() followed by sequential .push() calls in `build_dashboard_help()` as `vec_init_then_push` warning treated as error
- **Fix:** Converted to `vec![...]` macro with all items inline
- **Files modified:** `wsl-tui/src/ui/help_overlay.rs`
- **Verification:** `cargo clippy --workspace -- -D warnings` passes with zero warnings
- **Committed in:** `1db82a5` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — trivial clippy lint)
**Impact on plan:** Zero impact — the vec![] form is more idiomatic Rust anyway.

## Issues Encountered

None — the shell attach pattern, modal routing, and filter methods all compiled cleanly on the first attempt. The only issue was the clippy lint fixed inline.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Help overlay and confirm modal provide the UI overlay foundation for all future modal patterns in Phase 05 (import/export flows)
- Shell attach is fully functional: press Enter on any distro to get a shell, Ctrl+D or `exit` returns to TUI with preserved state
- `popup.rs` is ready for reuse by any future modal (import path input, export path input, error messages)
- `ModalState::Confirm` pattern established — future destructive actions follow: set modal → render confirm → ConfirmYes executes

## Self-Check: PASSED

- FOUND: `wsl-tui/src/ui/popup.rs` (created)
- FOUND: `wsl-tui/src/ui/help_overlay.rs` (created, 140+ lines)
- FOUND: `wsl-tui/src/ui/confirm_modal.rs` (created, 100+ lines)
- FOUND commit 1db82a5: feat(02-04): help overlay, fuzzy filter, and popup utility
- FOUND commit 2038d37: feat(02-04): confirmation modal and shell attach with full restore
- 48 wsl-tui unit tests pass, 67 wsl-core tests pass, 5 doc-tests pass, zero clippy warnings workspace-wide

---
*Phase: 02-core-distro-management-tui*
*Completed: 2026-02-21*
