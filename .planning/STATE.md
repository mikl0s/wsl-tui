# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-21)

**Core value:** A user can go from "WSL installed" to "fully provisioned dev environment" in under 5 minutes by selecting packs and hitting go — reproducibly, idempotently, every time.
**Current focus:** Phase 2 — Core TUI

## Current Position

Phase: 2 of 7 (Core Distro Management TUI)
Plan: 4 of 5 in current phase
Status: Phase 2 in progress
Last activity: 2026-02-21 — Plan 02-04 complete (help overlay, confirm modal, shell attach, fuzzy filter)

Progress: [███████░░░] 30%

## Performance Metrics

**Velocity:**
- Total plans completed: 7
- Average duration: 7 min
- Total execution time: 0.92 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-foundation | 4/4 | 40 min | 10 min |
| 02-core-distro-management-tui | 4/5 | 21 min | 5 min |

**Recent Trend:**
- Last 5 plans: 01-04 (4 min), 02-01 (4 min), 02-02 (6 min), 02-03 (6 min), 02-04 (5 min)
- Trend: consistent 5-6 min execution; TUI interaction model now complete

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Pre-Phase 1]: Shell attach (CONN-01) ships in Phase 2 with Core TUI, not deferred to Phase 5 — it is table stakes for the distro management MVP
- [Pre-Phase 1]: Embedded PTY (ConPTY) deferred to v2 — requires a spike; Phase 5 covers external terminal and Termius only
- [Pre-Phase 1]: Rust 1.88 MSRV set by sysinfo 0.37.2; Rust 2024 edition required by ratatui 0.30
- [Pre-Phase 1]: libsql Windows stack overflow requires `/STACK:8000000` linker flag in .cargo/config.toml — must be Phase 1 day one
- [01-01]: GNU toolchain (x86_64-pc-windows-gnu) chosen over MSVC — MSVC C++ workload not in PATH on this machine; MinGW-w64 GCC from MSYS2 resolves link.exe ambiguity; MSVC target config preserved for future
- [01-01]: Config::load_from(PathBuf) test helper added — avoids touching ~/.wsl-tui/ in tests while keeping production API clean
- [01-01]: ENV_LOCK mutex required for ALL Config::load_from tests — Rust test process shares env vars; any load_from test is affected by WSL_TUI_* leakage from other tests
- [01-02]: StorageValue/StorageRow instead of libsql types in trait — keeps StorageBackend backend-independent; both backends implement without coupling to libsql
- [01-02]: open_storage factory swallows libsql error in Auto mode — transparent fallback per locked decision; calling code never knows if JSON was used
- [01-02]: migration_available detection-only in Phase 1 — flag set when libsql active AND data.json exists; actual migration deferred to Phase 2 migration prompt UI
- [01-03]: WSL_UTF8_LOCK mutex pattern established — same ENV_LOCK approach for WSL_UTF8 env var; required because wsl.exe output tests read process-global env in parallel
- [01-03]: decode_output as public fn on WslExecutor — enables direct unit testing without spawning wsl.exe; CI-safe pattern
- [01-03]: #[allow(dead_code)] on App::first_run — field structurally correct, Phase 2 consumer not yet present; suppresses false -D warnings lint
- [01-03]: Synchronous event::read() for Phase 1 — no background async tasks yet; EventStream + tokio::select! deferred to Phase 2
- [01-04]: Read actual source before documenting — all CLAUDE.md content verified against real code; aspirational descriptions replaced with actual implementations
- [02-01]: parse_list_verbose uses whitespace split after stripping * prefix — handles variable column widths reliably across wsl.exe output variations
- [02-01]: parse_list_online uses splitn(2, 2 spaces) as column separator — matches fixed-width table format from wsl.exe --list --online
- [02-01]: Executor lifecycle methods are thin 2-line wrappers around self.run() — no extra logic needed; parse functions handle output transformation
- [02-02]: RawKeybindings implements Default manually — #[derive(Default)] yields empty strings; manual impl calls same default_*() functions that serde uses, so Config::default() and TOML deserialization produce identical values
- [02-02]: KeyBindings::from_config panics at startup on invalid key strings — config validation at startup not runtime; user sees clear message rather than silent no-op
- [02-02]: parse_key_str returns Option — callers control error handling; from_config uses expect (startup panic); future callers can return errors to user
- [02-03]: execute_action is async, resolve_action is sync — clean separation; resolve_action is a pure key→Action mapping function with no .await; execute_action has the async context for spawn_blocking
- [02-03]: Action::None doubles as welcome-screen dismiss sentinel — when show_welcome is true, any key maps to None, execute_action handles None by calling dismiss_welcome()
- [02-03]: Three overlapping Paragraphs for status bar — Left/Centre/Right alignment over the same Rect; simpler than manual string padding arithmetic
- [02-03]: chrono::Local::now() called inline in render, not cached in App — no state management needed for a clock
- [02-04]: Shell attach lives in run_app (not execute_action) — needs &mut terminal for ratatui::restore/init; AttachShell intercepted before execute_action call
- [02-04]: popup.rs shared utility — both modals reuse popup_area() rather than duplicating Flex::Center layout code
- [02-04]: deactivate_filter() resets selection to index 0 — predictable UX when exiting filter mode; no "previous selection" state needed
- [02-04]: ConfirmYes clones ModalState before clearing — avoids Rust borrow conflict between reading modal fields and writing to app.modal

### Pending Todos

None.

### Blockers/Concerns

- [Phase 1 ongoing]: MSVC C++ workload not installed on dev machine — GNU toolchain in use; building with MSVC requires installing Desktop development with C++ in VS Build Tools and adding to PATH
- [Phase 3]: Pack idempotency step state schema is an open design question — research-phase recommended during Phase 3 planning
- [Phase 5]: ConPTY + Ratatui embedded terminal has sparse Rust documentation — if PTY is re-scoped into v1, a spike is required before design commitment (currently deferred to v2)
- [Phase 6]: mlua sandbox design (safe stdlib subsets, UserData exposure) has limited authoritative documentation — research-phase recommended during Phase 6 planning

## Session Continuity

Last session: 2026-02-21
Stopped at: Completed 02-04-PLAN.md (help overlay, confirm modal, shell attach, fuzzy filter)
Resume file: .planning/phases/02-core-distro-management-tui/02-04-SUMMARY.md
