# wsl-tui — Agent Reference

`wsl-tui` is the TUI binary crate for the WSL TUI workspace. It owns the interactive terminal interface: the event loop, application state, and all UI rendering. Business logic, storage, and WSL execution are in `wsl-core`.

---

## Crate Purpose

- Entry point for the interactive TUI (`wsl-tui.exe`)
- Manages terminal lifecycle (raw mode, alternate screen, panic recovery)
- Drives the event loop and dispatches input to application state
- Renders UI frames to the terminal using ratatui

---

## Source Layout

```
wsl-tui/src/
├── main.rs          — entry point: Config::load, ratatui::init, run_app, ratatui::restore
├── app.rs           — App struct (running, first_run, show_welcome) + methods
└── ui/
    ├── mod.rs       — render() dispatcher: welcome vs placeholder
    └── welcome.rs   — first-run welcome screen (centered, Catppuccin colors)
```

---

## Entry Point (`main.rs`)

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;          // 1. Load config (creates ~/.wsl-tui/ on first run)
    let mut app = App::new(&config);       // 2. Build app state

    let mut terminal = ratatui::init();    // 3. Enter raw mode + alternate screen + install panic hook

    let result = run_app(&mut terminal, &mut app).await;

    ratatui::restore();                    // 4. Always restore terminal (even after panic via hook)

    result
}
```

**`ratatui::init()`** switches to raw mode, enters the alternate screen, and installs a panic hook that calls `ratatui::restore()` before re-raising. This guarantees the terminal is always cleaned up.

---

## Application State (`app.rs`)

```rust
pub struct App {
    pub running: bool,         // false → event loop exits
    pub first_run: bool,       // mirrors Config.first_run; Phase 2 status bar consumer
    pub show_welcome: bool,    // true on first run; false after any key press
}
```

| Method | Description |
|---|---|
| `App::new(config)` | Sets `running=true`, copies `first_run` and `show_welcome` from `config.first_run` |
| `App::quit()` | Sets `running = false` — event loop exits on next iteration |
| `App::dismiss_welcome()` | Sets `show_welcome = false` — renders main screen on next frame |

**Note on `first_run`:** The field carries `#[allow(dead_code)]` in Phase 1 because the Phase 2 status bar consumer does not yet exist. Do not remove this attribute until Phase 2 adds the consumer.

---

## Event Loop (`run_app` in main.rs)

```rust
while app.running {
    terminal.draw(|frame| ui::render(app, frame))?;

    let event = crossterm::event::read()?;   // synchronous block

    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {  // CRITICAL: Windows double-fire filter
            continue;
        }

        if app.show_welcome {
            app.dismiss_welcome();            // any key dismisses welcome
        } else {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => app.quit(),
                _ => {}                       // Phase 2 keybindings go here
            }
        }
    }
}
```

**Phase 1 uses synchronous `crossterm::event::read()`** — there are no background async tasks competing for the loop. Phase 2 will upgrade to `EventStream` + `tokio::select!` when background distro polling is added.

---

## Event Handling Rules

### KeyEventKind::Press Filter (Mandatory)

On Windows, crossterm generates **both a Press and a Release event** for every keystroke. Without this filter, every key is processed twice. This must appear in every key event handler:

```rust
if key.kind != KeyEventKind::Press {
    continue;
}
```

**This rule applies to any new event handler added in later phases.** Never process `KeyEventKind::Release` or `KeyEventKind::Repeat` unless explicitly required.

---

## UI Rendering (`ui/` module)

### Dispatch (`mod.rs`)

```rust
pub fn render(app: &App, frame: &mut Frame) {
    if app.show_welcome {
        welcome::render_welcome(frame);
    } else {
        render_placeholder(frame);  // Phase 1 stub; replaced in Phase 2
    }
}
```

Add new views by:
1. Adding a new `pub mod <view>` in `ui/mod.rs`
2. Creating `ui/<view>.rs` with a `render_<view>(frame: &mut Frame)` function
3. Adding the dispatch condition in `render()` (check `app.<flag>`)
4. Adding the corresponding `App` field + keybinding

### Welcome Screen (`welcome.rs`)

Displayed when `app.show_welcome` is `true` (first run only). Features:
- Vertically centered using `Layout::Fill(1)` sandwich (12-row fixed height)
- Horizontally centered with `width.clamp(44, 72)` for narrow terminal support
- Catppuccin-inspired colors: Cyan title, Yellow config path, Green hint, Magenta border
- Content: config path (`~/.wsl-tui/config.toml`), customization hint, any-key dismiss prompt

**Catppuccin color usage in Phase 1:**

| UI element | Color |
|---|---|
| Title text | `Color::Cyan` + `BOLD` |
| Config path | `Color::Yellow` |
| Success hint | `Color::Green` |
| Dismiss prompt | `Color::DarkGray` + `ITALIC` |
| Border | `Color::Magenta` |
| Border title | `Color::Magenta` + `BOLD` |

Full Catppuccin Mocha palette and guidelines are in `docs/THEME_GUIDELINES.md`.

---

## Adding a New View

1. Create `wsl-tui/src/ui/<view_name>.rs`.
2. Implement `pub fn render_<view_name>(frame: &mut Frame)`.
3. In `ui/mod.rs`, add `pub mod <view_name>;`.
4. Add the dispatch in `ui::render()`:
   ```rust
   } else if app.show_<view_name> {
       <view_name>::render_<view_name>(frame);
   }
   ```
5. Add `pub show_<view_name>: bool` to `App` in `app.rs`.
6. Add a method to `App` to toggle the view (e.g., `fn open_<view_name>(&mut self)`).
7. Add the keybinding in `run_app` in `main.rs`:
   ```rust
   KeyCode::Char('<key>') => app.open_<view_name>(),
   ```
8. Add unit tests for the new `App` methods.

---

## Dependencies (wsl-tui)

| Crate | Usage |
|---|---|
| `wsl-core` | Config, App state types |
| `ratatui 0.30` | Terminal rendering framework |
| `crossterm 0.29` | Terminal backend (raw mode, events) |
| `tokio 1 full` | Async runtime (`#[tokio::main]`) |
| `anyhow 1` | Error propagation in binary |

---

## Testing

- Unit tests live in `app.rs` (`#[cfg(test)] mod tests`)
- Tests use a `make_config(first_run: bool) -> Config` helper that constructs a minimal `Config` without touching `~/.wsl-tui/`
- Run with: `cargo test -p wsl-tui`
- Integration tests (Phase 2+): `wsl-tui/tests/` directory

**Phase 1 test count:** 6 unit tests in `app.rs` covering `App::new`, `quit`, `dismiss_welcome`, idempotency, and flag independence.
