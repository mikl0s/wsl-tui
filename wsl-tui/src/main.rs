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
//!
//! # Install / update progress channel
//!
//! Long-running operations (wsl --install, wsl --update) run in
//! `tokio::task::spawn_blocking` and send `(step, percent, completed)` tuples
//! through a `tokio::sync::mpsc::channel` to the main loop, which updates
//! `app.modal` on each received message.

mod action;
mod app;
pub mod keybindings;
pub mod theme;
mod ui;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::StreamExt;
use tokio::sync::mpsc;
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
/// Uses `EventStream` + `tokio::select!` to multiplex three async sources:
///
/// 1. A 5-second ticker that triggers a background distro-list refresh.
/// 2. The crossterm event stream for keyboard input.
/// 3. (Optional) A progress channel for install/update background tasks.
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

            // Install / update progress from background task.
            Some((step, percent, completed)) = async {
                if let Some(ref mut rx) = app.install_rx {
                    rx.recv().await
                } else {
                    // No channel active — yield a pending future so other branches run.
                    std::future::pending::<Option<(String, u16, bool)>>().await
                }
            } => {
                match &mut app.modal {
                    ModalState::InstallProgress {
                        step: modal_step,
                        percent: modal_percent,
                        completed: modal_completed,
                        ..
                    } => {
                        *modal_step = step;
                        *modal_percent = percent;
                        *modal_completed = completed;
                    }
                    ModalState::UpdateProgress {
                        step: modal_step,
                        completed: modal_completed,
                    } => {
                        *modal_step = step;
                        *modal_completed = completed;
                    }
                    _ => {}
                }
                if completed {
                    // Channel is exhausted; drop receiver.
                    app.install_rx = None;
                }
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

/// Compute the default export path for a given distro name.
///
/// Returns `C:\Users\<username>\Downloads\<name>.tar` if `dirs::download_dir()` is
/// available, otherwise falls back to `C:\<name>.tar`.
fn default_export_path(distro_name: &str) -> String {
    if let Some(dl) = dirs::download_dir() {
        format!("{}\\{}.tar", dl.display(), distro_name)
    } else if let Some(home) = dirs::home_dir() {
        format!("{}\\Downloads\\{}.tar", home.display(), distro_name)
    } else {
        format!("C:\\{}.tar", distro_name)
    }
}

/// Translate a raw [`KeyEvent`] into an [`Action`].
///
/// Routing priority (highest to lowest):
/// 1. Welcome screen — any key dismisses.
/// 2. Modal active — routes to modal-specific key handler.
/// 3. Filter active — characters go to the filter bar.
/// 4. Normal — keybinding table + number-key view switching.
fn resolve_action(app: &App, kb: &KeyBindings, key: &KeyEvent) -> Action {
    // ── Welcome screen ────────────────────────────────────────────────────────
    if app.show_welcome {
        return Action::None; // handled separately via dismiss_welcome in execute_action
    }

    // ── Modal routing ─────────────────────────────────────────────────────────
    match &app.modal {
        ModalState::None => {}

        ModalState::Confirm { .. } | ModalState::Help => {
            return match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmYes,
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::ConfirmNo,
                _ => Action::None,
            };
        }

        ModalState::InstallPicker { .. } => {
            return match key.code {
                KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
                KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
                KeyCode::Enter => Action::ConfirmYes, // select distro
                KeyCode::Esc => Action::ConfirmNo,    // cancel picker
                _ => Action::None,
            };
        }

        ModalState::InstallProgress { completed, .. }
        | ModalState::UpdateProgress { completed, .. } => {
            // While in progress: ignore all keys (progress modal blocks UI).
            // When completed: any key dismisses.
            return if *completed {
                Action::ConfirmNo // dismiss completed modal
            } else {
                Action::None
            };
        }

        ModalState::ExportInput { .. } => {
            return match key.code {
                KeyCode::Char(c) => Action::ModalInputChar(c),
                KeyCode::Backspace => Action::ModalInputBackspace,
                KeyCode::Left => Action::ModalInputLeft,
                KeyCode::Right => Action::ModalInputRight,
                KeyCode::Enter => Action::ConfirmYes,
                KeyCode::Esc => Action::ConfirmNo,
                _ => Action::None,
            };
        }

        ModalState::ImportInput { .. } => {
            return match key.code {
                KeyCode::Char(c) => Action::ModalInputChar(c),
                KeyCode::Backspace => Action::ModalInputBackspace,
                KeyCode::Left => Action::ModalInputLeft,
                KeyCode::Right => Action::ModalInputRight,
                KeyCode::Tab => Action::ModalInputTab,
                KeyCode::Enter => Action::ConfirmYes,
                KeyCode::Esc => Action::ConfirmNo,
                _ => Action::None,
            };
        }
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
    } else if let KeyCode::Char('I') = key.code {
        // Capital I for Install (lowercase i is Import)
        Action::InstallDistro
    } else if let KeyCode::Char('u') = key.code {
        Action::UpdateWsl
    } else {
        Action::None
    }
}

