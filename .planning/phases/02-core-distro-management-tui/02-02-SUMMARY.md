---
phase: 02-core-distro-management-tui
plan: 02
subsystem: ui
tags: [catppuccin, ratatui, crossterm, keybindings, theme, config]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: Config struct, wsl-core crate structure, Cargo workspace

provides:
  - Catppuccin Mocha theme module with 22 Color::Rgb constants in wsl-tui
  - parse_key_str function parsing key strings to crossterm KeyCode/KeyModifiers
  - KeyBindings struct loaded from Config at startup
  - KeyAction enum covering all 14 Phase 2 user actions
  - RawKeybindings struct in wsl-core Config for TOML deserialization

affects:
  - 02-03 (dashboard UI imports theme constants and KeyBindings)
  - 02-04 (overlays use theme colors and keybinding matches)
  - 02-05 (status bar uses theme colors)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Catppuccin Mocha palette as pub const Color::Rgb values in theme.rs"
    - "Key string notation: single char, ctrl+, alt+, special names, f1-f12"
    - "KeyBindings::from_config panic-at-startup pattern for config validation"
    - "RawKeybindings manual Default impl so programmatic defaults match serde defaults"

key-files:
  created:
    - wsl-tui/src/theme.rs
    - wsl-tui/src/keybindings.rs
  modified:
    - wsl-tui/src/main.rs
    - wsl-tui/src/app.rs
    - wsl-core/src/config.rs
    - wsl-core/src/lib.rs

key-decisions:
  - "RawKeybindings implements Default manually (not derived) so Config::default() returns correct key strings"
  - "KeyBindings::from_config panics at startup on invalid key strings — config validation at startup, not runtime"
  - "parse_key_str returns None for unrecognised strings; callers decide how to handle (panic vs fallback)"
  - "RawKeybindings re-exported from wsl_core root so wsl-tui can reference it in test helpers"

patterns-established:
  - "theme.rs: import colors with `use crate::theme::MAUVE` or `use crate::theme`"
  - "keybindings.rs: construct once at startup with KeyBindings::from_config(&config), store in App"
  - "Event loop usage: if kb.matches(&key, KeyAction::Quit) { app.quit(); }"

requirements-completed: [TUI-13, TUI-14]

# Metrics
duration: 6min
completed: 2026-02-21
---

# Phase 02 Plan 02: Theme and Keybindings Summary

**Catppuccin Mocha theme constants (22 Color::Rgb) and configurable keybindings (14 actions) from TOML config using crossterm key string notation**

## Performance

- **Duration:** 6 min
- **Started:** 2026-02-21T22:39:44Z
- **Completed:** 2026-02-21T22:45:11Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- 22 Catppuccin Mocha `pub const Color::Rgb` constants in `wsl-tui/src/theme.rs`, all RGB values verified against THEME_GUIDELINES.md hex table
- `parse_key_str` parses single chars, `ctrl+`, `alt+`, named keys (`enter`, `esc`, `tab`, `backspace`, `space`, arrows), and `f1`-`f12`
- `KeyBindings::from_config` constructs parsed key pairs at startup; panics on invalid config strings (fast fail)
- `Config.keybindings: RawKeybindings` wired into `Config::load()`, `Config::load_from()`, and `Config::default()`
- `[keybindings]` TOML section documented in `DEFAULT_CONFIG_TOML` with all 14 binding names

## Task Commits

Each task was committed atomically:

1. **Task 1: Catppuccin Mocha theme module** - `ca1da54` (feat)
2. **Task 2: Configurable keybindings system** - `4dd4e30` (feat)

**Plan metadata:** (docs commit — see final_commit below)

## Files Created/Modified

- `wsl-tui/src/theme.rs` — 22 Catppuccin Mocha Color::Rgb constants, module + per-constant doc comments, 3 unit tests
- `wsl-tui/src/keybindings.rs` — `parse_key_str`, `KeyBindings`, `KeyAction`, 10 unit tests
- `wsl-tui/src/main.rs` — `pub mod theme;` and `pub mod keybindings;` declarations added
- `wsl-tui/src/app.rs` — Test helper `make_config` updated with `keybindings` field
- `wsl-core/src/config.rs` — `RawKeybindings` struct, manual `Default` impl, 14 default functions, `Config.keybindings` field wired in load paths, 2 new tests
- `wsl-core/src/lib.rs` — `RawKeybindings` re-exported from crate root

## Decisions Made

- **RawKeybindings manual Default:** `#[derive(Default)]` would produce empty strings. A manual `Default` impl calls the same `default_*()` functions that serde uses, so `Config::default()` and TOML deserialization produce identical values.
- **Panic at startup:** `KeyBindings::from_config` panics on invalid key strings. This is intentional — bad config should be caught at startup rather than silently ignored at runtime (the user would see no response to their configured key and have no feedback).
- **`parse_key_str` returns `Option`:** Callers control error handling. `from_config` uses `expect` (startup panic); future callers could return an error to the user gracefully.
- **`RawKeybindings` re-exported:** Required because `app.rs` test helper constructs `Config` directly with struct literal syntax, which needs access to the type name.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed app.rs test helper missing keybindings field**
- **Found during:** Task 2 (keybindings system — running workspace tests)
- **Issue:** `make_config()` in `app.rs` tests constructed `Config { storage, config_dir, first_run }` as a struct literal — adding `keybindings` to `Config` caused a compile error: "missing field `keybindings`"
- **Fix:** Added `keybindings: wsl_core::RawKeybindings::default()` to the struct literal
- **Files modified:** `wsl-tui/src/app.rs`
- **Verification:** `cargo test --workspace` passes with 88 tests
- **Committed in:** `4dd4e30` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 — Bug)
**Impact on plan:** Required fix; adding a new field to a struct used in test helpers always causes this compile error. No scope creep.

## Issues Encountered

- `#[derive(Default)]` on `RawKeybindings` would produce empty strings (not the configured defaults) — switched to manual `Default` impl that calls the same `default_*()` functions serde uses. Caught by `test_config_default_keybindings` on first run.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `use crate::theme::MAUVE` (and any other constant) compiles in any `wsl-tui` module
- `KeyBindings::from_config(&Config::default())` creates valid bindings for all 14 actions
- Plans 03-05 can import theme constants and `KeyBindings` immediately
- No blockers

---
*Phase: 02-core-distro-management-tui*
*Completed: 2026-02-21*

## Self-Check: PASSED

- FOUND: `wsl-tui/src/theme.rs`
- FOUND: `wsl-tui/src/keybindings.rs`
- FOUND: `wsl-core/src/config.rs`
- FOUND: `.planning/phases/02-core-distro-management-tui/02-02-SUMMARY.md`
- FOUND commit `ca1da54` (feat: Catppuccin Mocha theme module)
- FOUND commit `4dd4e30` (feat: configurable keybindings system)
