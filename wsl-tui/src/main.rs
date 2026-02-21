//! WSL TUI — entry point.
//!
//! Initialises the terminal, loads config, runs the event loop, and restores
//! the terminal unconditionally (both on normal exit and on panic).
//!
//! # Panic safety
//!
//! `ratatui::init()` installs a panic hook that calls `ratatui::restore()`
//! before unwinding, so the terminal is always left in a clean state.
//!
//! # KeyEventKind filter
//!
//! On Windows, crossterm fires two events per key press (Press + Release).
//! The event loop filters to `KeyEventKind::Press` only to prevent double
//! handling.

mod app;
pub mod theme;
mod ui;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use wsl_core::Config;

use app::App;

/// Application entry point.
///
/// 1. Load config (creates `~/.wsl-tui/` and default `config.toml` on first run)
/// 2. Initialise terminal (installs panic hook via `ratatui::init`)
/// 3. Run event loop
/// 4. Restore terminal unconditionally
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    let mut app = App::new(&config);

    // ratatui::init() switches to raw mode, alternate screen, and installs a
    // panic hook that calls ratatui::restore() before re-raising.
    let mut terminal = ratatui::init();

    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal regardless of whether run_app returned Ok or Err.
    ratatui::restore();

    result
}

/// Drive the TUI event loop until `app.running` is `false`.
///
/// On each iteration:
/// 1. Draw a frame.
/// 2. Wait for and process the next terminal event.
///
/// Uses synchronous `crossterm::event::read()` — sufficient for Phase 1
/// where there are no background async tasks competing for the loop.
/// Phase 2 will upgrade to `EventStream` + `tokio::select!` when needed.
async fn run_app(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
) -> anyhow::Result<()> {
    while app.running {
        // Draw current state.
        terminal.draw(|frame| ui::render(app, frame))?;

        // Block until an event arrives.
        let event = crossterm::event::read()?;

        if let Event::Key(key) = event {
            // CRITICAL: filter to Press events only.
            // On Windows, crossterm generates both a Press and a Release event
            // for every keystroke.  Without this filter, each key would be
            // processed twice.
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.show_welcome {
                // Any key dismisses the welcome screen.
                app.dismiss_welcome();
            } else {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => app.quit(),
                    // Placeholder: Phase 2 keybindings go here.
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
