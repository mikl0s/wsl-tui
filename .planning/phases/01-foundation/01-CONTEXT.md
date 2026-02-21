# Phase 1: Foundation - Context

**Gathered:** 2026-02-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Compilable Rust workspace scaffold with embedded storage (libsql + JSON fallback), WSL command executor with encoding detection, compile-time plugin registry, TUI event loop skeleton, and CLAUDE.md living documents. No user-facing features — this phase plants the abstractions that every subsequent phase depends on.

</domain>

<decisions>
## Implementation Decisions

### Config & first-run experience
- First run shows a **welcome screen** with key info (config location, how to customize) before proceeding to the main TUI
- Config directory is `~/.wsl-tui/` — auto-created on first run
- Default `config.toml` is **fully commented** — all available options present but commented out with descriptions, like a typical dotfile
- **Environment variable overrides** supported: `WSL_TUI_*` env vars take precedence over config.toml values (useful for CI/scripting)

### Coding standards (CLAUDE.md)
- **Unit tests required** for every public function, integration tests for cross-crate boundaries
- Error handling pattern, Rust conventions (unwrap policy, visibility, doc comments), and async runtime policy are all **Claude's discretion** — pick best practices for this project's needs

### Storage fallback behavior
- **Status bar indicator** shows current storage backend — visible in the TUI status bar
- When `storage = "auto"` and libsql fails, fall back transparently with the status bar reflecting the active backend
- When libsql becomes available after running on JSON fallback, **offer migration** — prompt user to migrate data, keep JSON as backup until confirmed
- Explicit `storage = "libsql"` failure behavior and status bar display conditions (always vs fallback-only) are **Claude's discretion**
- JSON storage location relative to libsql is **Claude's discretion** — pick what simplifies the StorageBackend abstraction

### Claude's Discretion
- Directory structure within `~/.wsl-tui/` — create subdirectories as needed by each phase
- Error handling pattern (thiserror/anyhow strategy)
- Rust conventions beyond clippy (unwrap policy, visibility defaults, doc comment requirements)
- Async runtime policy (tokio vs minimal async based on dependency needs)
- Whether status bar shows storage backend always or only on fallback
- Explicit storage = "libsql" failure behavior (refuse to start vs fall back with warning)
- JSON data file location relative to libsql

</decisions>

<specifics>
## Specific Ideas

- Welcome screen should feel polished — this is the user's first impression of the app
- Fully commented config.toml should read like good documentation, not just a dump of defaults
- Status bar storage indicator aligns with the Phase 2 requirement (TUI-07) for a status bar showing active distro, state, storage indicator, and clock

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-02-21*
