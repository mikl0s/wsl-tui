# Pitfalls Research

**Domain:** Rust TUI + WSL2 management tool (Windows-only, embedded DB, plugin system, PTY)
**Researched:** 2026-02-21
**Confidence:** MEDIUM-HIGH (multiple sources verified; some WSL-specific behavior is LOW confidence due to limited Rust-specific documentation)

---

## Critical Pitfalls

### Pitfall 1: libsql Stack Overflow on Windows

**What goes wrong:**
The libsql embedded database crate causes a `STATUS_STACK_OVERFLOW` (exit code `0xc00000fd`) crash on Windows when the SQL parser executes. The crash happens silently during parsing — not on query execution — and works fine on Linux and macOS. This is a known open issue.

**Why it happens:**
The `sqlite3-parser` dependency (built with the lemon-rs parser generator) uses deep recursion during SQL parsing. Windows has a significantly smaller default thread stack size than Linux, causing overflow during recursive descent. This is not a code error — it is a platform-level incompatibility in the dependency.

**How to avoid:**
Add explicit stack size flags to `.cargo/config.toml` before any database integration work begins:

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "link-arg=/STACK:8000000"]

[target.x86_64-pc-windows-gnu]
rustflags = ["-C", "link-arg=-Wl,--stack,8000000"]
```

Verify on Windows with a minimal integration test that creates a table and inserts a row before building further on the storage layer.

**Warning signs:**
- Process exits with `0xc00000fd` or `STATUS_STACK_OVERFLOW`
- Crash occurs at startup or first DB operation
- Crash is not reproducible on Linux/Mac CI

**Phase to address:**
Foundation phase — before any feature work. Add `.cargo/config.toml` stack size override and a Windows-specific integration smoke test on day one of storage layer work.

---

### Pitfall 2: wsl.exe Output Encoding Ambiguity (UTF-16LE vs UTF-8)

**What goes wrong:**
`wsl.exe` outputs UTF-16LE by default. However, when the user has `WSL_UTF8=1` set in their environment, `wsl.exe` switches to UTF-8 output. Tools that hard-code UTF-16LE decoding silently produce garbled text (corrupted distro names, empty lists) when this variable is present. VS Code has shipped this bug to production (see microsoft/vscode#276253).

**Why it happens:**
The default output format and the environment-variable override are poorly documented. Developers test on their own machine (which may not have `WSL_UTF8` set) and don't discover the breakage until a user reports corrupted output.

**How to avoid:**
Always detect the encoding before parsing `wsl.exe` output:

1. Check `std::env::var("WSL_UTF8")` before spawning the command. If `"1"`, expect UTF-8; otherwise expect UTF-16LE.
2. Alternatively, set `WSL_UTF8=1` explicitly on the child process environment before spawning and always parse as UTF-8 — this is simpler and more robust.
3. Write integration tests that verify output parsing with both encoding paths.

**Warning signs:**
- Distro names rendered as garbage characters or empty strings
- Works on developer machine, broken for some users
- Output parsing succeeds but list is empty

**Phase to address:**
Foundation / WSL interaction layer. Address during the first `wsl.exe` command wrapper implementation, before building any UI that displays distro names.

---

### Pitfall 3: Crossterm Duplicate Key Events on Windows

**What goes wrong:**
On Windows, crossterm emits both `KeyEventKind::Press` and `KeyEventKind::Release` for every keypress. On Linux and macOS only `Press` is emitted. Without filtering, every key action fires twice — commands execute twice, navigation jumps double distance, counters increment by two. This has been independently documented in crossterm, ratatui, and tui-realm.

**Why it happens:**
This is intentional crossterm behavior — it accurately reflects Win32 console input semantics. But TUI applications designed on Linux don't encounter it, so the filter is often omitted from examples and tutorials.

**How to avoid:**
Filter in the event loop — accept this as a permanent platform requirement, not a bug to wait for a fix:

```rust
if let Event::Key(key) = event {
    if key.kind == KeyEventKind::Press {
        // handle key
    }
}
```

This is noted in PROJECT.md as a known decision (`Filter KeyEventKind::Press`). Enforce it in the event handler module's code review checklist.

**Warning signs:**
- Actions execute twice (e.g., pressing `j` moves cursor two rows)
- All keyboard tests pass on Linux CI but fail on Windows
- Any navigation appears "double-speed"

**Phase to address:**
Foundation / TUI scaffolding — add the filter in the initial event loop skeleton so it can never be omitted.

---

### Pitfall 4: No Panic Hook = Corrupted Terminal on Crash

**What goes wrong:**
When a Ratatui application panics without a properly installed panic hook, the terminal is left in raw mode with the alternate screen still active. The user's shell becomes unusable — keypresses are invisible, output is garbled, and they must open a new terminal window. This is one of the most common complaints from TUI users.

**Why it happens:**
Ratatui enters raw mode and the alternate screen during initialization. Without a panic hook, Rust's default panic handler writes the error message over the broken terminal state, making the error unreadable and leaving cleanup undone.

**How to avoid:**
Use Ratatui 0.28.1+ built-in `ratatui::init()` and `ratatui::restore()` (they install the panic hook automatically). If using `color-eyre` for error reporting, install its hook before `ratatui::init()`:

```rust
color_eyre::install()?;
let terminal = ratatui::init();
// ... app loop ...
ratatui::restore();
```

For any custom panic handler: call `crossterm::terminal::disable_raw_mode()` and `crossterm::execute!(stdout(), crossterm::terminal::LeaveAlternateScreen)` before invoking the original hook.

**Warning signs:**
- Panic leaves terminal in broken state during development
- Terminal output is invisible after running the app
- Error messages are garbled on crash

**Phase to address:**
Foundation / TUI scaffolding — first thing in `main()`, before any other initialization.

---

### Pitfall 5: Cargo Workspace Feature Unification Breaks Windows-Only Code

**What goes wrong:**
In a Cargo workspace, if any member crate requests a feature on a shared dependency, that feature is enabled for ALL crates in the workspace during a unified build. This can cause a crate that does not need a feature (e.g., `wsl-web` not needing a C-library-backed compressor) to fail compilation because the workspace-level build now requires that C library — even though `wsl-web` never uses it.

**Why it happens:**
Cargo's default resolver (v1) unifies features across the entire workspace dependency graph. This is an intentional design for build cache efficiency, but causes cross-contamination of platform-specific dependencies.

**How to avoid:**
Add `resolver = "2"` to the workspace root `Cargo.toml` immediately:

```toml
[workspace]
resolver = "2"
members = ["wsl-tui", "wsl-web", "wsl-core"]
```

Resolver 2 ensures target-specific features are not enabled when that target is not being built, which correctly isolates feature flags across binaries. Verify with `cargo tree -e features` to inspect the actual feature set per binary.

**Warning signs:**
- `wsl-web` fails to compile with errors referencing `wsl-tui`-specific C dependencies
- `cargo build -p wsl-web` succeeds but `cargo build` fails
- Mysterious `cmake` or `cc` build errors on CI for the "wrong" binary

**Phase to address:**
Foundation — workspace `Cargo.toml` setup. Set `resolver = "2"` before adding any crate-specific features.

---

### Pitfall 6: PTY/ConPTY Architecture Mismatch for Embedded Terminal

**What goes wrong:**
On Windows, an embedded terminal (PTY) for WSL distros requires ConPTY (Windows Pseudo Console API), not Unix PTY. The common approach of using the `nix` crate's `openpty` fails entirely on Windows. WinPTY is an older alternative that does not support TrueColor or some escape sequences. Getting PTY to work across Windows Terminal, ConHost, Alacritty, and WezTerm requires careful selection of the backing implementation.

**Why it happens:**
Most Rust PTY examples and crates target Unix. The ConPTY API was introduced in Windows 10 1809 and is significantly different from POSIX PTY. The WSL process itself runs Linux, but the PTY allocation must happen on the Windows side.

**How to avoid:**
Use the `pseudoterminal` crate (cross-platform, supports ConPTY on Windows and PTY on Unix) or `portable-pty` from the WezTerm project (battle-tested). Do NOT use `nix::pty` for the embedded terminal feature. Test the PTY implementation against all four supported terminals early, before the embedding view is designed.

Architectural decision: the PTY process is spawned on the Windows side calling into WSL; WSL handles the Linux process tree internally. The Windows app owns the ConPTY handle.

**Warning signs:**
- Terminal output appears with raw escape codes visible
- Color does not render correctly in some host terminals
- PTY works in Windows Terminal but breaks in ConHost
- Process spawning works but resize events are lost

**Phase to address:**
Connectivity / Connection Modes phase — allocate dedicated spike time for PTY implementation before committing to embedded terminal as a shipped feature.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Hard-code UTF-16LE for wsl.exe parsing | Simpler initial code | Breaks for WSL_UTF8=1 users; silent garbled output | Never — always detect encoding |
| Skip panic hook setup | Faster scaffolding | Every panic leaves terminal broken; bad user reputation from day one | Never |
| Use resolver v1 (default) in workspace | No action needed | Feature unification causes mysterious cross-crate compile failures | Never — set resolver = "2" immediately |
| Blocking `wsl.exe` calls on the async executor thread | Simpler code, no channel setup | Freezes the TUI during wsl.exe calls (especially `--export` which can run for minutes) | MVP only, with clear TODO to fix |
| Single-threaded event loop without async | Simpler architecture | Background tasks (monitoring, backup) block rendering | Acceptable for Phase 1 with documented limitation |
| Lua plugins with full stdlib access | Easier plugin authoring | Plugins can execute arbitrary OS commands, read files, exfiltrate data | Never — restrict stdlib from day one |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `wsl.exe --list --verbose` | Parse output as UTF-8 strings | Detect `WSL_UTF8` env var; if absent, decode as UTF-16LE; strip BOM |
| `wsl.exe --export` | Treat as fast operation; no progress | It can take minutes for large distros; run via `spawn_blocking` or `Command::spawn` with streamed stderr; show spinner |
| `wsl.exe --terminate` before export | Expect immediate stop | Use `--shutdown` (terminates all distros + VM), wait for confirmation before export; `--terminate` leaves VM running |
| libsql on Windows | Run as normal dependency | Requires `.cargo/config.toml` stack size override; add before first build |
| crossterm event loop | Process all `Event::Key` events | Filter for `KeyEventKind::Press` only to prevent double-fire on Windows |
| Axum web server localhost | Assume localhost is safe | CORS must be configured to allow only `127.0.0.1` / `::1`; any origin policy opens XSS pivot risks |
| mlua Lua plugins | Pass full Lua stdlib | Remove `os`, `io`, `package` libraries; only expose the plugin API surface you define |
| wslapi crate | Use for distro registration | wslapi's `RegisterDistribution` is limited (one registration per executable); use `wsl.exe --import` instead |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Blocking `wsl.exe` call on render thread | TUI freezes for 0.5–60s | Use `tokio::task::spawn_blocking` + channel to deliver result to event loop | Every invocation; worst during `--export` |
| Polling wsl.exe for resource monitoring every render frame | High CPU, unnecessary process spawning | Use a background tick task at 1–5s interval; send metric updates via mpsc channel | Immediate; renders at 60fps = 60 wsl calls/sec |
| Re-creating Lua VM per plugin invocation | High latency for plugin execution | Create VM once at startup; reuse across invocations; reload scripts on file change only | Every plugin call in hot path |
| Rendering large log views without virtualization | UI freezes scrolling 10k+ log lines | Use `ratatui::widgets::List` with only the visible window; maintain a ring buffer for logs | At ~1000+ log entries |
| libsql opened on every request (wsl-web) | DB connection overhead per API call | Use a single connection wrapped in `Arc<Mutex<Connection>>` or connection pool | Under any load; immediate |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Lua plugins with `os.execute` / `io.open` available | Plugin can execute arbitrary Windows commands, read/write files outside intended scope | Remove `os`, `io`, `package`, `debug` libraries on Lua state init; expose only an explicit plugin API |
| Axum web server binding to `0.0.0.0` | Other machines on LAN can reach the management API | Bind exclusively to `127.0.0.1`; add middleware to reject non-localhost requests |
| No CORS restriction on Axum API | Browser-based XSS on any tab can call the management API | Explicitly allowlist only `http://localhost` and `http://127.0.0.1` origins in tower-http CorsLayer |
| Storing distro credentials / SSH keys in libsql without encryption | Credentials at rest in plaintext SQLite file | Do not store secrets in the DB; use Windows Credential Manager via the `keyring` crate |
| Running untrusted TOML provisioning packs without review | Pack can embed shell commands that run as the WSL user | Show all shell commands in dry-run mode before execution; consider pack signing for future milestone |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No minimum terminal size guard | Layout panics or renders as garbage below ~80x24 | Detect resize events; if terminal < minimum, render a "Terminal too small" overlay instead of the layout |
| wsl.exe operations with no feedback | User sees frozen TUI during long `--export` (can be 1–5+ minutes) | Show animated spinner + elapsed timer for all wsl.exe operations; use streamed output for progress where available |
| Panic leaving terminal broken | User must kill terminal window; first impression destroyed | Install panic hook in `main()` before anything else; verified by Ratatui 0.28.1+ `ratatui::init()` |
| Catppuccin colors in 256-color fallback mode | Colors degrade to nearest 256-color approximation; theme looks wrong | Detect truecolor support (`COLORTERM` env var = `truecolor` or `24bit`); show warning if truecolor absent; test in ConHost which may lack truecolor |
| Plugin API breaking changes | User plugins silently stop working on upgrade | Version the plugin API from day one; expose `API_VERSION` constant in Lua sandbox; fail loudly on version mismatch |

