# Feature Research

**Domain:** WSL2 management TUI + Web companion (Rust)
**Researched:** 2026-02-21
**Confidence:** HIGH (backed by competitor analysis, official WSL docs, and TUI ecosystem study)

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete or broken.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| List all installed distros with status | Every WSL management tool shows this; users need it before anything else | LOW | `wsl --list --verbose` wraps cleanly; show Running/Stopped, WSL version, disk size |
| Start / stop / terminate individual distros | Core lifecycle; every competitor has this | LOW | Three distinct commands: `--exec`, `--terminate`, forced shutdown |
| Set default distro | WSL has native default concept; users expect GUI control | LOW | `wsl --set-default` |
| Install new distro from available online list | First action new users take; competitors all support this | MEDIUM | `wsl --list --online` + `wsl --install -d`; show progress stream |
| Remove (unregister) distro with confirmation | Destructive operation needs confirmation; every tool has this | LOW | `wsl --unregister`; guard with modal |
| Export distro to .tar file | Built into `wsl.exe`; users expect GUI wrapper | MEDIUM | `wsl --export`; must stop distro first; show file path picker |
| Import distro from .tar file | Paired with export; expected | MEDIUM | `wsl --import`; requires destination dir |
| Shell attach / connect to distro | The #1 reason someone opens a WSL manager | LOW | Suspend TUI, run `wsl -d <name>`, restore on exit |
| Status bar with active distro and state | Standard in every dashboard TUI (k9s, lazygit, btop) | LOW | Bottom bar: distro name, state, clock, storage indicator |
| Help overlay showing keybindings | lazygit, k9s, btop all have `?` to show help | LOW | Modal overlay; context-aware per active view |
| Vim-style navigation (hjkl + arrows) | Expected by TUI-literate users; lazygit set the bar | LOW | h/j/k/l for nav, / for search, q for back/quit |
| Responsive layout (adapts to terminal size) | TUIs must work in any terminal width; k9s, lazygit do this | MEDIUM | Ratatui constraints-based layout; min-width for side panels |
| Configurable keybindings | Power users customize; taskwarrior-tui, atuin all support this | LOW | TOML-based keybind overrides in `config.toml` |
| Basic settings view | Users expect to configure defaults without editing TOML manually | MEDIUM | TUI settings panel editing `~/.wsl-tui/config.toml` |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but where wsl-tui wins.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Stackable pack provisioning system | No competitor offers automated, idempotent, GUI-driven provisioning — this is the product's core value | HIGH | TOML packs, topological dependency resolution, per-step idempotency, resume-on-failure |
| Interactive provisioning wizard | Guided variable prompts per pack (which shell? which Node version?) make packs accessible | MEDIUM | Modal wizard with field-per-variable UI; persist answers in storage |
| Dry-run mode for packs | Lets users see exactly what will change before committing — reduces fear of automation | MEDIUM | Plan output listing each step; mark skipped/new/changed |
| 9 built-in packs (home-setup, claude-code, nvm-node, python-dev, rust-dev, docker, ai-stack, gui-desktop, server-base) | Opinionated packs for common dev workflows mean users get value immediately without writing TOML | HIGH | Each pack is a curated sequence of idempotent steps |
| Custom TOML pack authoring and sharing | Teams share reproducible dev environments as plain text files — solves "works on my machine" | MEDIUM | Schema-documented TOML format; import from local path or URL |
| Resource monitoring dashboard with gauges and charts | Competitor tools (wsl2-distro-manager) have zero monitoring; this fills a real gap | HIGH | Per-distro CPU/memory/disk; real-time via polling within WSL; Ratatui sparklines + gauges |
| Embedded terminal pane (PTY) | Users can stay in the TUI while running commands — kills the need to alt-tab | HIGH | `portable-pty` / ConPTY on Windows; split-pane layout in TUI |
| Termius connection mode with SSH auto-setup | Termius is popular among homelab users; launching directly from WSL manager is a frictionless workflow | MEDIUM | SSH server provisioning inside distro + port management + Termius deep link |
| Named snapshots with timestamps | Export is table stakes; named snapshots with history tracking is significantly better UX | MEDIUM | Named snapshot + metadata (size, date, description) stored in libsql |
| Lua plugin runtime | Neovim proved Lua extensibility drives community adoption of TUI tools | HIGH | mlua crate; sandboxed env; permission model: declare + approve |
| Command palette (`:` prefix) | Modern TUI pattern popularized by VS Code; ccboard-tui shows this works in Ratatui | MEDIUM | Fuzzy-matched command list; triggered by `:` or configurable key |
| Fuzzy search / filter across distros | k9s `/-f` fuzzy filter and lazygit `/` filter are standard; users expect this | LOW | Levenshtein-distance fuzzy on distro names and pack names |
| Web UI companion (wsl-web binary) | Axum REST API + embedded SPA; manage WSL from browser or from CI/CD scripts | HIGH | Shares wsl-core; REST mirrors TUI functionality |
| Resource history logging | Queryable historical resource data enables trend analysis — no competitor does this | MEDIUM | Store metrics in libsql with timestamp; surface in monitor view |
| Re-provision existing distros | Users add packs to running distros, not just new installs — ongoing value | LOW | Same pack engine; UI shows installed vs available packs per distro |
| WSL kernel update from TUI | Power users want control over WSL kernel (`wsl --update`) without dropping to CLI | LOW | Shell out to `wsl --update`; show changelog if accessible |
| External terminal launch with configurable command template | Windows Terminal, Alacritty, WezTerm users each have a preferred terminal | LOW | Template: `wt.exe -p "Ubuntu" wsl -d {distro_name}` |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| WSL1 management | Some users still have WSL1 distros | WSL1 has different architecture (no real VM), different resource model, and different wsl.exe flags; supporting both doubles edge-case handling with <1% of target users | Document that WSL2 is required; scope note in README |
| Remote WSL management (other machines) | Power users want to manage WSL on multiple Windows machines | Requires network layer, auth system, encrypted communication — triples scope; core value is local management | Out of scope v1; REST API from wsl-web enables scripted multi-machine workflows eventually |
| Pack marketplace / central registry | Users want to discover community packs | Registry needs hosting, versioning, trust model, CDN, moderation — massive scope; adds a persistent infrastructure dependency | Support importing packs by URL directly; community can use GitHub repos as registries |
| Auto-update mechanism | Users want always-current binary | Requires signed update server, delta-update logic, background services — significant platform-specific work | Use winget + GitHub releases; winget handles updates cleanly |
| Windows Store distribution | Broad discoverability | Store submission requires sandboxing that conflicts with wsl.exe shell-out and PTY usage | GitHub releases + winget; power users use winget |
| Real-time everything (sub-100ms polling) | "Live" feel | WSL resource data requires exec inside each distro (`cat /proc/meminfo` etc); very frequent polling raises CPU overhead and can destabilize slow distros | 1-second polling with configurable interval; smooth sparkline interpolation hides gaps |
| GUI window (native Windows UI) | Non-TUI users might expect GUI | Defeats the purpose; requires Tauri/Flutter/WPF — different language, different team, different deployment | Web UI companion (wsl-web) covers browser-based GUI use case |
| Interactive shell inside embedded terminal running as root | Convenience shortcut | Security risk — many provisioning packs already run as root; encouraging root shell habit is dangerous | Require explicit sudo; default to user shell |
| WASM plugin runtime in v1 | Maximum extensibility | WASM sandboxing, ABI design, and cross-compilation complexity before Lua API is stable wastes design budget | Lua Phase 1; WASM Phase 2 after API stabilizes |
| Distro cloning (duplicate an existing distro) | Templating use case | Not a native WSL operation; requires export + import + rename + wsl.exe path manipulation; easy to corrupt if interrupted | Named snapshots + import-as-new-distro achieves the same outcome with two explicit steps |

