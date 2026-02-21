# Phase 2: Core Distro Management TUI - Research

**Researched:** 2026-02-21
**Domain:** Ratatui TUI layout, async event loop, WSL command execution, shell attach, theming
**Confidence:** HIGH (stack verified via official docs and ratatui.rs; WSL commands verified via Microsoft Learn)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Dashboard layout:**
- Right-side details panel: distro list on left, details for selected distro on right
- Details panel inspired by Midnight Commander / lazygit split-pane layout

**Destructive action confirmation:**
- Use y/N single-key confirmation for destructive actions (remove distro)
- Simple and fast: press 'y' to confirm, anything else cancels
- Modal popup showing what will happen, with clear "cannot be undone" warning

**Install progress:**
- Modal overlay with progress bar and step-by-step status
- Blocks the distro list until install completes (no background install)

**Shell attach:**
- Instant swap: TUI clears immediately and drops into shell, no transition message
- Auto-start: pressing Enter on a stopped distro starts it first, then attaches
- Exit shell normally: type 'exit' or Ctrl+D to return to TUI
- Full restore on return: exact scroll position, selection, and view state preserved

**Fuzzy filter:**
- Inline filter bar: pressing '/' shows a text input at the top/bottom of the list
- Distros filter as you type, Esc to clear filter

### Claude's Discretion

- Distro list format (table rows vs styled list items)
- Details panel content and action hints
- Responsive layout behavior on narrow terminals
- Install flow initiation (popup picker vs dedicated view)
- Export/import file path UX
- Status bar content and layout
- Help overlay style (full-screen vs centered modal)
- View switching feel (instant vs tab bar)

### Deferred Ideas (OUT OF SCOPE)

- Auto-start distro on Windows boot / run distro in background after reboot
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| DIST-01 | User can see all installed WSL distros with state (Running/Stopped), WSL version, and default indicator | `wsl --list --verbose` parsing; List + ListState pattern |
| DIST-02 | User can install a new distro from the available online list with progress feedback | `wsl --list --online` + `wsl --install <name>`; Gauge widget for progress; modal overlay pattern |
| DIST-03 | User can start a stopped distro | `wsl -d <name>` (no-op start); wsl start pattern via executor |
| DIST-04 | User can stop a running distro | `wsl --terminate <name>` |
| DIST-05 | User can terminate a distro (force stop) | `wsl --terminate <name>` (same as stop for WSL2) |
| DIST-06 | User can set a distro as the WSL default | `wsl --set-default <name>` |
| DIST-07 | User can remove (unregister) a distro with a confirmation prompt | `wsl --unregister <name>`; y/N modal with Clear widget |
| DIST-08 | User can export a distro to a `.tar` file | `wsl --export <name> <file>`; text input for path |
| DIST-09 | User can import a distro from a `.tar` file | `wsl --import <name> <install-dir> <file>`; text input for fields |
| DIST-10 | User can update the WSL kernel from within the TUI | `wsl --update`; progress indicator |
| CONN-01 | User can connect to a distro via shell attach (TUI suspends, drops into shell, restores on exit) | crossterm disable_raw_mode + LeaveAlternateScreen + spawn + wait + re-init pattern |
| TUI-01 | Dashboard view shows distro list, details panel, and resource monitor summary | Layout::horizontal split; List widget (left) + Paragraph/Block (right) |
| TUI-07 | Status bar showing active distro, state, storage indicator, and clock | Rect split at bottom; Paragraph with spans |
| TUI-08 | Vim-style navigation (h/j/k/l, arrows, Tab for panels) | KeyCode::Char('j'/'k'/'h'/'l') in event loop; panel focus enum |
| TUI-09 | Help overlay (?) showing context-aware keybindings per active view | Clear widget + centered popup area + Paragraph |
| TUI-10 | Fuzzy search/filter (/) across distros | Inline input bar; String filter state; List items filtered on render |
| TUI-12 | Responsive layout adapting to terminal size with min-width guards | frame.area().width checks; conditional layout branches |
| TUI-13 | Catppuccin Mocha theme applied consistently | Color::Rgb values from THEME_GUIDELINES.md; no catppuccin crate needed |
| TUI-14 | Keybindings are configurable via `config.toml` | Custom serde for KeyCode (string like "j", "ctrl+d"); Config struct extension |
| TUI-15 | Views accessible via number keys (1-5) | KeyCode::Char('1'..'5') dispatch in event loop |
</phase_requirements>

---

## Summary

Phase 2 is a significant step up from Phase 1's skeleton. The core challenge is threefold: (1) building a proper split-pane interactive TUI with keyboard navigation and multiple modal layers on top of the existing ratatui + crossterm foundation; (2) executing WSL commands asynchronously with progress feedback without freezing the UI; and (3) implementing shell attach — the most platform-sensitive operation, requiring temporary complete teardown of the terminal state.