/// Execute an [`Action`] against the application state.
///
/// Blocking wsl.exe calls (start, stop, set_default, unregister, export, import)
/// are wrapped in `tokio::task::spawn_blocking` so the async executor is not blocked.
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
            match &mut app.modal {
                ModalState::InstallPicker { list_state, online_distros } => {
                    let count = online_distros.len();
                    if count == 0 {
                        return;
                    }
                    let current = list_state.selected().unwrap_or(0);
                    list_state.select(Some(current.saturating_sub(1)));
                }
                _ => {
                    if app.focus == FocusPanel::DistroList {
                        app.move_selection_up();
                    }
                }
            }
        }
        Action::MoveDown => {
            match &mut app.modal {
                ModalState::InstallPicker { list_state, online_distros } => {
                    let count = online_distros.len();
                    if count == 0 {
                        return;
                    }
                    let current = list_state.selected().unwrap_or(0);
                    list_state.select(Some((current + 1).min(count - 1)));
                }
                _ => {
                    if app.focus == FocusPanel::DistroList {
                        app.move_selection_down();
                    }
                }
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

        // ── Modal text input ──────────────────────────────────────────────────
        Action::ModalInputChar(c) => {
            match &mut app.modal {
                ModalState::ExportInput { path, cursor, .. } => {
                    let pos = *cursor;
                    let mut chars: Vec<char> = path.chars().collect();
                    chars.insert(pos, c);
                    *path = chars.iter().collect();
                    *cursor = pos + 1;
                }
                ModalState::ImportInput {
                    name,
                    install_dir,
                    tar_path,
                    active_field,
                    cursor,
                } => {
                    let field = match active_field {
                        0 => name,
                        1 => install_dir,
                        _ => tar_path,
                    };
                    let pos = *cursor;
                    let mut chars: Vec<char> = field.chars().collect();
                    chars.insert(pos, c);
                    *field = chars.iter().collect();
                    *cursor = pos + 1;
                }
                _ => {}
            }
        }
        Action::ModalInputBackspace => {
            match &mut app.modal {
                ModalState::ExportInput { path, cursor, .. } => {
                    if *cursor > 0 {
                        let pos = *cursor - 1;
                        let mut chars: Vec<char> = path.chars().collect();
                        if pos < chars.len() {
                            chars.remove(pos);
                        }
                        *path = chars.iter().collect();
                        *cursor = pos;
                    }
                }
                ModalState::ImportInput {
                    name,
                    install_dir,
                    tar_path,
                    active_field,
                    cursor,
                } => {
                    let field = match active_field {
                        0 => name,
                        1 => install_dir,
                        _ => tar_path,
                    };
                    if *cursor > 0 {
                        let pos = *cursor - 1;
                        let mut chars: Vec<char> = field.chars().collect();
                        if pos < chars.len() {
                            chars.remove(pos);
                        }
                        *field = chars.iter().collect();
                        *cursor = pos;
                    }
                }
                _ => {}
            }
        }
        Action::ModalInputLeft => {
            match &mut app.modal {
                ModalState::ExportInput { cursor, .. } => {
                    *cursor = cursor.saturating_sub(1);
                }
                ModalState::ImportInput { cursor, .. } => {
                    *cursor = cursor.saturating_sub(1);
                }
                _ => {}
            }
        }
        Action::ModalInputRight => {
            match &mut app.modal {
                ModalState::ExportInput { path, cursor, .. } => {
                    let len = path.chars().count();
                    *cursor = (*cursor + 1).min(len);
                }
                ModalState::ImportInput {
                    name,
                    install_dir,
                    tar_path,
                    active_field,
                    cursor,
                } => {
                    let field = match active_field {
                        0 => name.as_str(),
                        1 => install_dir.as_str(),
                        _ => tar_path.as_str(),
                    };
                    let len = field.chars().count();
                    *cursor = (*cursor + 1).min(len);
                }
                _ => {}
            }
        }
        Action::ModalInputTab => {
            if let ModalState::ImportInput {
                active_field,
                cursor,
                name,
                install_dir,
                tar_path,
                ..
            } = &mut app.modal
            {
                *active_field = (*active_field + 1) % 3;
                // Reset cursor to end of the newly active field.
                let new_field: &str = match active_field {
                    0 => name,
                    1 => install_dir,
                    _ => tar_path,
                };
                *cursor = new_field.chars().count();
            }
        }

        // ── Modal responses ───────────────────────────────────────────────────
        Action::ConfirmYes => {
            match app.modal.clone() {
                ModalState::Confirm { ref distro_name, .. } => {
                    let name = distro_name.clone();
                    let executor = app.executor.clone();
                    // Run unregister in a blocking task — it calls wsl.exe.
                    let _ = tokio::task::spawn_blocking(move || executor.unregister(&name)).await;
                    // Refresh list to reflect the removal.
                    let _ = app.refresh_distros();
                    app.modal = ModalState::None;
                }

                ModalState::InstallPicker {
                    online_distros,
                    list_state,
                } => {
                    // User selected a distro — start the install.
                    let idx = list_state.selected().unwrap_or(0);
                    if let Some(distro) = online_distros.get(idx) {
                        let distro_name = distro.name.clone();
                        let (tx, rx) = mpsc::channel::<(String, u16, bool)>(32);
                        app.install_rx = Some(rx);
                        app.modal = ModalState::InstallProgress {
                            distro_name: distro_name.clone(),
                            step: "Starting install...".to_string(),
                            percent: 0,
                            completed: false,
                        };

                        // Spawn the install in a blocking task, sending progress updates.
                        tokio::task::spawn_blocking(move || {
                            run_install_with_progress(&distro_name, tx);
                        });
                    } else {
                        app.modal = ModalState::None;
                    }
                }

                ModalState::InstallProgress { completed, .. }
                | ModalState::UpdateProgress { completed, .. } => {
                    // Only dismiss if completed; otherwise ignore.
                    if completed {
                        app.modal = ModalState::None;
                        let _ = app.refresh_distros();
                    }
                }

                ModalState::ExportInput {
                    ref distro_name,
                    ref path,
                    ..
                } => {
                    let name = distro_name.clone();
                    let export_path = path.clone();
                    let executor = app.executor.clone();
                    let _ =
                        tokio::task::spawn_blocking(move || executor.export_distro(&name, &export_path))
                            .await;
                    app.modal = ModalState::None;
                }

                ModalState::ImportInput {
                    ref name,
                    ref install_dir,
                    ref tar_path,
                    ..
                } => {
                    // Validate: all three fields must be non-empty.
                    if !name.is_empty() && !install_dir.is_empty() && !tar_path.is_empty() {
                        let n = name.clone();
                        let d = install_dir.clone();
                        let t = tar_path.clone();
                        let executor = app.executor.clone();
                        let _ =
                            tokio::task::spawn_blocking(move || executor.import_distro(&n, &d, &t))
                                .await;
                        let _ = app.refresh_distros();
                    }
                    app.modal = ModalState::None;
                }

                _ => {
                    app.modal = ModalState::None;
                }
            }
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

        Action::ExportDistro => {
            if let Some(name) = app.selected_name.clone() {
                let default_path = default_export_path(&name);
                let cursor = default_path.chars().count();
                app.modal = ModalState::ExportInput {
                    distro_name: name,
                    path: default_path,
                    cursor,
                };
            }
        }

        Action::ImportDistro => {
            let default_install_dir = "C:\\WSL\\new-distro".to_string();
            let cursor = 0;
            app.modal = ModalState::ImportInput {
                name: String::new(),
                install_dir: default_install_dir,
                tar_path: String::new(),
                active_field: 0,
                cursor,
            };
        }

        Action::InstallDistro => {
            // Fetch online distros list and show the picker.
            let executor = app.executor.clone();
            let result =
                tokio::task::spawn_blocking(move || executor.list_online()).await;

            match result {
                Ok(Ok(online_distros)) => {
                    use ratatui::widgets::ListState;
                    app.modal = ModalState::InstallPicker {
                        online_distros,
                        list_state: ListState::default().with_selected(Some(0)),
                    };
                }
                _ => {
                    // list_online failed — no-op, user sees no modal.
                }
            }
        }

        Action::UpdateWsl => {
            let (tx, rx) = mpsc::channel::<(String, u16, bool)>(8);
            app.install_rx = Some(rx);
            app.modal = ModalState::UpdateProgress {
                step: "Updating WSL...".to_string(),
                completed: false,
            };

            tokio::task::spawn_blocking(move || {
                run_update_with_progress(tx);
            });
        }
    }
}

/// Run `wsl.exe --install <name>` in a blocking context and send progress updates.
///
/// Uses a time-based approach (increment 1% per 500ms, capped at 90%) since
/// `wsl --install` does not provide machine-readable progress.  Jumps to 100%
/// on process exit.
fn run_install_with_progress(distro_name: &str, tx: mpsc::Sender<(String, u16, bool)>) {
    use std::process::Command;
    use std::time::{Duration, Instant};

    let mut child = match Command::new("wsl.exe")
        .args(["--install", distro_name, "--no-launch"])
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            // Fail gracefully — send a completion signal so the modal dismisses.
            let _ = tx.blocking_send(("Install failed".to_string(), 0, true));
            return;
        }
    };

    let mut percent: u16 = 0;
    let start = Instant::now();
    let total_estimate = Duration::from_secs(120); // rough upper bound

    loop {
        // Check if the process is still running.
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Process exited — jump to 100%.
                let _ = tx.blocking_send(("Complete".to_string(), 100, true));
                return;
            }
            Ok(None) => {
                // Still running — advance progress estimate.
                let elapsed = start.elapsed();
                let estimated = (elapsed.as_secs_f64() / total_estimate.as_secs_f64() * 90.0)
                    as u16;
                percent = estimated.min(90);

                let step = if percent < 30 {
                    "Downloading..."
                } else if percent < 70 {
                    "Installing..."
                } else {
                    "Configuring..."
                };

                if tx
                    .blocking_send((step.to_string(), percent, false))
                    .is_err()
                {
                    // Receiver dropped — abort.
                    let _ = child.kill();
                    return;
                }
            }
            Err(_) => {
                let _ = tx.blocking_send(("Install failed".to_string(), percent, true));
                return;
            }
        }

        std::thread::sleep(Duration::from_millis(500));
    }
}