---

## Feature Dependencies

```
[Distro List / State]
    └──required-by──> [Start / Stop / Terminate]
    └──required-by──> [Shell Attach]
    └──required-by──> [Resource Monitoring]
    └──required-by──> [Backup / Export]
    └──required-by──> [Pack Provisioning]
    └──required-by──> [Named Snapshots]
    └──required-by──> [Embedded Terminal (PTY)]
    └──required-by──> [Termius Connection Mode]

[Storage Backend (libsql / JSON)]
    └──required-by──> [Pack State / Idempotency Tracking]
    └──required-by──> [Resource History Logging]
    └──required-by──> [Named Snapshots Metadata]
    └──required-by──> [Plugin Permission Store]
    └──required-by──> [Settings Store]

[Pack Engine (TOML parser + dependency resolver)]
    └──required-by──> [Built-in Packs]
    └──required-by──> [Custom Pack Authoring]
    └──required-by──> [Interactive Provisioning Wizard]
    └──required-by──> [Dry-Run Mode]
    └──required-by──> [Re-provision Existing Distros]

[Shell Attach]
    └──enhances──> [Embedded Terminal (PTY)]
    (shell attach is the fallback; PTY is the rich version)

[Resource Monitoring]
    └──enhances──> [Resource History Logging]
    (logging requires monitoring data pipeline)

[Lua Plugin Runtime]
    └──requires──> [Plugin Registry / Lifecycle]
    └──requires──> [Permission Model]

[Termius Connection Mode]
    └──requires──> [SSH Server Provisioning Step]
    (needs to install/configure SSH inside the distro first)

[Web UI (wsl-web)]
    └──requires──> [wsl-core library] (all core logic)
    └──requires──> [REST API layer]
    └──required-by──> nothing in TUI (orthogonal binary)
```