The ratatui 0.30 API is fully verified and stable. The key widgets needed — `List` with `ListState`, `Table` with `TableState`, `Gauge`, `Clear` for popups, and `Layout::horizontal` — are all present in the version already in `Cargo.toml`. No new ratatui-version-related breaking changes affect this phase. The one structural change in 0.30 (crate split into `ratatui-core` + `ratatui-widgets`) is transparent to consumers using the `ratatui` umbrella crate.

The event loop upgrade from synchronous `event::read()` to `EventStream` + `tokio::select!` is required in Phase 2 because background async tasks (WSL command execution during install, polling for distro state changes) must not block the render loop. This was explicitly noted as deferred from Phase 1 and is now the right time. The `crossterm` dependency already has the `event-stream` feature enabled in `Cargo.toml`, and `tokio` and `futures` are already present.

**Primary recommendation:** Build the phase in three layers — (1) event loop upgrade + App state expansion, (2) the split-pane dashboard with distro list and details, (3) modal overlays (confirm, install progress, help, filter, export/import). Shell attach is a discrete, well-understood operation: call `ratatui::restore()`, spawn `wsl.exe -d <name>`, `child.wait()`, then `ratatui::init()` again.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ratatui` | 0.30 (already in workspace) | TUI rendering, widgets, layout | Workspace standard; all widgets verified available |
| `crossterm` | 0.29 (already in workspace) | Terminal events, raw mode, alternate screen | Workspace standard; `event-stream` feature already enabled |
| `tokio` | 1 full (already in workspace) | Async runtime; required for `EventStream` + `tokio::select!` | Workspace standard |
| `futures` | 0.3 (already in workspace) | `StreamExt` trait for `EventStream` | Already present; needed for `.next().await` on the event stream |
| `wsl-core` | workspace (already in workspace) | `WslExecutor` for all WSL commands | Workspace standard |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde` | 1 (already in workspace) | Serialize/deserialize keybinding config | Extending `Config` struct for TUI-14 configurable keybindings |
| `toml` | 0.8 (already in workspace) | Parse keybinding values from `config.toml` | Already used for config parsing |

**No new crate dependencies are required for Phase 2.** All needed libraries are already in the workspace.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `catppuccin` crate | Hard-coded `Color::Rgb` constants | The crate adds a dependency; the palette is small and stable; THEME_GUIDELINES.md already has all values — use a `theme.rs` module with `const` RGB tuples |
| `tui-textarea` crate | Hand-rolled input state (String + char index) | For a single-line filter bar and path inputs, the hand-rolled approach is sufficient; tui-textarea adds a dependency for marginal benefit |
| `tui-popup` crate | `Clear` + `popup_area()` helper | Official ratatui pattern is equally simple; avoid extra dependency |

**Installation:** No new `cargo add` commands needed — all dependencies already present.

---

## Architecture Patterns

### Recommended File Structure for Phase 2

```
wsl-tui/src/
├── main.rs                     — entry point (upgrade to EventStream event loop)
├── app.rs                      — App struct (expand with distro state, view, focus, filter)
├── action.rs                   — Action enum (distro commands, navigation, modal open/close)
├── theme.rs                    — Catppuccin Mocha Color::Rgb constants
├── keybindings.rs              — KeyBindings struct, serde helpers for KeyCode
└── ui/
    ├── mod.rs                  — render() dispatcher
    ├── welcome.rs              — (existing Phase 1)
    ├── dashboard.rs            — split-pane distro list + details panel
    ├── status_bar.rs           — bottom status bar
    ├── help_overlay.rs         — ? help popup
    ├── confirm_modal.rs        — y/N confirmation popup (remove, etc.)
    ├── install_modal.rs        — install progress overlay
    └── input_modal.rs          — text input modal (export path, import fields)

wsl-core/src/
├── wsl/
│   ├── mod.rs                  — re-export WslExecutor + DistroInfo
│   └── executor.rs             — extend with parse_list_verbose, list_online, install, etc.
│   └── distro.rs               — NEW: DistroInfo struct, DistroState enum
```

### Pattern 1: App State Machine with Focus and View

The `App` struct must grow substantially. Use an enum for active view and an enum for focus panel.