/// Run `wsl.exe --update` in a blocking context and send a completion signal.
fn run_update_with_progress(tx: mpsc::Sender<(String, u16, bool)>) {
    let _ = tx.blocking_send(("Checking for updates...".to_string(), 0, false));

    let result = std::process::Command::new("wsl.exe")
        .arg("--update")
        .output();

    match result {
        Ok(output) if output.status.success() => {
            let _ = tx.blocking_send(("Update complete".to_string(), 100, true));
        }
        _ => {
            let _ = tx.blocking_send(("Update failed".to_string(), 0, true));
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

    // ── ExportDistro triggers ExportInput modal ────────────────────────────────

    #[tokio::test]
    async fn test_export_modal_triggers() {
        let config = make_config();
        let mut app = App::new(&config);
        app.selected_name = Some("Ubuntu".to_string());

        execute_action(&mut app, Action::ExportDistro).await;

        match &app.modal {
            ModalState::ExportInput { distro_name, .. } => {
                assert_eq!(distro_name, "Ubuntu", "ExportInput modal should target selected distro");
            }
            other => panic!("Expected ModalState::ExportInput, got {other:?}"),
        }
    }

    // ── ImportDistro triggers ImportInput modal ────────────────────────────────

    #[tokio::test]
    async fn test_import_modal_triggers() {
        let config = make_config();
        let mut app = App::new(&config);

        execute_action(&mut app, Action::ImportDistro).await;

        match &app.modal {
            ModalState::ImportInput { active_field, .. } => {
                assert_eq!(*active_field, 0, "ImportInput should start on field 0");
            }
            other => panic!("Expected ModalState::ImportInput, got {other:?}"),
        }
    }

    // ── ExportInput character handling ────────────────────────────────────────

    #[tokio::test]
    async fn test_export_input_char_handling() {
        let config = make_config();
        let mut app = App::new(&config);
        app.modal = ModalState::ExportInput {
            distro_name: "Ubuntu".to_string(),
            path: String::new(),
            cursor: 0,
        };

        execute_action(&mut app, Action::ModalInputChar('a')).await;
        execute_action(&mut app, Action::ModalInputChar('b')).await;
        execute_action(&mut app, Action::ModalInputChar('c')).await;

        match &app.modal {
            ModalState::ExportInput { path, cursor, .. } => {
                assert_eq!(path, "abc", "chars should append to export path");
                assert_eq!(*cursor, 3, "cursor should advance to 3");
            }
            other => panic!("Expected ModalState::ExportInput, got {other:?}"),
        }

        // Test backspace.
        execute_action(&mut app, Action::ModalInputBackspace).await;
        match &app.modal {
            ModalState::ExportInput { path, cursor, .. } => {
                assert_eq!(path, "ab", "backspace should remove last char");
                assert_eq!(*cursor, 2, "cursor should retreat to 2");
            }
            other => panic!("Expected ModalState::ExportInput, got {other:?}"),
        }
    }

    // ── ImportInput field cycling ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_import_field_cycling() {
        let config = make_config();
        let mut app = App::new(&config);
        app.modal = ModalState::ImportInput {
            name: String::new(),
            install_dir: "C:\\WSL".to_string(),
            tar_path: String::new(),
            active_field: 0,
            cursor: 0,
        };

        execute_action(&mut app, Action::ModalInputTab).await;
        match &app.modal {
            ModalState::ImportInput { active_field, .. } => {
                assert_eq!(*active_field, 1, "Tab should advance to field 1");
            }
            other => panic!("Expected ModalState::ImportInput, got {other:?}"),
        }

        execute_action(&mut app, Action::ModalInputTab).await;
        match &app.modal {
            ModalState::ImportInput { active_field, .. } => {
                assert_eq!(*active_field, 2, "Tab should advance to field 2");
            }
            other => panic!("Expected ModalState::ImportInput, got {other:?}"),
        }

        execute_action(&mut app, Action::ModalInputTab).await;
        match &app.modal {
            ModalState::ImportInput { active_field, .. } => {
                assert_eq!(*active_field, 0, "Tab should cycle back to field 0");
            }
            other => panic!("Expected ModalState::ImportInput, got {other:?}"),
        }
    }

    // ── install_rx channel assignment ─────────────────────────────────────────

    #[tokio::test]
    async fn test_install_rx_some_after_trigger() {
        let config = make_config();
        let mut app = App::new(&config);
        assert!(app.install_rx.is_none(), "install_rx should start as None");

        // Simulate the WSL update trigger (which sets install_rx without
        // requiring wsl.exe to be available).
        execute_action(&mut app, Action::UpdateWsl).await;

        assert!(
            app.install_rx.is_some(),
            "install_rx should be Some after UpdateWsl triggers a channel"
        );
    }

    // ── install_picker_to_progress_transition (simulated) ────────────────────

    #[tokio::test]
    async fn test_install_picker_to_progress_transition() {
        use ratatui::widgets::ListState;
        use wsl_core::OnlineDistro;

        let config = make_config();
        let mut app = App::new(&config);

        // Directly set picker state with one distro.
        app.modal = ModalState::InstallPicker {
            online_distros: vec![OnlineDistro {
                name: "Ubuntu".to_string(),
                friendly_name: "Ubuntu".to_string(),
            }],
            list_state: ListState::default().with_selected(Some(0)),
        };

        // ConfirmYes on the picker should transition to InstallProgress.
        execute_action(&mut app, Action::ConfirmYes).await;

        match &app.modal {
            ModalState::InstallProgress {
                distro_name,
                percent,
                completed,
                ..
            } => {
                assert_eq!(distro_name, "Ubuntu");
                assert_eq!(*percent, 0);
                assert!(!completed);
            }
            other => panic!("Expected ModalState::InstallProgress, got {other:?}"),
        }
    }
}
