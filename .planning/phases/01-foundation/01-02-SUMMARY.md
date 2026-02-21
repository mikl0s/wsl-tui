---
phase: 01-foundation
plan: 02
subsystem: database
tags: [rust, libsql, sqlite, json, async-trait, serde, storage, backend-abstraction]

requires:
  - phase: 01-01
    provides: "Compilable workspace, CoreError enum, Config with StorageMode, 8MB Windows stack linker flag"

provides:
  - "StorageValue and StorageRow types (backend-independent row representation)"
  - "StorageBackend async trait (execute, query, backend_name)"
  - "BackendKind enum with From<StorageMode> conversion"
  - "LibsqlBackend: opens local file or in-memory DB, converts StorageValue to/from libsql types"
  - "JsonBackend: file-based JSON persistence, mini SQL parser for CREATE/INSERT/SELECT/DELETE"
  - "StorageResult struct: backend + backend_name + migration_available flag"
  - "open_storage factory: Auto/Libsql/Json modes with transparent libsql-to-JSON fallback"
  - "migration_available detection: true when libsql active and data.json exists from prior JSON run"
  - "34 new storage tests; 57 total wsl-core tests pass; zero clippy warnings"

affects:
  - "03-wsl-exec: imports storage for distro state persistence"
  - "04-tui: consumes backend_name for status bar display"
  - "05-web: consumes open_storage for web backend"
  - "migration-prompt (Phase 2): migration_available flag drives the UI prompt"

tech-stack:
  added:
    - "async-trait 0.1 (already in workspace — used for dyn StorageBackend trait objects)"
    - "serde derive on StorageValue/JsonData (JSON serialization for json backend)"
    - "serde_json for JsonData persistence (already in workspace)"
  patterns:
    - "StorageValue enum: backend-independent cell type; maps to/from libsql::Value and serde_json"
    - "StorageRow = Vec<StorageValue>: uniform row representation across backends"
    - "Arc<Mutex<JsonData>>: shared mutable state for JsonBackend"
    - "JSON mini-parser: parse_create_table/insert/select/delete helpers via string prefix matching"
    - "open_storage factory: BackendKind::Auto swallows libsql error and retries with JSON"

key-files:
  created:
    - "wsl-core/src/storage/mod.rs (StorageBackend trait, BackendKind, StorageResult, open_storage)"
    - "wsl-core/src/storage/libsql.rs (LibsqlBackend with open/open_memory, execute, query)"
    - "wsl-core/src/storage/json.rs (JsonBackend with JsonData, SQL mini-parser, flush-on-write)"
  modified:
    - "wsl-core/src/lib.rs (added pub mod storage)"

key-decisions:
  - "StorageValue/StorageRow instead of libsql types in trait: keeps trait backend-independent; both backends can implement without libsql dependency"
  - "JsonBackend uses HashMap<String, Vec<StorageRow>> not serde_json::Value map: typed storage avoids double-parsing overhead; rows are already StorageValue slices"
  - "open_storage factory swallows libsql error in Auto mode: caller never knows if fallback occurred — transparent per locked decision"
  - "migration_available detection is detection-only in Phase 1: flag set when libsql active AND data.json exists; no actual data movement until Phase 2 migration prompt UI"
  - "LibsqlBackend holds _db: libsql::Database to keep DB alive: libsql::Connection borrows from Database; Database must outlive Connection — struct fields are dropped in declaration order so _db must come first"

patterns-established:
  - "Pattern 5: Backend abstraction — StorageBackend trait with async_trait; call code uses Box<dyn StorageBackend> and is never coupled to libsql or JSON"
  - "Pattern 6: Factory + auto-fallback — open_storage(dir, BackendKind::Auto) returns whichever backend works; calling code never needs try/catch logic for storage"
  - "Pattern 7: Migration detection — check config_dir/data.json existence after successful libsql open; set migration_available flag in StorageResult"

requirements-completed: [FOUND-02, FOUND-03, DX-04]

duration: 5min
completed: 2026-02-21
---

# Phase 1 Plan 02: Storage Backend System Summary

**StorageBackend async trait with libsql primary and JSON fallback — smoke test passes on Windows, factory auto-switches backends, migration_available detects prior JSON runs for future migration prompt UI**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-21T21:10:51Z
- **Completed:** 2026-02-21T21:16:00Z
- **Tasks:** 2 of 2
- **Files modified:** 4

## Accomplishments

- StorageBackend trait (execute, query, backend_name) with StorageValue/StorageRow types decoupled from any specific backend
- LibsqlBackend opens local file DB (production) or in-memory DB (tests); smoke test passes on Windows confirming 8MB stack fix
- JsonBackend persists to `data.json` with a mini SQL parser supporting CREATE/INSERT/SELECT/DELETE — round-trips correctly across reopen
- `open_storage` factory implements Auto/Libsql/Json modes; Auto mode silently falls back to JSON if libsql fails
- `StorageResult.migration_available` detects prior JSON data when libsql is now active — establishes data contract for Phase 2 migration prompt
- All 57 wsl-core tests pass; `cargo clippy --workspace -- -D warnings` exits 0