```rust
// wsl-tui/src/app.rs
pub enum View {
    Dashboard,
    // Provision, Monitor, etc. are Phase 3+
}

pub enum FocusPanel {
    DistroList,
    Details,
}

pub enum ModalState {
    None,
    Confirm { action: DistroAction, distro_name: String },
    InstallProgress { distro: String, step: String, percent: u16 },
    Help,
    FilterInput { query: String },
    ExportInput { distro: String, path: String, cursor: usize },
    ImportInput { /* fields */ },
}

pub struct App {
    pub running: bool,
    pub first_run: bool,
    pub current_view: View,
    pub focus: FocusPanel,

    // Distro state
    pub distros: Vec<DistroInfo>,
    pub list_state: ListState,
    pub filter: String,
    pub filtered_indices: Vec<usize>,

    // Modal layer
    pub modal: ModalState,

    // Status bar
    pub storage_backend: String,
}
```

**When to use:** Any time the rendering or event handler needs to branch on application state.

### Pattern 2: EventStream + tokio::select! Event Loop

Replace synchronous `event::read()` with `EventStream` to allow concurrent async operations (WSL installs, state polls).

```rust
// wsl-tui/src/main.rs
use crossterm::event::EventStream;
use futures::StreamExt;
use tokio::time::{Duration, interval};

async fn run_app(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> anyhow::Result<()> {
    let mut events = EventStream::new();
    let mut poll_ticker = interval(Duration::from_secs(5)); // distro state refresh

    while app.running {
        tokio::select! {
            _ = poll_ticker.tick() => {
                // Refresh distro list from wsl.exe --list --verbose
                app.refresh_distros()?;
            }
            maybe_event = events.next() => {
                let Some(Ok(event)) = maybe_event else { break };
                handle_event(app, event)?;
            }
        }
        terminal.draw(|frame| ui::render(app, frame))?;
    }
    Ok(())
}
```

**Key detail:** The `crossterm` dep already has `event-stream` feature in `Cargo.toml`. `futures::StreamExt` gives `.next()` on the stream.

### Pattern 3: Split-Pane Layout (Dashboard)

```rust
// wsl-tui/src/ui/dashboard.rs
use ratatui::layout::{Constraint, Layout, Rect};

pub fn render_dashboard(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Reserve bottom 1 row for status bar
    let [main_area, status_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
    ]).areas(area);

    // Split main area: 40% list, 60% details
    let [list_area, details_area] = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ]).areas(main_area);

    render_distro_list(app, frame, list_area);
    render_details_panel(app, frame, details_area);
    status_bar::render(app, frame, status_area);
}
```

**Responsive guard:** Check `area.width < MIN_WIDTH` and render a "terminal too narrow" message instead.

### Pattern 4: StatefulWidget List with ListState

```rust
// wsl-tui/src/ui/dashboard.rs
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::style::{Style, Modifier};

fn render_distro_list(app: &App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = app.visible_distros().iter().map(|d| {
        let state_sym = if d.state == DistroState::Running { "●" } else { "○" };
        let state_color = if d.state == DistroState::Running {
            Color::Rgb(166, 227, 161)  // Green
        } else {
            Color::Rgb(243, 139, 168)  // Red
        };
        let default_prefix = if d.is_default { "▸ " } else { "  " };
        ListItem::new(format!("{}{} {} v{}", default_prefix, state_sym, d.name, d.version))
            .style(Style::default().fg(state_color))
    }).collect();

    let border_color = if app.focus == FocusPanel::DistroList {
        Color::Rgb(203, 166, 247)  // Mauve (active)
    } else {
        Color::Rgb(69, 71, 90)     // Surface1 (inactive)
    };

    let list = List::new(items)
        .block(Block::bordered().title("Distros").border_style(Style::default().fg(border_color)))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(list, area, &mut app.list_state.clone());
}
```

**Note:** `ListState` requires mutable access during render — either clone it or store it mutably accessible from the render pass.

### Pattern 5: Modal Overlay with Clear

```rust
// wsl-tui/src/ui/confirm_modal.rs
use ratatui::widgets::Clear;
use ratatui::layout::{Constraint, Flex, Layout};

fn popup_area(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let [area] = Layout::vertical([Constraint::Percentage(height_pct)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::horizontal([Constraint::Percentage(width_pct)])
        .flex(Flex::Center)
        .areas(area);
    area
}

pub fn render_confirm(app: &App, frame: &mut Frame, distro_name: &str, action: &DistroAction) {
    let popup = popup_area(frame.area(), 60, 30);
    frame.render_widget(Clear, popup);  // erase background under popup

    let block = Block::bordered()
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Rgb(203, 166, 247)))  // Mauve
        .title(Span::styled("Confirm", Style::default().fg(Color::Rgb(243, 139, 168)).bold()));

    let text = Paragraph::new(format!(
        "Remove '{}' from WSL?\n\nAll data will be permanently lost.\n\nPress [y] to confirm, any other key to cancel.",
        distro_name
    ))
    .block(block)
    .wrap(Wrap { trim: true });

    frame.render_widget(text, popup);
}
```