---

## "Looks Done But Isn't" Checklist

- [ ] **wsl.exe output parsing:** Verified with `WSL_UTF8=1` set in environment — not just default encoding
- [ ] **Key event handling:** Verified that pressing a key triggers action exactly once in Windows Terminal and ConHost
- [ ] **Panic recovery:** Verified that a deliberate `panic!()` in the app loop restores the terminal (shell is usable after the crash)
- [ ] **libsql on Windows:** Verified with `.cargo/config.toml` stack size override; first SQL operation does not stack overflow
- [ ] **`wsl.exe --export` for 5GB+ distro:** Verified UI remains responsive during export (background task, spinner visible)
- [ ] **Embedded terminal (PTY):** Verified in Windows Terminal, ConHost, Alacritty, and WezTerm — not just one terminal emulator
- [ ] **Lua plugin restriction:** Verified `os.execute("calc.exe")` from a plugin raises an error, not executes
- [ ] **Terminal resize:** Verified layout redraws correctly when terminal is resized to minimum and back
- [ ] **Web UI CORS:** Verified the REST API rejects requests from `http://evil.example.com` origin
- [ ] **Workspace feature unification:** Verified `cargo tree -e features` shows no unexpected features on either binary after adding a new dependency

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| libsql stack overflow discovered late | LOW | Add `.cargo/config.toml` flags; rebuild; no code changes needed |
| UTF-16LE hard-coded throughout wsl-core | HIGH | Refactor all wsl.exe output consumers to accept `Vec<u8>` + encoding hint; add encoding detection layer; audit all call sites |
| No panic hook shipped to users | MEDIUM | Ship hotfix with `ratatui::init()` / `ratatui::restore()` upgrade; communicates poorly as "fixes crash behavior" |
| Workspace resolver v1 causes build failures | LOW-MEDIUM | Add `resolver = "2"` to workspace Cargo.toml; rebuild; may require resolving newly exposed feature conflicts |
| PTY implementation wrong (WinPTY instead of ConPTY) | HIGH | Complete rewrite of connection module's embedded terminal backend; ConPTY and WinPTY have different APIs |
| Lua plugins have unintended OS access | HIGH | Emergency release to remove stdlib; existing plugins may rely on `os.execute`; requires plugin API redesign and user communication |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| libsql stack overflow on Windows | Phase 1: Foundation / Storage | Smoke test: create table + insert + query on Windows; exits cleanly |
| wsl.exe UTF-16/UTF-8 encoding ambiguity | Phase 1: Foundation / WSL Layer | Integration test: parse output with `WSL_UTF8=1` set and unset |
| Crossterm duplicate key events | Phase 1: Foundation / TUI Scaffold | Input test: press each nav key once; verify single action fires |
| No panic hook | Phase 1: Foundation / TUI Scaffold | Manual test: trigger `panic!()` in loop; verify terminal usable after |
| Cargo workspace feature unification | Phase 1: Foundation / Workspace Setup | `cargo tree -e features` shows no unexpected cross-contamination |
| PTY/ConPTY architecture mismatch | Phase 3: Connectivity / Connection Modes | PTY smoke test in Windows Terminal, ConHost, Alacritty, WezTerm |
| Long-running wsl.exe blocking render thread | Phase 2: Core Features / Backup + Monitor | Verify TUI stays responsive during 60s `--export` operation |
| Lua plugin stdlib exposure | Phase 4: Extensibility / Plugin System | Security test: plugin attempts `os.execute("calc.exe")` — must fail |
| Terminal too small causing panic | Phase 1: Foundation / TUI Scaffold | Resize terminal to 20x10; verify "too small" overlay, not crash |
| Catppuccin truecolor in 256-color terminals | Phase 1: Foundation / Theme | Run in ConHost and Windows Terminal; compare visual fidelity |
| Axum CORS misconfiguration | Phase 5: Web UI | Curl with `Origin: http://evil.com` header; verify 403 or no ACAO header |

