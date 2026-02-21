# wsl-web — Agent Reference

`wsl-web` is the web binary crate for the WSL TUI workspace. It will serve a REST API and an embedded SPA for browser-based management of WSL2 distros.

**Current status: Phase 1 stub.** The binary compiles and exits immediately after printing a message. No server logic exists yet.

---

## Current State (Phase 1)

```rust
// wsl-web/src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("wsl-web: not yet implemented");
    Ok(())
}
```

The crate exists in the workspace to:
1. Verify the three-crate workspace compiles end-to-end
2. Reserve the binary name and crate structure for Phase 7
3. Ensure `cargo clippy --workspace -- -D warnings` covers all three crates from day one

---

## Planned Architecture (Phase 7)

Implementation details are tracked in the project roadmap (`.planning/ROADMAP.md` Phase 7 section).

### Planned Tech Stack

| Component | Library |
|---|---|
| HTTP framework | Axum 0.8+ |
| Middleware | tower-http (CORS, compression, tracing) |
| Embedded SPA | rust-embed (bundles SPA assets into the binary) |
| Shared logic | `wsl-core` (same as `wsl-tui`) |
| Async runtime | tokio (already in workspace) |

### Planned Endpoint Categories

- `GET /api/distros` — list WSL distros (via `WslExecutor`)
- `POST /api/distros/{name}/start` — start a distro
- `POST /api/distros/{name}/stop` — stop a distro
- `GET /api/config` — read current config
- `GET /` — serve embedded SPA

### Planned SPA

The web frontend will be compiled separately and embedded into the Rust binary via `rust-embed`. This means the binary is self-contained — no separate web server or asset directory required.

---

## Adding Phase 7 Implementation

When Phase 7 starts:

1. Add Axum, tower-http, and rust-embed to `[workspace.dependencies]` in root `Cargo.toml`.
2. Add them to `wsl-web/Cargo.toml` with `{ workspace = true }`.
3. Replace `wsl-web/src/main.rs` with the Axum router setup.
4. Create `wsl-web/src/routes/` for route handlers.
5. Create `wsl-web/src/state.rs` for shared server state (`Config`, `WslExecutor`, storage).
6. Build and embed the SPA assets using `rust-embed`.

See the kickoff document at `docs/KICKOFF_WEB.md` for the detailed Phase 7 developer brief.

---

## Cross-Crate Note

`wsl-web` depends on `wsl-core` for all business logic — the same `Config`, `WslExecutor`, `open_storage`, `Plugin`, and `PluginRegistry` types used by `wsl-tui`. Keep the two binaries independent of each other — all sharing goes through `wsl-core`.

---

## Dependencies (wsl-web, Phase 1)

| Crate | Usage |
|---|---|
| `wsl-core` | Shared library (currently unused — stub only) |
| `tokio 1 full` | Async runtime (`#[tokio::main]`) |
| `anyhow 1` | Error propagation |