### Pattern 6: Shell Attach

This is the critical pattern for CONN-01. Shell attach requires completely suspending the TUI, running a child process that inherits stdin/stdout, then fully restoring the TUI.

```rust
// wsl-tui/src/app.rs (or a separate shell.rs)
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use std::io::stdout;
use std::process::Command;

/// Attach to a WSL distro shell. Suspends the TUI, runs the shell, restores on return.
/// Returns the saved scroll position and selection for full state restore.
pub fn attach_shell(distro_name: &str) -> anyhow::Result<()> {
    // 1. Suspend TUI — must happen before spawning child
    ratatui::restore();  // disables raw mode + leaves alternate screen

    // 2. Start distro if needed (wsl -d <name> -- true is a no-op start)
    //    Then attach: inherit stdin/stdout/stderr
    let status = Command::new("wsl.exe")
        .args(["-d", distro_name])
        .status()?;

    // 3. Re-initialize TUI — restore raw mode + alternate screen + reinstall panic hook
    //    Reassign the terminal returned from ratatui::init() to the caller
    // Note: caller must re-call ratatui::init() and update its terminal handle

    let _ = status;  // shell exit code is informational, not an error
    Ok(())
}
```

**Caller pattern in main.rs:**

```rust
// In the event handler when Enter is pressed on a distro:
if app.focus == FocusPanel::DistroList {
    if let Some(distro) = app.selected_distro() {
        // If stopped, start first
        if distro.state == DistroState::Stopped {
            executor.run(&["-d", &distro.name, "--", "true"])?;
        }
        // Suspend TUI and attach
        ratatui::restore();
        let _ = Command::new("wsl.exe").args(["-d", &distro.name]).status();
        // Re-init (terminal handle must be reassigned)
        *terminal = ratatui::init();
        // State is already preserved in app.list_state (unchanged during attach)
    }
}
```

**Why this works:** `ratatui::restore()` calls `disable_raw_mode()` + `execute!(LeaveAlternateScreen)`. `ratatui::init()` re-enters raw mode + alternate screen + reinstalls the panic hook. The child process runs while the alternate screen is off, so `wsl.exe` sees a normal terminal. Scroll position and `ListState` selection are untouched during the attach period.

### Pattern 7: Fuzzy Filter (Inline)

Store a `filter: String` in `App`. On each render, derive `visible_distros()` by filtering `distros` on `name.to_lowercase().contains(&filter.to_lowercase())`. Show filter bar as a single-line `Paragraph` at the top of the list area when `filter_active == true`.

```rust
// In app.rs
pub fn visible_distros(&self) -> Vec<&DistroInfo> {
    if self.filter.is_empty() {
        self.distros.iter().collect()
    } else {
        let q = self.filter.to_lowercase();
        self.distros.iter().filter(|d| d.name.to_lowercase().contains(&q)).collect()
    }
}
```

Event handling: when `filter_active`, route `Char(c)` → append to filter, `Backspace` → pop, `Esc` → clear and deactivate.

### Pattern 8: WSL Command Mapping

All WSL operations go through `WslExecutor::run()` in `wsl-core`. Phase 2 needs these additions to `wsl-core/src/wsl/`:

| Operation | wsl.exe command | New method |
|-----------|----------------|------------|
| List installed | `--list --verbose` | `parse_list_verbose()` → `Vec<DistroInfo>` |
| List online | `--list --online` | `list_online()` → `Vec<OnlineDistro>` |
| Install | `--install <name>` | (spawn async, stream output lines) |
| Start (no-op) | `-d <name> -- true` | `start_distro(name)` |
| Stop | `--terminate <name>` | `terminate_distro(name)` |
| Set default | `--set-default <name>` | `set_default(name)` |
| Unregister | `--unregister <name>` | `unregister(name)` |
| Export | `--export <name> <file>` | `export_distro(name, path)` |
| Import | `--import <name> <dir> <file>` | `import_distro(name, dir, path)` |
| Update kernel | `--update` | `update_wsl()` |

### Pattern 9: Parsing `wsl --list --verbose`

Output format (verified, source: Microsoft Learn):
```
  NAME                STATE      VERSION
* Ubuntu              Running    2
  docker-desktop      Stopped    2
```

Parse rule: skip header line, trim each line, check if starts with `*` (default), extract fields by fixed-width column positions or whitespace split. The default marker `*` and NAME may be fused — split on the first space after potential `*`.

