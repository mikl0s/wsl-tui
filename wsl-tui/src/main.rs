//! WSL TUI — entry point.
//!
//! Initialises the terminal, loads config, runs the async event loop, and
//! restores the terminal unconditionally (both on normal exit and on panic).
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
//!
//! # Async event loop
//!
//! The event loop uses `crossterm::event::EventStream` + `tokio::select!` so
//! that a background ticker can trigger distro-list refreshes every 5 seconds
//! without blocking the UI.

mod action;
mod app;
pub mod keybindings;
pub mod theme;
mod ui;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::StreamExt;
use tokio::time::{Duration, interval};
use wsl_core::Config;

use action::Action;
use app::{App, FocusPanel, ModalState, View};
use keybindings::{KeyAction, KeyBindings};

/// Application entry point.
///
/// 1. Load config (creates `~/.wsl-tui/` and default `config.toml` on first run)
/// 2. Build app state and parse keybindings from config
/// 3. Initialise terminal (installs panic hook via `ratatui::init`)
/// 4. Run the async event loop
/// 5. Restore terminal unconditionally
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    let keybindings = KeyBindings::from_config(&config);
    let mut app = App::new(&config);

    // ratatui::init() switches to raw mode, alternate screen, and installs a
    // panic hook that calls ratatui::restore() before re-raising.
    let mut terminal = ratatui::init();

    let result = run_app(&mut terminal, &mut app, &keybindings).await;

    // Restore terminal regardless of whether run_app returned Ok or Err.
    ratatui::restore();

    result
}

/// Drive the TUI event loop until `app.running` is `false`.
///
/// Uses `EventStream` + `tokio::select!` to multiplex two async sources:
///
/// 1. A 5-second ticker that triggers a background distro-list refresh.
/// 2. The crossterm event stream for keyboard input.
///
/// This ensures the UI is never blocked by a wsl.exe invocation.
async fn run_app(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    keybindings: &KeyBindings,
) -> anyhow::Result<()> {
    let mut events = EventStream::new();
    let mut poll_ticker = interval(Duration::from_secs(5));

    // Initial distro load — run inline on the first frame. Errors are logged
    // to stderr but do not crash the application.
    if let Err(e) = app.refresh_distros() {
        eprintln!("Warning: failed to load distros on startup: {e}");
    }

    while app.running {
        terminal.draw(|frame| ui::render(app, frame))?;

        tokio::select! {
            _ = poll_ticker.tick() => {
                // Background refresh — swallow errors; the previous list stays visible.
                let _ = app.refresh_distros();
            }
            maybe_event = events.next() => {
                let Some(Ok(event)) = maybe_event else { break };
                if let Event::Key(key) = event {
                    // CRITICAL: filter to Press events only.
                    // On Windows, crossterm generates both a Press and a Release event
                    // for every keystroke.  Without this filter, each key would be
                    // processed twice.
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    let action = resolve_action(app, keybindings, &key);
                    execute_action(app, action).await;
                }
            }
        }
    }

    Ok(())
}

/// Translate a raw [`KeyEvent`] into an [`Action`].
///
/// Routing priority (highest to lowest):
/// 1. Welcome screen — any key dismisses.
/// 2. Modal active — Escape cancels, y/n respond.
/// 3. Filter active — characters go to the filter bar.
/// 4. Normal — keybinding table + number-key view switching.
fn resolve_action(app: &App, kb: &KeyBindings, key: &KeyEvent) -> Action {
    // ── Welcome screen ────────────────────────────────────────────────────────
    if app.show_welcome {
        return Action::None; // handled separately via dismiss_welcome in execute_action
    }

    // ── Modal routing ─────────────────────────────────────────────────────────
    if app.modal != ModalState::None {
        return match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmYes,
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::ConfirmNo,
            _ => Action::None,
        };
    }

    // ── Filter routing ────────────────────────────────────────────────────────
    if app.filter_active {
        return match key.code {
            KeyCode::Esc => Action::FilterEscape,
            KeyCode::Backspace => Action::FilterBackspace,
            KeyCode::Char(c) => Action::FilterChar(c),
            _ => Action::None,
        };
    }

    // ── Normal keybindings ────────────────────────────────────────────────────

    // Number keys 1-5: view switching.
    if let KeyCode::Char(c) = key.code {
        match c {
            '1' => return Action::SwitchView(View::Dashboard),
            '2' => return Action::SwitchView(View::Provision),
            '3' => return Action::SwitchView(View::Monitor),
            '4' => return Action::SwitchView(View::Backup),
            '5' => return Action::SwitchView(View::Logs),
            _ => {}
        }
    }

    // Tab: switch focus between panels.
    if let KeyCode::Tab = key.code {
        return Action::SwitchFocus;
    }

    // Keybinding table.
    if kb.matches(key, KeyAction::Quit) {
        Action::Quit
    } else if kb.matches(key, KeyAction::Up) || matches!(key.code, KeyCode::Up) {
        Action::MoveUp
    } else if kb.matches(key, KeyAction::Down) || matches!(key.code, KeyCode::Down) {
        Action::MoveDown
    } else if kb.matches(key, KeyAction::Left) || matches!(key.code, KeyCode::Left) {
        Action::MoveLeft
    } else if kb.matches(key, KeyAction::Right) || matches!(key.code, KeyCode::Right) {
        Action::MoveRight
    } else if kb.matches(key, KeyAction::Help) {
        Action::ToggleHelp
    } else if kb.matches(key, KeyAction::Filter) {
        Action::ToggleFilter
    } else if kb.matches(key, KeyAction::Attach) {
        Action::AttachShell
    } else if kb.matches(key, KeyAction::Start) {
        Action::StartDistro
    } else if kb.matches(key, KeyAction::Stop) {
        Action::StopDistro
    } else if kb.matches(key, KeyAction::SetDefault) {
        Action::SetDefault
    } else if kb.matches(key, KeyAction::Remove) {
        Action::RemoveDistro
    } else if kb.matches(key, KeyAction::Export) {
        Action::ExportDistro
    } else if kb.matches(key, KeyAction::Import) {
        Action::ImportDistro
    } else {
        Action::None
    }
}