---

## Sources

- [libsql + tokio Windows stack overflow (GitHub issue #1051)](https://github.com/tursodatabase/libsql/issues/1051) — MEDIUM confidence (issue documented, workaround verified)
- [VSCode WSL_UTF8 encoding bug (microsoft/vscode #276253)](https://github.com/microsoft/vscode/issues/276253) — HIGH confidence (official Microsoft repo, fixed in PR #276517)
- [Duplicate Key Events on Windows (ratatui/ratatui #347)](https://github.com/ratatui/ratatui/issues/347) — HIGH confidence (official ratatui repo)
- [Crossterm duplicate key events (veeso/tui-realm #54)](https://github.com/veeso/tui-realm/issues/54) — HIGH confidence (multiple independent reproductions)
- [Ratatui FAQ — crossterm version conflicts, double draw calls](https://ratatui.rs/faq/) — HIGH confidence (official documentation)
- [Ratatui Panic Hooks recipe](https://ratatui.rs/recipes/apps/panic-hooks/) — HIGH confidence (official documentation)
- [Cargo Workspace Feature Unification Pitfall (nickb.dev)](https://nickb.dev/blog/cargo-workspace-and-the-feature-unification-pitfall/) — MEDIUM confidence (verified against official Cargo docs)
- [Cargo Workspaces resolver = "2" (The Cargo Book)](https://doc.rust-lang.org/cargo/reference/workspaces.html) — HIGH confidence (official Rust documentation)
- [wsl.exe --export no progress feedback (microsoft/WSL #5161)](https://github.com/microsoft/WSL/issues/5161) — HIGH confidence (official Microsoft repo)
- [wsl.exe --terminate vs --shutdown for export](https://www.mslinn.com/wsl/3000-wsl-backup.html) — MEDIUM confidence (community-verified pattern)
- [mlua security sandbox limitations](https://users.rust-lang.org/t/lua-sandbox-in-rust/67321) — MEDIUM confidence (community discussion)
- [Luau sandbox documentation (official)](https://luau.org/sandbox/) — HIGH confidence (official Luau docs)
- [pseudoterminal crate (ConPTY cross-platform)](https://github.com/michaelvanstraten/pseudoterminal) — LOW confidence (small crate, limited production use data)
- [winpty-rs crate](https://crates.io/crates/winpty-rs) — MEDIUM confidence (documented WinPTY vs ConPTY tradeoffs)
- [Common async Rust mistakes (elias.sh)](https://www.elias.sh/posts/common_mistakes_with_async_rust) — MEDIUM confidence (verified against tokio docs)
- [WSL2 does not change state from Stopped to Running (microsoft/WSL #5406)](https://github.com/microsoft/WSL/issues/5406) — MEDIUM confidence (older issue; behavior may have improved)

---
*Pitfalls research for: Rust TUI + WSL2 management tool*
*Researched: 2026-02-21*