```rust
// wsl-core/src/wsl/distro.rs
pub struct DistroInfo {
    pub name: String,
    pub state: DistroState,
    pub version: u8,    // 1 or 2
    pub is_default: bool,
}

pub enum DistroState { Running, Stopped }
```

Parse logic: `let is_default = line.starts_with('*'); let rest = line.trim_start_matches('*').trim();` then split whitespace to get `[name, state, version]`.

### Pattern 10: Keybindings Configuration (TUI-14)

Store keybindings as `String` in `config.toml` and parse to `KeyCode` at load time. No external crate needed.

```toml
# ~/.wsl-tui/config.toml
[keybindings]
move_down = "j"
move_up = "k"
attach = "enter"
quit = "q"
help = "?"
filter = "/"
```

```rust
// wsl-tui/src/keybindings.rs
#[derive(Debug, serde::Deserialize)]
pub struct KeyBindings {
    pub move_down: String,    // "j", "down", "ctrl+n"
    pub move_up: String,      // "k", "up"
    // ...
}

impl KeyBindings {
    pub fn matches_down(&self, key: &KeyEvent) -> bool {
        parse_key_code(&self.move_down) == Some(key.code)
    }
}

fn parse_key_code(s: &str) -> Option<KeyCode> {
    match s.to_lowercase().as_str() {
        "j" | "down" => Some(KeyCode::Down),  // "j" matches KeyCode::Char('j') not Down
        // ...more matches
    }
}
```

**Simpler approach for Phase 2:** Store default keybindings as constants in code, add serde config struct in `Config` with `#[serde(default)]`. Map string → KeyCode at startup. The mapping function is 30 lines; no crate needed.

### Anti-Patterns to Avoid

- **Blocking on `wsl.exe` in the render thread:** WSL commands (especially `--install`) can take minutes. Always spawn to a `tokio::task::spawn_blocking` or use `Command::new("wsl.exe").output()` in a `tokio::spawn` async task and send progress via a channel.
- **Storing `ListState` behind `&` during render:** `render_stateful_widget` requires `&mut State`. Either clone the state for render or pass a `&mut ListState` field from App through the render call.
- **Not filtering `KeyEventKind::Press`:** Windows double-fire. The filter is already in Phase 1's event loop — preserve it in the upgraded async loop.
- **Calling `ratatui::restore()` inside panic hook while it's already registered:** `ratatui::init()` installs the hook; calling `ratatui::restore()` for shell attach before `ratatui::init()` re-registers is fine — the hook is one-time per init call.
- **Using `.unwrap()` anywhere:** Project standard prohibits it. Use `?` or `.expect("descriptive reason")` for truly unrecoverable cases.
- **Making `ListState::selected` the source of truth for index:** Track selection index independently; `ListState` can drift if filter changes the visible list. Reconcile on filter change.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Popup centering | Custom layout math | `Layout::horizontal/vertical` with `Flex::Center` | Official ratatui pattern; handles resize |
| Clear background under popup | Manual overdraw | `frame.render_widget(Clear, area)` | Built-in widget; correct approach |
| UTF-16LE decode of wsl.exe output | Custom decoder | `WslExecutor::decode_output()` | Already implemented in Phase 1 |
| Input text buffer with Unicode | Hand-rolled string + byte index | Keep it simple: `String::chars()` for display, push_str/pop for edit | For single-line filter, full textarea is overkill |
| Async runtime | Manual threads | `tokio::spawn` + channels | Already present in workspace |
| Color palette | Hard-coded Color::Yellow etc. | `theme.rs` module with `Color::Rgb` constants | Maintainable, matches THEME_GUIDELINES.md exactly |

**Key insight:** The ratatui ecosystem has first-class solutions for everything Phase 2 needs. The main work is integration, not building new primitives.

---

## Common Pitfalls

### Pitfall 1: wsl.exe Output Encoding on `--list --verbose`

**What goes wrong:** `WslExecutor::decode_output()` handles UTF-16LE by default, but `wsl --list --verbose` with `WSL_UTF8=1` outputs UTF-8. Without the env var, all output is UTF-16LE. The existing `decode_output` already handles this correctly.

**Why it happens:** wsl.exe uses UTF-16LE by default on Windows. `list_verbose()` calls `run()` which calls `decode_output()` — already correct.

**How to avoid:** Always use `WslExecutor::run()` / `list_verbose()`. Never read raw bytes from wsl.exe directly.

**Warning signs:** Garbled or empty distro names in the parsed list.

### Pitfall 2: `wsl --list --verbose` Header Line and Unicode Markers

**What goes wrong:** The first line is always the header `NAME  STATE  VERSION`. The default distro has `*` prepended with a space before the name. Parsing on whitespace split naively will give `["*", "Ubuntu", "Running", "2"]` or `["Ubuntu", "Running", "2"]`.