### Dependency Notes

- **Distro List required by everything:** The WSL distro enumeration is the root dependency. Nothing else can work until the app can reliably list distros with their current state.
- **Storage Backend required by state features:** Any feature that persists state across sessions (pack tracking, snapshots, history, plugin perms) requires the storage layer. Must be built before provisioning.
- **Pack Engine required by provisioning features:** The TOML parser and dependency resolver must be built before any pack-related UI. The wizard, dry-run, and re-provision UI all sit on top of the engine.
- **Shell Attach enhances Embedded Terminal:** Shell attach (suspend TUI, exec wsl) is a simpler implementation that should ship first. Embedded PTY is the rich version that can be deferred.
- **Resource Monitoring required before Resource History:** Monitoring creates the data; history logging stores it. Must be sequential.
- **Lua Plugin Runtime is independent:** Can be built after all other features are stable without blocking anything.
- **Termius requires SSH setup:** The Termius connection mode must first provision SSH inside the distro (`sshd` install + port config). This is itself a pack step.

---

## MVP Definition

### Launch With (v0.1 — Core Management)

Minimum viable for developer adoption and initial GitHub traction.

- [x] Distro list with Running/Stopped/WSL version/disk status — without this, there's no product
- [x] Start, stop, terminate, set default — lifecycle management
- [x] Install from online list with progress stream — first-run experience
- [x] Remove with confirmation modal — destructive ops need guards
- [x] Shell attach (suspend TUI, launch wsl, restore) — primary connection need
- [x] Dashboard view with distro list and status bar — minimum polished UI
- [x] Vim-style navigation, help overlay (`?`), fuzzy filter (`/`) — table stakes UX
- [x] Catppuccin Mocha theme applied throughout — brand identity from day one
- [x] `config.toml` with configurable keybindings — foundation for settings

### Add After Validation (v0.2 — Provisioning)

Core differentiator. Adds the pack system that makes the product uniquely valuable.

- [ ] Pack engine (TOML parser, dependency resolver, idempotency) — trigger: v0.1 installs confirmed working
- [ ] Interactive provisioning wizard — trigger: pack engine complete
- [ ] Dry-run mode — trigger: pack engine complete
- [ ] All 9 built-in packs — trigger: wizard complete
- [ ] Re-provision existing distros — trigger: packs working
- [ ] Custom TOML pack import — trigger: pack engine complete

### Add After Validation (v0.3 — Connectivity)

Full connection mode matrix.

- [ ] Embedded terminal pane (PTY / ConPTY) — trigger: shell attach validated; significant complexity
- [ ] External terminal launch with configurable template — trigger: shell attach working
- [ ] Termius connection mode — trigger: external terminal working; needs SSH provisioning pack

### Add After Validation (v0.4 — Operational)

Features for homelab admins and power users.

- [ ] Resource monitoring dashboard (CPU/memory/disk gauges + sparklines) — trigger: provisioning validated
- [ ] Resource history logging — trigger: monitoring working
- [ ] Export to .tar with file picker — trigger: monitoring working
- [ ] Named snapshots with metadata — trigger: storage proven

### Future Consideration (v0.5 — Extensibility)

- [ ] Lua plugin runtime (mlua + sandboxing + permission model) — defer until API is stable
- [ ] Command palette (`:` prefix with fuzzy match) — defer until core UX settled
- [ ] Settings view (TUI-based config editor) — defer; config.toml works for power users

### Future Consideration (v1.x — Web)

- [ ] wsl-web binary (Axum REST API + embedded SPA) — defer until TUI stable
- [ ] WASM plugin runtime — defer until Lua API stabilized and documented
- [ ] Pack import by URL — defer; local file import covers v1 use case

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Distro list + state | HIGH | LOW | P1 |
| Start / stop / terminate | HIGH | LOW | P1 |
| Install from online list | HIGH | MEDIUM | P1 |
| Shell attach | HIGH | LOW | P1 |
| Dashboard UI + nav | HIGH | MEDIUM | P1 |
| Help overlay + vim nav | HIGH | LOW | P1 |
| Pack provisioning engine | HIGH | HIGH | P1 |
| Interactive wizard | HIGH | MEDIUM | P1 |
| Built-in packs (9) | HIGH | HIGH | P1 |
| Export / import .tar | HIGH | MEDIUM | P1 |
| Resource monitoring | MEDIUM | HIGH | P2 |
| Named snapshots | MEDIUM | MEDIUM | P2 |
| Embedded terminal PTY | MEDIUM | HIGH | P2 |
| Termius connection | MEDIUM | MEDIUM | P2 |
| External terminal launch | MEDIUM | LOW | P2 |
| Resource history | MEDIUM | MEDIUM | P2 |
| Custom TOML packs | MEDIUM | LOW | P2 |
| Dry-run mode | MEDIUM | MEDIUM | P2 |
| Lua plugin runtime | LOW | HIGH | P3 |
| Command palette | LOW | MEDIUM | P3 |
| Settings view (TUI) | LOW | MEDIUM | P3 |
| wsl-web binary | LOW | HIGH | P3 |
| WASM plugin runtime | LOW | HIGH | P3 |
| WSL kernel update | LOW | LOW | P2 |

