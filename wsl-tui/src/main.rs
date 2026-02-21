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
//!
//! # Shell attach
//!
//! Shell attach is handled synchronously in `run_app` (not inside
//! `execute_action`) so the mutable terminal reference is accessible.
//! The pattern is:
//! 1. `execute_action` returns `true` when an `AttachShell` action is requested.
//! 2. `run_app` detects this sentinel and runs the shell attach inline.
//! 3. TUI is suspended (`ratatui::restore()`), `wsl.exe` runs with inherited
//!    stdio, then TUI is re-initialised (`ratatui::init()`).

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
///
/// When `execute_action` signals an `AttachShell` request (returns `true`),
/// the shell attach is performed here so the terminal handle is available.
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

                    // Shell attach must be handled here because it needs to
                    // mutate the terminal reference.
                    if action == Action::AttachShell {
                        attach_shell(terminal, app);
                    } else {
                        execute_action(app, action).await;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Suspend the TUI, drop into a `wsl.exe` shell, and fully restore the TUI.
///
/// Per the locked decision:
/// - **Instant swap:** `ratatui::restore()` is called immediately — no transition message.
/// - **Auto-start:** If the distro is stopped, it is started before attaching.
/// - **Full restore:** `ratatui::init()` re-enters raw mode + alternate screen;
///   app state (selection, scroll, view) is preserved in `app` automatically.
/// - **Refresh:** After return, `refresh_distros()` picks up any state changes
///   made during the shell session.
///
/// The `wsl.exe` process inherits stdin/stdout/stderr from the current process
/// so the shell runs on the user's terminal directly.
fn attach_shell(terminal: &mut ratatui::DefaultTerminal, app: &mut App) {
    // Resolve the selected distro name; bail if nothing is selected.
    let Some(distro_name) = app.selected_name.clone() else {
        return;
    };

    // Auto-start: start the distro if it is stopped.
    if let Some(distro) = app.selected_distro() {
        if distro.state == wsl_core::DistroState::Stopped {
            let executor = app.executor.clone();
            let name = distro_name.clone();
            // Blocking call — acceptable here because we are about to suspend
            // the TUI anyway.
            let _ = executor.start_distro(&name);
        }
    }

    // Instant swap: suspend TUI, give the terminal back to the shell.
    ratatui::restore();

    // Drop into shell — inherits stdin/stdout/stderr from the calling process.
    let _ = std::process::Command::new("wsl.exe")
        .args(["-d", &distro_name])
        .status();

    // Full restore: re-enter raw mode + alternate screen + reinstall panic hook.
    *terminal = ratatui::init();

    // App state (selection, scroll, view) is already preserved in `app`.
    // Refresh the distro list to pick up any state changes from the session.
    let _ = app.refresh_distros();
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
            KeyCode::Esc | KeyCode::Enter => Action::FilterEscape,
            KeyCode::Backspace => Action::FilterBackspace,
            KeyCode::Up => Action::MoveUp,
            KeyCode::Down => Action::MoveDown,
            KeyCode::Char('k') => Action::MoveUp,
            KeyCode::Char('j') => Action::MoveDown,
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
/// Blocking wsl.exe calls (start, stop, set_default, unregister) are wrapped
/// in `tokio::task::spawn_blocking` so the async executor is not blocked.
///
/// **Note:** `Action::AttachShell` is handled in `run_app` directly (not here)
/// because it requires mutating the `terminal` reference.
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
        Action::ToggleFilter => app.activate_filter(),

        // ── Filter input ──────────────────────────────────────────────────────
        Action::FilterChar(c) => app.filter_push_char(c),
        Action::FilterBackspace => app.filter_pop_char(),
        Action::FilterEscape => app.deactivate_filter(),

        // ── Modal responses ───────────────────────────────────────────────────
        Action::ConfirmYes => {
            // Extract the distro name while the modal is active.
            if let ModalState::Confirm {
                ref distro_name, ..
            } = app.modal.clone()
            {
                let name = distro_name.clone();
                let executor = app.executor.clone();
                // Run unregister in a blocking task — it calls wsl.exe.
                let _ = tokio::task::spawn_blocking(move || executor.unregister(&name)).await;
                // Refresh list to reflect the removal.
                let _ = app.refresh_distros();
            }
            app.modal = ModalState::None;
        }
        Action::ConfirmNo => {
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
            // Show the confirmation modal; the actual unregister runs on ConfirmYes.
            if let Some(name) = app.selected_name.clone() {
                app.modal = ModalState::Confirm {
                    distro_name: name,
                    action_label: "Confirm Remove".to_string(),
                };
            }
        }

        // AttachShell is handled in run_app, not here — it needs &mut terminal.
        Action::AttachShell => {}

        Action::ExportDistro | Action::ImportDistro | Action::InstallDistro | Action::UpdateWsl => {
            // Phase 05 implements these flows.
            // Placeholder: no-op.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wsl_core::Config;

    /// Build a minimal Config for tests.
    fn make_config() -> Config {
        Config {
            storage: wsl_core::StorageMode::Auto,
            config_dir: std::path::PathBuf::new(),
            first_run: false,
            keybindings: wsl_core::RawKeybindings::default(),
        }
    }

    // ── RemoveDistro triggers ModalState::Confirm ──────────────────────────────

    #[tokio::test]
    async fn test_remove_triggers_confirm_modal() {
        let config = make_config();
        let mut app = App::new(&config);
        // Simulate a selected distro.
        app.selected_name = Some("Ubuntu".to_string());

        execute_action(&mut app, Action::RemoveDistro).await;

        match &app.modal {
            ModalState::Confirm { distro_name, .. } => {
                assert_eq!(distro_name, "Ubuntu", "Confirm modal should target the selected distro");
            }
            other => panic!("Expected ModalState::Confirm, got {other:?}"),
        }
    }

    // ── ConfirmNo clears modal ─────────────────────────────────────────────────

    #[tokio::test]
    async fn test_confirm_cancel_clears_modal() {
        let config = make_config();
        let mut app = App::new(&config);
        // Put a confirm modal in place.
        app.modal = ModalState::Confirm {
            distro_name: "Ubuntu".to_string(),
            action_label: "Confirm Remove".to_string(),
        };

        execute_action(&mut app, Action::ConfirmNo).await;

        assert_eq!(
            app.modal,
            ModalState::None,
            "ConfirmNo should clear the modal"
        );
    }

    // ── ToggleHelp opens and closes help overlay ───────────────────────────────

    #[tokio::test]
    async fn test_toggle_help_opens_help_modal() {
        let config = make_config();
        let mut app = App::new(&config);
        assert_eq!(app.modal, ModalState::None);

        execute_action(&mut app, Action::ToggleHelp).await;
        assert_eq!(app.modal, ModalState::Help, "ToggleHelp should open Help modal");

        execute_action(&mut app, Action::ToggleHelp).await;
        assert_eq!(app.modal, ModalState::None, "ToggleHelp again should close Help modal");
    }

    // ── ToggleFilter activates filter ─────────────────────────────────────────

    #[tokio::test]
    async fn test_toggle_filter_activates() {
        let config = make_config();
        let mut app = App::new(&config);
        assert!(!app.filter_active);

        execute_action(&mut app, Action::ToggleFilter).await;
        assert!(app.filter_active, "ToggleFilter should activate the filter");
    }

    // ── FilterEscape deactivates filter ───────────────────────────────────────

    #[tokio::test]
    async fn test_filter_escape_deactivates() {
        let config = make_config();
        let mut app = App::new(&config);
        app.activate_filter();
        app.filter_push_char('u');
        app.filter_push_char('b');

        execute_action(&mut app, Action::FilterEscape).await;
        assert!(!app.filter_active, "FilterEscape should deactivate filter");
        assert!(app.filter_text.is_empty(), "FilterEscape should clear filter text");
    }
}