**Why it happens:** The asterisk and name are separated by a space: `* Ubuntu  Running  2`.

**How to avoid:** Skip the first line. For each subsequent line, `trim()` first, check `starts_with('*')`, then `trim_start_matches('*').trim()` before splitting on whitespace. Expect exactly 3 tokens: `[name, state, version]`.

**Warning signs:** Index out of bounds panics or wrong `is_default` assignments.

### Pitfall 3: ListState Selection Drift on Filter

**What goes wrong:** User selects index 3, activates filter, visible list shrinks to 2 items. `ListState::selected()` still returns `Some(3)` → `visible_distros()[3]` panics.

**Why it happens:** `ListState` holds a raw index; filtering changes the visible count but not the state.

**How to avoid:** When filter string changes, clamp `list_state.select(Some(0))` or find the currently selected distro's name in the filtered list and re-select by new index. Store the selected distro's name as the canonical selection, derive the index on render.

**Warning signs:** Panic on `visible_distros()[selected_idx]` after typing in the filter bar.

### Pitfall 4: Shell Attach Terminal State Corruption

**What goes wrong:** If `ratatui::restore()` is not called before `Command::new("wsl.exe").status()`, the child process inherits raw mode and alternate screen. The shell appears garbled or input is swallowed by ratatui's event loop.

**Why it happens:** Raw mode disables canonical input processing. Alternate screen hides the shell output.

**How to avoid:** Always call `ratatui::restore()` before spawning the shell and `ratatui::init()` (reassigning the terminal handle) immediately after `child.wait()`. Pattern is: restore → spawn → wait → init.

**Warning signs:** Shell prompt invisible, no input echoing, garbage characters after returning to TUI.

### Pitfall 5: KeyEventKind Double-Fire in Upgraded Event Loop

**What goes wrong:** After upgrading to `EventStream`, the `KeyEventKind::Press` filter must be preserved. If the filter is accidentally removed during refactor, every key fires twice.

**Why it happens:** Windows crossterm generates Press + Release for each keypress, regardless of sync vs async event reading.

**How to avoid:** Ensure `if key.kind != KeyEventKind::Press { continue; }` (or equivalent) appears in all event handlers, including the new async path.

**Warning signs:** Every keystroke moves the list selection by 2 rows instead of 1.

### Pitfall 6: tokio::task blocking on wsl.exe during install

**What goes wrong:** `wsl --install <name>` runs interactively and may take minutes. Calling it with `.output()` in the main task blocks the entire event loop.

**Why it happens:** `std::process::Command::output()` is a blocking call.