**Priority key:**
- P1: Must have for launch (v0.1–v0.2)
- P2: Should have, add after core validated (v0.3–v0.4)
- P3: Nice to have, future consideration (v0.5+)

---

## Competitor Feature Analysis

| Feature | wsl2-distro-manager (Flutter) | wsl-gui-tool (Pascal/Delphi) | wsl-tui (this project) |
|---------|-------------------------------|-------------------------------|------------------------|
| Distro list / status | Yes | Yes | Yes |
| Start / stop / terminate | Yes | Yes (start/stop only) | Yes |
| Install from online list | Via Docker images / LXC | No | Yes (wsl --list --online) |
| Remove distro | Yes | Yes | Yes |
| Export / import | Yes (mentioned) | Yes | Yes |
| Named snapshots | No | No | Yes (differentiator) |
| Provisioning packs | No | No | Yes (core differentiator) |
| Interactive setup wizard | No | No | Yes |
| Dry-run mode | No | No | Yes |
| Resource monitoring | No | No | Yes (differentiator) |
| Resource history | No | No | Yes (differentiator) |
| Shell attach | No | No | Yes |
| Embedded terminal (PTY) | No | No | Yes (differentiator) |
| External terminal launch | Implied | No | Yes |
| Termius integration | No | No | Yes (differentiator) |
| Plugin system | No | No | Yes (Lua, differentiator) |
| Fuzzy search / filter | No | No | Yes |
| Command palette | No | No | Yes |
| Help overlay | No | No | Yes |
| Vim keybindings | No | No | Yes |
| Configurable keybindings | No | No | Yes |
| Web UI companion | No | No | Yes (v1.x) |
| Technology | Flutter / Dart | Object Pascal | Rust TUI |
| Deployment | GitHub releases | GitHub releases | GitHub releases + winget |

**Gap summary:** Both competitors are basic lifecycle managers. Neither has provisioning, monitoring, snapshots, plugin systems, or polished TUI UX. The entire differentiator column is open for wsl-tui.

---

## Sources

- [wsl2-distro-manager GitHub](https://github.com/bostrot/wsl2-distro-manager) — competitor feature inventory (MEDIUM confidence — GitHub README)
- [wsl-gui-tool GitHub](https://github.com/emeric-martineau/wsl-gui-tool) — competitor feature inventory (MEDIUM confidence — GitHub README)
- [Microsoft Learn: Basic WSL Commands](https://learn.microsoft.com/en-us/windows/wsl/basic-commands) — authoritative wsl.exe command list (HIGH confidence — official docs)
- [Microsoft Learn: Advanced WSL Settings](https://learn.microsoft.com/en-us/windows/wsl/wsl-config) — .wslconfig and resource limits (HIGH confidence — official docs)
- [lazygit GitHub](https://github.com/jesseduffield/lazygit) — TUI navigation patterns, panel layout, vim keybindings (HIGH confidence — official source)
- [k9s official site](https://k9scli.io/) — fuzzy search, keybinding help, resource monitoring patterns (HIGH confidence — official)
- [btop GitHub](https://github.com/aristocratos/btop) — resource monitoring dashboard design patterns (HIGH confidence — official source)
- [awesome-ratatui GitHub](https://github.com/ratatui/awesome-ratatui) — Ratatui TUI ecosystem features (MEDIUM confidence — curated list)
- [Termius WSL documentation](https://www.sabbirz.com/blog/configure-wsl-inside-termius) — Termius + WSL integration (MEDIUM confidence — community blog)
- WSL backup/restore: [XDA developers](https://www.xda-developers.com/how-back-up-restore-wsl/), [Ubuntu WSL docs](https://documentation.ubuntu.com/wsl/stable/howto/backup-and-restore/) (HIGH confidence — multiple official/authoritative sources)
- [WSL Find and Install Distributions - Medium 2026](https://medium.com/@pachoyan/windows-wsl-windows-subsystem-linux-find-and-install-different-distributions-bf3e64d54e43) — online distro install patterns (MEDIUM confidence — community)
- [ccboard-tui command palette pattern](https://lib.rs/crates/ccboard-tui) — Ratatui command palette UX (MEDIUM confidence — lib.rs)

---
*Feature research for: WSL2 TUI management tool (Rust / Ratatui)*
*Researched: 2026-02-21*
