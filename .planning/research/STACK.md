# Stack Research

**Domain:** Rust TUI + Web UI for WSL2 management on Windows 11
**Researched:** 2026-02-21
**Confidence:** HIGH (core stack), MEDIUM (supporting libs), LOW (flagged items)

---

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | stable 1.86+ | Language | MSRV required by Ratatui 0.30; Rust 2024 edition gives improved ergonomics and lifetime rules |
| ratatui | 0.30.0 | TUI framework | Largest active fork of tui-rs, 0.30 is the biggest release yet — modular workspace, no_std, improved layout API, Catppuccin integration via feature flag |
| crossterm | 0.29.0 | Terminal backend | Default ratatui backend; cross-platform Windows support including ConHost, Windows Terminal, Alacritty, WezTerm; pure-Rust, 73.7M downloads |
| axum | 0.8.8 | Web framework | Official Tokio-project web framework; 0.8 released Jan 2025; ergonomic extractors, first-class tower integration, WebSocket support built-in; latest stable is 0.8.8 |
| tokio | 1.x (≥1.43 LTS) | Async runtime | De facto standard; powers axum; 1.43.x is current LTS (until March 2026), 1.47.x LTS until Sept 2026; use `full` feature for TUI event loops |
| serde | 1.0.228 | Serialization | Required by every major crate in this stack; use `derive` feature |
| serde_json | 1.0.149 | JSON backend | JSON fallback storage; config interchange; REST API payloads |

### WSL2 Integration

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Shell-out to wsl.exe | N/A (system) | Primary WSL interaction | Most reliable, exposes all wsl.exe subcommands; output is UTF-16LE on Windows, must parse explicitly |
| wslapi | 0.1.3 | Native WSL API bindings | Supplemental: `WslGetDistributionConfiguration`, `WslIsDistributionRegistered`, `WslLaunch`; wraps wslapi.dll; version is old (2021) — use only for APIs not reachable via wsl.exe |
| windows | 0.58+ | Windows API bindings | Microsoft-maintained; needed for registry reads, process handles, ConPTY if not using portable-pty |

**Note on wslapi crate confidence: LOW.** Version 0.1.3 has not seen active updates. Evaluate at implementation time whether to use it vs. direct `windows` crate bindings or raw shell-out. Shell-out to `wsl.exe` is the primary path.

### Embedded Terminal (PTY)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| portable-pty | 0.8.x | Cross-platform PTY | Built and maintained by WezTerm team; supports ConPTY on Windows natively; provides `PtySystem` trait for runtime backend selection; most battle-tested Windows PTY crate |

Do NOT use `winpty-rs` — it depends on the legacy WinPTY binary which is not required on Windows 10 1903+ (ConPTY is sufficient). Do NOT use raw `CreatePseudoConsole` directly — portable-pty wraps it correctly.

### Storage

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| libsql | 0.9.29 | Embedded database (default) | Turso-maintained SQLite fork; fully embeddable, no network required; supports transparent replication if needed later; Rust API is async-native |
| serde_json | 1.0.149 | JSON fallback storage | Human-readable, zero-install fallback when libsql fails to compile or is explicitly disabled; trait-based backend makes swap transparent |

### Plugin System

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| mlua | 0.11.5 | Lua 5.4 scripting runtime | Most mature Lua bindings for Rust; supports async/await natively; serde integration for data exchange; Lua 5.4 chosen over LuaJIT for better Windows compatibility and modern feature set |

Use `mlua` with features `["lua54", "vendored", "async"]`. The `vendored` feature compiles Lua from source — no system Lua dependency required, which is critical for Windows.

### Theming

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| catppuccin | 2.4+ | Catppuccin Mocha palette | Official Rust crate; enable `ratatui` feature flag for direct `ratatui::style::Color` conversion; eliminates manual hex-to-color mapping |