**How to avoid:** Use `tokio::process::Command` (from tokio's `process` feature) or `tokio::task::spawn_blocking`. Send progress updates to `App` via a `tokio::sync::mpsc` channel. The main loop reads from the channel on each `tokio::select!` iteration.

**Warning signs:** TUI freezes during install; no UI updates until wsl.exe returns.

### Pitfall 7: `wsl --install` Progress — No Machine-Readable Output

**What goes wrong:** `wsl --install <name>` writes human-readable progress text to stdout that changes with WSL versions. Parsing it precisely is fragile.

**Why it happens:** Microsoft does not provide a structured progress API for `wsl --install`.

**How to avoid:** For the modal progress bar during install, show a coarse progress indicator (time-based spinner or step counter: "Downloading...", "Installing...", "Complete") rather than attempting to parse exact percentages from wsl output. The spec says "per-step progress feedback" — interpret this as step labels, not exact percentages.

**Warning signs:** Progress bar stuck at 0% or never reaching 100%.

---

## Code Examples

Verified patterns from official sources:

### Popup Area Helper (Source: ratatui.rs/examples/apps/popup)

```rust
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
```

### Clear + Block Popup (Source: ratatui.rs/examples/apps/popup)

```rust
let area = popup_area(frame.area(), 60, 20);
frame.render_widget(Clear, area);
frame.render_widget(Block::bordered().title("Popup"), area);
```

### Gauge for Install Progress (Source: docs.rs/ratatui Gauge)

```rust
Gauge::default()
    .block(Block::bordered().title("Installing Ubuntu-24.04"))
    .gauge_style(Style::new().fg(Color::Rgb(166, 227, 161)).bg(Color::Rgb(49, 50, 68)))
    .percent(app.install_progress_pct)
    .label(format!("{}%  {}", app.install_progress_pct, app.install_step))
    .use_unicode(true)
```

### Horizontal Layout Split (Source: docs.rs/ratatui Layout)

```rust
let [left, right] = Layout::horizontal([
    Constraint::Percentage(40),
    Constraint::Percentage(60),
]).areas(area);
```

### Stateful List Render (Source: docs.rs/ratatui List)

```rust
let list = List::new(items)
    .block(Block::bordered().title("Distros"))
    .highlight_style(Style::new().reversed())
    .highlight_symbol("▸ ");

frame.render_stateful_widget(list, area, &mut state);
```

### Shell Attach Pattern

```rust
// In event handler:
ratatui::restore();
let _ = std::process::Command::new("wsl.exe")
    .args(["-d", distro_name])
    .status();
*terminal = ratatui::init();
```

### Catppuccin Mocha Constants (Source: docs/THEME_GUIDELINES.md — verified against catppuccin.com)

```rust
// wsl-tui/src/theme.rs
use ratatui::style::Color;

pub const MAUVE:    Color = Color::Rgb(203, 166, 247);  // Primary accent, selection
pub const BLUE:     Color = Color::Rgb(137, 180, 250);  // Secondary accent
pub const GREEN:    Color = Color::Rgb(166, 227, 161);  // Running state, success
pub const YELLOW:   Color = Color::Rgb(249, 226, 175);  // Warning, pending
pub const RED:      Color = Color::Rgb(243, 139, 168);  // Error, stopped state
pub const SAPPHIRE: Color = Color::Rgb(116, 199, 236);  // Info, help text
pub const PEACH:    Color = Color::Rgb(250, 179, 135);  // Search highlight
pub const LAVENDER: Color = Color::Rgb(180, 190, 254);  // Tab headers
pub const TEAL:     Color = Color::Rgb(148, 226, 213);  // Connection status
pub const PINK:     Color = Color::Rgb(245, 194, 231);  // Provisioning
pub const TEXT:     Color = Color::Rgb(205, 214, 244);  // Primary text
pub const SUBTEXT1: Color = Color::Rgb(186, 194, 222);  // Secondary text
pub const SUBTEXT0: Color = Color::Rgb(166, 173, 200);  // Muted text
pub const OVERLAY0: Color = Color::Rgb(108, 112, 134);  // Dimmed text
pub const SURFACE2: Color = Color::Rgb(88, 91, 112);    // Scrollbars
pub const SURFACE1: Color = Color::Rgb(69, 71, 90);     // Borders, inactive
pub const SURFACE0: Color = Color::Rgb(49, 50, 68);     // Panels
pub const BASE:     Color = Color::Rgb(30, 30, 46);     // Main background
pub const MANTLE:   Color = Color::Rgb(24, 24, 37);     // Status bar background
```

---

## WSL Command Reference

All commands verified via Microsoft Learn (updated 2025-12-01).

| Operation | Command | Notes |
|-----------|---------|-------|
| List installed | `wsl.exe --list --verbose` | Output: header + rows with `*` for default |
| List online | `wsl.exe --list --online` | Output: header + NAME + FRIENDLY NAME columns |
| Install | `wsl.exe --install <Name>` | Interactive; long-running; no machine-readable progress |
| Start (no-op) | `wsl.exe -d <Name>` | Starts and attaches; for "just start", use `wsl -d <Name> -- true` |
| Shell attach | `wsl.exe -d <Name>` | Inherits stdin/stdout/stderr; blocks until exit |
| Stop/Terminate | `wsl.exe --terminate <Name>` | Same for stop and force-stop in WSL2 |
| Set default | `wsl.exe --set-default <Name>` | Changes `*` in verbose listing |
| Unregister | `wsl.exe --unregister <Name>` | PERMANENT data loss; requires confirmation |
| Export | `wsl.exe --export <Name> <file.tar>` | Creates .tar snapshot |
| Import | `wsl.exe --import <Name> <dir> <file.tar>` | Creates new distro from .tar |
| Update kernel | `wsl.exe --update` | Downloads and installs latest WSL kernel |

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Synchronous `event::read()` | `EventStream` + `tokio::select!` | Phase 2 (now) | Enables background async tasks alongside event handling |
| Monolithic `ratatui` crate | Split into `ratatui-core` + `ratatui-widgets` | ratatui 0.30 | Transparent for consumers using `ratatui` umbrella crate |
| `tui-rs` (archived) | `ratatui` (active fork) | 2023 | ratatui is the maintained standard |
| `highlight_symbol(&str)` | `highlight_symbol(Into<Line>)` | ratatui 0.30 | Allows styled highlight symbols; was breaking change |
| `Layout::split(area)` returns `Rc<[Rect]>` | `Layout::areas::<N>(area)` returns `[Rect; N]` | ratatui recent | Compile-time constraint count checking; destructuring works |

**Deprecated/outdated:**
- `tui-rs`: archived, use `ratatui`
- `wslconfig.exe`: deprecated, use `wsl.exe` subcommands
- `crossterm::execute!(stdout, EnterAlternateScreen)` manually: still valid but `ratatui::init()`/`ratatui::restore()` wraps this correctly

---

## Open Questions

1. **`wsl --install` progress parsing**
   - What we know: `wsl --install` produces human-readable text progress; format is not stable
   - What's unclear: Whether output can be reliably line-buffered for step-by-step status
   - Recommendation: Implement coarse phase labels ("Downloading", "Installing", "Configuring", "Done") driven by a `spawn_blocking` task that reads stdout lines and maps keywords to phase transitions. Accept that exact percentage from wsl is unavailable.

2. **`wsl -d <Name> -- true` for silent start**
   - What we know: Running `wsl -d <Name>` starts and attaches interactively; `-- true` should run `true` and exit, starting the distro without attaching
   - What's unclear: Whether `-- true` reliably starts the WSL instance without user-visible output on all Windows 11 WSL versions
   - Recommendation: Test on the dev machine; if unreliable, use `wsl -d <Name> -- echo started` as a probe; document the tested behavior.

3. **`ratatui::init()` call count safety**
   - What we know: Shell attach calls `ratatui::restore()` then `ratatui::init()` for each attach cycle
   - What's unclear: Whether calling `init()` multiple times (reinstalling the panic hook) causes issues
   - Recommendation: Inspect ratatui source or test empirically; likely safe since each `init()` installs a fresh hook and returns a new terminal handle.

4. **Keybinding serde for `KeyCode`**
   - What we know: `KeyCode` does not derive serde by default; needs custom impl or wrapping
   - What's unclear: Exact serde feature flag for ratatui (ratatui has a `serde` feature for `Color`/`Style` but not `KeyCode`)
   - Recommendation: Implement a simple parse function `fn parse_key(s: &str) -> Option<KeyCode>` that maps "j" → `KeyCode::Char('j')`, "enter" → `KeyCode::Enter`, etc. Store as `String` in TOML and parse at startup. No serde derive on KeyCode needed.

---

## Sources

### Primary (HIGH confidence)

- [ratatui 0.30 highlights](https://ratatui.rs/highlights/v030/) — crate structure, List/Layout API changes, StatefulWidget changes
- [docs.rs ratatui List](https://docs.rs/ratatui/latest/ratatui/widgets/struct.List.html) — ListState, highlight_style, render_stateful_widget
- [docs.rs ratatui Layout](https://docs.rs/ratatui/latest/ratatui/layout/struct.Layout.html) — Constraint variants, .areas(), Flex::Center
- [docs.rs ratatui Gauge](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Gauge.html) — percent, ratio, gauge_style, label, use_unicode
- [ratatui popup example](https://ratatui.rs/examples/apps/popup/) — Clear widget, popup_area helper with Flex::Center
- [ratatui async event stream tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) — EventStream, tokio::select! pattern
- [Microsoft Learn WSL basic commands](https://learn.microsoft.com/en-us/windows/wsl/basic-commands) (updated 2025-12-01) — all wsl.exe command syntax verified
- [catppuccin.com/palette](https://catppuccin.com/palette/) — exact RGB values for all Mocha colors
- [docs/THEME_GUIDELINES.md](../../../docs/THEME_GUIDELINES.md) — project-specific color role assignments

### Secondary (MEDIUM confidence)

- [ratatui terminal and event handler recipe](https://ratatui.rs/recipes/apps/terminal-and-event-handler/) — suspend/resume pattern description
- [crossterm terminal module docs](https://docs.rs/crossterm/latest/crossterm/terminal/index.html) — enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen
- [ratatui user input example](https://ratatui.rs/examples/apps/user_input/) — inline text input state pattern

### Tertiary (LOW confidence)

- WebSearch results on keybinding serde patterns — not directly verified with official docs; recommend parse_key() function approach

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in workspace, verified present in Cargo.toml
- Architecture: HIGH — ratatui widget APIs verified against docs.rs and ratatui.rs
- WSL commands: HIGH — verified against Microsoft Learn (updated 2025-12-01)
- Theme colors: HIGH — verified against catppuccin.com and project THEME_GUIDELINES.md
- Shell attach pattern: MEDIUM — core crossterm functions verified; exact ratatui::init() multi-call safety is an open question
- Install progress: MEDIUM — WSL install is interactive; progress detail level is an open question
- Keybinding serde: MEDIUM — parse_key() approach is straightforward; not tested

**Research date:** 2026-02-21
**Valid until:** 2026-04-21 (ratatui 0.30 is stable; WSL commands are stable; may need refresh if ratatui 0.31 releases)
