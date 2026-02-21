# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-21)

**Core value:** A user can go from "WSL installed" to "fully provisioned dev environment" in under 5 minutes by selecting packs and hitting go — reproducibly, idempotently, every time.
**Current focus:** Phase 1 — Foundation

## Current Position

Phase: 1 of 7 (Foundation)
Plan: 2 of 4 in current phase
Status: In progress
Last activity: 2026-02-21 — Plan 01-02 complete (storage backend system)

Progress: [██░░░░░░░░] 7%

## Performance Metrics

**Velocity:**
- Total plans completed: 2
- Average duration: 14 min
- Total execution time: 0.5 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-foundation | 2/4 | 29 min | 14 min |

**Recent Trend:**
- Last 5 plans: 01-01 (24 min), 01-02 (5 min)
- Trend: storage plan was fast (pre-scaffolded by linter)

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

### Pending Todos

None.

### Blockers/Concerns

- [Phase 1 ongoing]: MSVC C++ workload not installed on dev machine — GNU toolchain in use; building with MSVC requires installing Desktop development with C++ in VS Build Tools and adding to PATH
- [Phase 3]: Pack idempotency step state schema is an open design question — research-phase recommended during Phase 3 planning
- [Phase 5]: ConPTY + Ratatui embedded terminal has sparse Rust documentation — if PTY is re-scoped into v1, a spike is required before design commitment (currently deferred to v2)
- [Phase 6]: mlua sandbox design (safe stdlib subsets, UserData exposure) has limited authoritative documentation — research-phase recommended during Phase 6 planning

## Session Continuity

Last session: 2026-02-21
Stopped at: Completed 01-02-PLAN.md (storage backend system)
Resume file: .planning/phases/01-foundation/01-02-SUMMARY.md