### Web UI (Axum companion binary)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| rust-embed | 8.11.0 | Embed SPA files in binary | Compile-time embedding of JS/CSS/HTML assets; loads from filesystem in dev, embedded in release; 8.11.0 released 2026-01-14 |
| axum-embed | 0.1.x | Axum handler for rust-embed | Bridges rust-embed with axum's router; handles ETags, content-type, SPA fallback routing |
| tower-http | 0.6.x | HTTP middleware | CORS, compression, request tracing; official tower layer for axum; use with `full` or selective features |
| tokio-tungstenite | 0.26 | WebSocket (axum dep) | Upgraded in axum 0.8; used for real-time metric streaming to web dashboard; axum re-exports WebSocket via its extract::ws module |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| thiserror | 2.x | Custom error types | All library crates (wsl-core) — define structured error enums exposed to consumers |
| anyhow | 1.x | Application error handling | Binary crates (wsl-tui, wsl-web) — wraps any error with context, `.context()` for chain |
| tracing | 0.1.x | Structured logging | All crates; replaces log/env_logger; integrates with tokio; async-aware spans |
| tracing-subscriber | 0.3.x | Log output formatting | Binary crates; configure file appender + ANSI console; use `env_filter` feature |
| tracing-appender | 0.2.x | Non-blocking log file writes | wsl-tui binary — log to AppData file without blocking the event loop |
| clap | 4.5.x | CLI argument parsing | Both binaries; use `derive` feature for type-safe argument structs |
| toml | 0.8.x | TOML pack/config parsing | wsl-core — parse provisioning pack files and user config (Cargo uses this crate, well-maintained) |
| directories | 5.x | XDG/AppData paths | All crates; resolves `%APPDATA%` config dir on Windows, `~/.config` on Linux |
| nucleo | 0.5.x | Fuzzy matching | wsl-tui — command palette and distro filter; faster than fzf algorithm, maintained by Helix editor team |
| sysinfo | 0.37.2 | System resource monitoring | wsl-core — CPU, memory, disk per process; MSRV 1.88; Windows-native support |
| uuid | 1.x | Unique IDs | wsl-core — snapshot IDs, session tracking; use `v4` feature |
| chrono | 0.4.x | Timestamps | wsl-core — backup timestamps, log timestamps; use `serde` feature |
| tokio-util | 0.7.x | Async codec/framing | wsl-core — frame PTY output streams, pipe async reads |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo-nextest | Fast test runner | Parallel test execution; replace `cargo test` in CI |
| cargo-watch | File watcher | `cargo watch -x check` during development |
| cargo-deny | Dependency auditing | License and vulnerability checking; add to CI |
| clippy | Linting | Zero warnings policy; `#![deny(clippy::all)]` in all crates |
| rustfmt | Formatting | Enforce with `rustfmt.toml`; `edition = "2024"` |
| cross | Cross-compilation | If targeting Linux builds from Windows; likely not needed for Phase 1 |

---

## Cargo.toml Setup

```toml
# Workspace Cargo.toml
[workspace]
members = ["wsl-tui", "wsl-web", "wsl-core"]
resolver = "2"

[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# TUI
ratatui = "0.30"
crossterm = "0.29"
catppuccin = { version = "2", features = ["ratatui"] }

# Web
axum = { version = "0.8", features = ["ws", "macros"] }
tower-http = { version = "0.6", features = ["cors", "compression-gzip", "trace"] }
rust-embed = { version = "8", features = ["axum-ex"] }

# Storage
libsql = "0.9"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Errors
thiserror = "2"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"

# CLI
clap = { version = "4", features = ["derive"] }

# PTY
portable-pty = "0.8"

# Lua plugins
mlua = { version = "0.11", features = ["lua54", "vendored", "async"] }

# Utilities
directories = "5"
sysinfo = "0.37"
nucleo = "0.5"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }

[workspace.package]
edition = "2024"
license = "MIT"
authors = ["Mikkel Georgsen"]
rust-version = "1.86"
```

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| ratatui 0.30 | tui-rs | Never — tui-rs is unmaintained, ratatui is the official fork |
| crossterm | termion | Never for this project — termion is Linux-only, crossterm is the only Windows-compatible backend |
| crossterm | termwiz | Only if targeting WezTerm as sole terminal with deeper integration needed; more complex API |
| axum 0.8 | actix-web | If you need sync handlers or more battle-hardened HTTP/1.1 edge cases; axum is preferred for Tokio-native projects |
| axum 0.8 | warp | Never — warp's composable filter pattern is more complex and axum has overtaken it in community adoption |
| libsql | rusqlite | If you want zero compile-time surprise and pure SQLite with no forks; rusqlite is simpler but lacks async and replication |
| libsql | SQLite via rusqlite | Reasonable alternative if libsql compile complexity causes issues on Windows; decide at Phase 1 implementation time |
| mlua (vendored) | rlua | Never — rlua is unmaintained |
| mlua (vendored) | lua crate | Never — no async support, less maintained |
| portable-pty | raw ConPTY via windows crate | Only if you need extremely tight control over the PTY lifecycle; portable-pty wraps ConPTY correctly and is much less code |
| nucleo | fuzzy-matcher | fuzzy-matcher is fine but nucleo is faster and Helix-tested; prefer nucleo for consistent performance |
| tracing | env_logger / log | log/env_logger has no async-aware spans; tracing is the tokio-ecosystem standard |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| tui-rs | Unmaintained since 2022; ratatui is the community fork | ratatui 0.30 |
| termion | Linux-only; project targets Windows 11 | crossterm 0.29 |
| actix-web | Different async model (actix actor system); adds complexity without benefit for this use case | axum 0.8 |
| winpty-rs / WinPTY | Legacy; requires separate WinPTY binary; ConPTY (via portable-pty) works on Windows 10 1903+ | portable-pty |
| rlua | Unmaintained; no async support | mlua 0.11 |
| log + env_logger | No async spans, no structured data, not tokio-integrated | tracing + tracing-subscriber |
| std::process::Command for wsl.exe | Does not give UTF-16LE output handling; no async support | tokio::process::Command with explicit encoding |
| getrandom (directly) | Use uuid crate's v4 feature instead; getrandom quirks on Windows are handled by uuid | uuid = { features = ["v4"] } |
| warp | Abandoned in favour of axum by most of the community | axum 0.8 |

---

## Stack Patterns by Variant