## Task Commits

Each task was committed atomically:

1. **Task 1: StorageBackend trait and LibsqlBackend with smoke test** - `a59c723` (feat)
2. **Task 2: JsonBackend with SQL mini-parser and open_storage factory** - `c6d0d11` (feat)

**Plan metadata:** (docs commit — recorded below)

## Files Created/Modified

- `wsl-core/src/storage/mod.rs` — StorageValue, StorageRow, StorageBackend trait, BackendKind, StorageResult, open_storage factory; 5 integration tests
- `wsl-core/src/storage/libsql.rs` — LibsqlBackend struct with _db + conn; execute/query/backend_name; 4 tests (smoke, null handling, execute+query, backend name)
- `wsl-core/src/storage/json.rs` — JsonData/JsonBackend; SQL mini-parser helpers; execute/query/backend_name; flush-on-write; 5 tests (smoke, persistence, delete, backend name, unsupported stmt)
- `wsl-core/src/lib.rs` — Added `pub mod storage;`

## Decisions Made

- **StorageValue instead of libsql::Value in trait:** The trait must be independent of libsql so JsonBackend can implement it without knowing about libsql types. StorageValue mirrors libsql::Value with the same 5 variants (Null/Integer/Real/Text/Blob) but with no external dependency.

- **JSON mini-parser (not rusqlite or sqlparser):** Full SQL parsing would add a dependency and complexity far beyond Phase 1's needs. The application only ever issues a fixed set of statements (CREATE TABLE, INSERT, SELECT, DELETE). String prefix matching is simpler, faster, and entirely sufficient.

- **`_db` prefix on Database field in LibsqlBackend:** The `#[allow(dead_code)]` idiom would silence a warning but the field IS used — it keeps the Database alive. Naming it `_db` communicates intent ("kept alive, not accessed directly") without needing an attribute.

- **migration_available is detection-only in Phase 1:** Phase 2 will add a TUI status bar and migration prompt UI. Doing the detection now establishes the API contract so Phase 4's TUI code can consume the flag immediately.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added `#[derive(Debug)]` to JsonBackend**
- **Found during:** Task 2 (full test run with `cargo test -p wsl-core`)
- **Issue:** `Result<JsonBackend, CoreError>::unwrap_err()` requires `T: Debug`; JsonBackend was missing Debug derive
- **Fix:** Added `#[derive(Debug)]` to `JsonBackend` struct
- **Files modified:** `wsl-core/src/storage/json.rs`
- **Verification:** All 57 tests pass; zero clippy warnings
- **Committed in:** `c6d0d11` (Task 2 commit)

**2. [Out of scope - noted] Linter auto-scaffolded plugin and wsl modules**
- **Found during:** Task 1 (after initial commit)
- **Issue:** A linter/formatter created `wsl-core/src/plugin/` and `wsl-core/src/wsl/` directory stubs and modified `lib.rs` to reference them before these plans were executed
- **Assessment:** The scaffolded code compiled cleanly and all tests pass. These modules will be fully implemented in Plans 03+ of this phase. Not a deviation that required intervention; accepted as pre-scaffold.
- **Committed in:** Prior commit `441b095` (outside this plan's scope)

---

**Total deviations:** 1 auto-fixed (1 missing critical), 1 out-of-scope linter scaffold (accepted)
**Impact on plan:** Auto-fix was a 1-line addition. Linter scaffold was pre-existing. No scope creep on Plan 02 deliverables.

## Issues Encountered

None — plan executed cleanly. The libsql smoke test passed first attempt confirming the 8MB stack linker flag from Plan 01 is working correctly on Windows.

## User Setup Required

None — no external service configuration required. Storage runs embedded.

## Next Phase Readiness

- Phase 1 Plan 03 (WSL executor) can proceed: `open_storage` is available to persist distro state
- Phase 1 Plan 04 (TUI skeleton): `backend_name()` is ready for the status bar; `migration_available` is ready for the migration prompt placement
- Phase 2 migration prompt UI: `StorageResult.migration_available` establishes the detection API

---
*Phase: 01-foundation*
*Completed: 2026-02-21*

## Self-Check: PASSED

- FOUND: wsl-core/src/storage/mod.rs
- FOUND: wsl-core/src/storage/libsql.rs
- FOUND: wsl-core/src/storage/json.rs
- FOUND: .planning/phases/01-foundation/01-02-SUMMARY.md
- FOUND: commit a59c723 (Task 1)
- FOUND: commit c6d0d11 (Task 2)