/// Execute an [`Action`] against the application state.
///
/// Blocking wsl.exe calls (start, stop, set_default) are wrapped in
/// `tokio::task::spawn_blocking` so the async executor is not blocked.
async fn execute_action(app: &mut App, action: Action) {
    match action {
        Action::None => {
            // Special case: if the welcome screen is up, dismiss on any key.
            if app.show_welcome {
                app.dismiss_welcome();
            }
        }

        Action::Quit => app.quit(),

        // ── Navigation ────────────────────────────────────────────────────────
        Action::MoveUp => {
            if app.focus == FocusPanel::DistroList {
                app.move_selection_up();
            }
        }
        Action::MoveDown => {
            if app.focus == FocusPanel::DistroList {
                app.move_selection_down();
            }
        }
        Action::MoveLeft | Action::MoveRight => {
            app.switch_focus();
        }
        Action::SwitchFocus => app.switch_focus(),

        // ── View switching ────────────────────────────────────────────────────
        Action::SwitchView(view) => app.set_view(view),

        // ── UI toggles ────────────────────────────────────────────────────────
        Action::ToggleHelp => {
            app.modal = if app.modal == ModalState::Help {
                ModalState::None
            } else {
                ModalState::Help
            };
        }
        Action::ToggleFilter => {
            app.filter_active = !app.filter_active;
            if !app.filter_active {
                app.filter_text.clear();
            }
        }

        // ── Filter input ──────────────────────────────────────────────────────
        Action::FilterChar(c) => {
            app.filter_text.push(c);
        }
        Action::FilterBackspace => {
            app.filter_text.pop();
        }
        Action::FilterEscape => {
            app.filter_active = false;
            app.filter_text.clear();
        }

        // ── Modal responses ───────────────────────────────────────────────────
        Action::ConfirmYes | Action::ConfirmNo => {
            // Phase 04-05 will implement modal-specific logic.
            // For now, just close the modal.
            app.modal = ModalState::None;
        }

        // ── Distro actions ────────────────────────────────────────────────────
        // These call wsl.exe via spawn_blocking to avoid blocking the async runtime.

        Action::StartDistro => {
            if let Some(name) = app.selected_name.clone() {
                let executor = app.executor.clone();
                let _ = tokio::task::spawn_blocking(move || executor.start_distro(&name)).await;
                let _ = app.refresh_distros();
            }
        }

        Action::StopDistro | Action::TerminateDistro => {
            if let Some(name) = app.selected_name.clone() {
                let executor = app.executor.clone();
                let _ = tokio::task::spawn_blocking(move || executor.terminate_distro(&name)).await;
                let _ = app.refresh_distros();
            }
        }

        Action::SetDefault => {
            if let Some(name) = app.selected_name.clone() {
                let executor = app.executor.clone();
                let _ = tokio::task::spawn_blocking(move || executor.set_default(&name)).await;
                let _ = app.refresh_distros();
            }
        }

        Action::RemoveDistro => {
            // Phase 04-05 will add a confirm modal before calling unregister.
            // For now, just show the confirm modal (without acting).
            if let Some(name) = app.selected_name.clone() {
                app.modal = ModalState::Confirm {
                    distro_name: name.clone(),
                    action_label: format!("Remove distro '{name}'?"),
                };
            }
        }

        Action::AttachShell => {
            // Phase 04 implements actual shell attach.
            // Placeholder: no-op.
        }

        Action::ExportDistro | Action::ImportDistro | Action::InstallDistro | Action::UpdateWsl => {
            // Phase 04-05 implements these flows.
            // Placeholder: no-op.
        }
    }
}