**For the TUI binary (wsl-tui):**
- Main event loop: tokio::main with a separate thread for the ratatui terminal draw loop
- Use `crossterm::event::EventStream` (async) inside tokio to stream key events without blocking
- Filter `KeyEventKind::Press` only — Windows ConHost sends both Press and Release, causing double events
- Use `tokio::sync::mpsc` channels to communicate between the async WSL command layer and the sync TUI draw layer

**For the Web binary (wsl-web):**
- Single tokio::main entry; axum router with embedded SPA via rust-embed + axum-embed
- WebSocket endpoint for real-time metric streaming (tokio broadcast channel → axum ws handler)
- REST endpoints mirror wsl-tui commands via wsl-core shared functions
- Serve SPA fallback: all non-API routes return index.html for client-side routing

**For wsl-core (shared library):**
- All WSL operations are async (tokio::process::Command)
- StorageBackend trait: implement for libsql and JSON; selected at startup via config
- All public APIs return Result<T, WslError> (thiserror enum)
- Plugin API: expose Lua UserData types for Distro, Pack, BackupEntry

**For TOML provisioning packs:**
- Parse with toml crate + serde Deserialize
- Validate with custom validator (not a heavy validation library)
- Store pack definitions in `%APPDATA%\wsl-tui\packs\` using the `directories` crate

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| ratatui 0.30 | crossterm 0.29 (default) | ratatui 0.30 supports crossterm 0.28 and 0.29 via feature flags; default is 0.29 |
| ratatui 0.30 | MSRV 1.86 | Rust 2024 edition required |
| axum 0.8 | tokio 1.x | tokio-tungstenite 0.26 included; MSRV 1.80 |
| libsql 0.9.x | tokio 1.x | Async-native, compatible with tokio runtime |
| mlua 0.11 | tokio 1.x | async feature requires tokio or async-std; use `tokio` runtime |
| sysinfo 0.37 | MSRV 1.88 | Note: higher MSRV than ratatui (1.86); compile on Rust 1.88+ to satisfy both |
| portable-pty | windows-rs compatible | Uses ConPTY via windows syscalls; no conflict with windows crate |

**Critical:** The workspace MSRV must be set to **1.88** (driven by sysinfo 0.37.2), which satisfies all other crates (ratatui requires 1.86, axum requires 1.80).

---

## Sources

- [Ratatui 0.30.0 highlights](https://ratatui.rs/highlights/v030/) — confirmed version, features, crossterm compatibility (HIGH confidence)
- [Ratatui 0.30.0 release on GitHub](https://github.com/ratatui/ratatui/releases/tag/ratatui-v0.30.0) — release date December 2025 (HIGH confidence)
- [Ratatui docs.rs 0.30.0](https://docs.rs/ratatui/latest/ratatui/) — backend list, MSRV 1.86, Rust 2024 edition (HIGH confidence)
- [Axum 0.8.0 announcement](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) — released January 1 2025, path syntax change, no #[async_trait] needed (HIGH confidence)
- [Axum CHANGELOG](https://github.com/tokio-rs/axum/blob/main/axum/CHANGELOG.md) — latest stable 0.8.8, MSRV 1.80, tokio-tungstenite 0.26 (HIGH confidence)
- [crossterm 0.29.0 on crates.io](https://crates.io/crates/crossterm/0.29.0) — released April 2025, 73.7M downloads (HIGH confidence)
- [tokio releases](https://github.com/tokio-rs/tokio/releases) — 1.49.0 latest; LTS: 1.43.x until March 2026, 1.47.x until Sept 2026 (HIGH confidence)
- [libsql crates.io](https://crates.io/crates/libsql) — 0.9.29, MIT, Turso-maintained (MEDIUM confidence — version from search, not direct crates.io fetch)
- [mlua 0.11.5 docs.rs](https://docs.rs/crate/mlua/latest) — latest version 0.11.5, Lua 5.4 support, async (HIGH confidence)
- [rust-embed 8.11.0](https://docs.rs/crate/rust-embed/latest) — released 2026-01-14 (HIGH confidence)
- [sysinfo 0.37.2](https://crates.io/crates/sysinfo) — 83.9M downloads, MSRV 1.88 (MEDIUM confidence — version from search)
- [portable-pty docs.rs](https://docs.rs/portable-pty) — ConPtySystem support on Windows (MEDIUM confidence)
- [wslapi 0.1.3 docs.rs](https://docs.rs/wslapi/latest/wslapi/) — version 0.1.3, minimal maintenance indicators (LOW confidence — version is old, verify at impl time)
- [catppuccin rust crate](https://github.com/catppuccin/rust) — official crate, ratatui feature flag (MEDIUM confidence)
- [nucleo GitHub](https://github.com/helix-editor/nucleo) — Helix editor fuzzy matcher (MEDIUM confidence)
- [serde_json 1.0.149](https://crates.io/crates/serde_json) — released 2026-01-06 (HIGH confidence)
- WebSearch (2026) — tracing, anyhow, thiserror, toml, directories, clap findings (MEDIUM confidence, widely established)

---

*Stack research for: Rust TUI + Web UI for WSL2 management on Windows 11*
*Researched: 2026-02-21*
