# WSL Web — Agent Developer Kickoff Prompt

Copy this prompt to start a new session with an AI coding agent to implement the Web UI binary.

---

## Kickoff Prompt

```
You are building WSL Web — the web UI companion to WSL TUI. This is a Rust binary in the same Cargo workspace monorepo that serves a web-based interface for managing WSL2 on Windows 11.

## Project Context

This is the `wsl-web` crate in the WSL TUI monorepo. The `wsl-core` shared library and all plugin crates already exist. Your job is to build the web frontend that consumes them.

Read these docs before writing any code:
- docs/plans/2026-02-21-wsl-tui-design.md — Full architecture and design
- docs/PRD.md — Product requirements
- docs/THEME_GUIDELINES.md — Catppuccin Mocha theme (section 5 covers web adaptation)

## What You're Building

An Axum-based web server (`wsl-web` binary) that:
1. Exposes a REST API mirroring all plugin commands from wsl-core
2. Serves an embedded SPA frontend (single-page app bundled into the binary)
3. Provides the same functionality as the TUI but in a browser
4. Runs on `127.0.0.1:3000` by default (local only, no auth for v1)

## Architecture

```
crates/wsl-web/
├── src/
│   ├── main.rs              # Server startup, config, plugin registration
│   ├── api/
│   │   ├── mod.rs
│   │   ├── distros.rs       # GET/POST/DELETE /api/distros
│   │   ├── provision.rs     # POST /api/provision, GET /api/packs
│   │   ├── monitor.rs       # GET /api/monitor (SSE for real-time)
│   │   ├── backup.rs        # POST /api/backup, /api/restore
│   │   └── connect.rs       # POST /api/connect
│   ├── state.rs             # Shared app state (plugin registry, storage)
│   └── static/              # Embedded SPA assets
├── frontend/                # SPA source (if using a JS framework)
│   ├── src/
│   ├── index.html
│   └── package.json
└── Cargo.toml
```

## Key Technical Decisions

- **Web framework:** Axum 0.8+ with tower-http middleware (CORS, compression, static files)
- **Async runtime:** Tokio (shared with wsl-core)
- **Static serving:** `tower-http::services::ServeDir` or `rust-embed` for embedding assets
- **Real-time updates:** Server-Sent Events (SSE) for monitoring data and provisioning progress
- **SPA framework options** (pick one):
  - **Leptos** (Rust WASM) — if staying pure Rust
  - **SolidJS/Svelte** — if preferring a lightweight JS framework
  - **Static HTML + HTMX** — if preferring server-driven UI
- **Theme:** Catppuccin Mocha via CSS custom properties (see THEME_GUIDELINES.md section 5)
- **Font:** JetBrains Mono for data, system sans-serif for UI

## REST API Design

```
GET    /api/distros              — List all distros
POST   /api/distros/install      — Install a new distro
POST   /api/distros/:name/start  — Start a distro
POST   /api/distros/:name/stop   — Stop a distro
DELETE /api/distros/:name        — Remove a distro
POST   /api/distros/:name/default — Set as default

GET    /api/packs                — List available packs
POST   /api/provision            — Apply packs to a distro
POST   /api/provision/plan       — Dry-run a provisioning plan
GET    /api/provision/status     — SSE stream of provisioning progress

GET    /api/monitor              — SSE stream of resource metrics
GET    /api/monitor/history      — Historical metrics

POST   /api/backup/export        — Export a distro
POST   /api/backup/import        — Import a distro
GET    /api/backup/snapshots     — List snapshots

POST   /api/connect              — Initiate connection (returns connection info)

GET    /api/config               — Get current config
PUT    /api/config               — Update config
GET    /api/plugins              — List plugins
```

## Reuse from wsl-core

Everything is already implemented in wsl-core and the plugin crates:
- `StorageBackend` — use the same storage as TUI
- `Plugin` trait — same plugins, same commands
- `Pack` / `ProvisionStep` — same provisioning engine
- `WslExecutor` — same WSL command execution

Your job is to wire Axum routes to these existing interfaces and build the web frontend.

## Quality Standards

- All API endpoints return JSON with consistent error format: `{ "error": "message" }`
- SSE endpoints handle client disconnection gracefully
- Static assets gzip-compressed via tower-http
- CORS configured for local development
- API integration tests

Start by setting up the Axum server, registering plugins from wsl-core, and implementing the distro API endpoints. Then build the SPA frontend.
```
