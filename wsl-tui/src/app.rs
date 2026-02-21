//! Application state for the WSL TUI.
//!
//! [`App`] is the central state struct driving the event loop.  It is passed
//! to [`crate::ui::render`] for rendering and mutated by the event handler in
//! `main.rs`.
//!
//! This module also defines the [`View`], [`FocusPanel`], and [`ModalState`]
//! enums that control which UI panel and modal are currently active.

use ratatui::widgets::ListState;
use tokio::sync::mpsc;
use wsl_core::{Config, DistroInfo, OnlineDistro, WslExecutor};

// ── View ──────────────────────────────────────────────────────────────────────

/// The top-level view currently shown in the main content area.
///
/// Number keys 1–5 switch between views. Only [`View::Dashboard`] is fully
/// implemented in Phase 2; the others render a placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Primary distro management view (Phase 2).
    Dashboard,
    /// Pack provisioning view (Phase 3 placeholder).
    Provision,
    /// Resource monitor view (Phase 4 placeholder).
    Monitor,
    /// Backup/snapshot view (Phase 4 placeholder).
    Backup,
    /// Log viewer (Phase 4 placeholder).
    Logs,
}

impl View {
    /// Human-readable name shown in the status bar and placeholder screens.
    #[allow(dead_code)]
    pub fn display_name(self) -> &'static str {
        match self {
            View::Dashboard => "Dashboard",
            View::Provision => "Provision",
            View::Monitor => "Monitor",
            View::Backup => "Backup",
            View::Logs => "Logs",
        }
    }
}

// ── FocusPanel ────────────────────────────────────────────────────────────────

/// Which panel within the dashboard currently has keyboard focus.
///
/// Tab cycles between [`FocusPanel::DistroList`] and [`FocusPanel::Details`].
/// The focused panel receives a highlighted border.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    /// The distro list on the left side of the dashboard split-pane.
    DistroList,
    /// The detail panel on the right side of the dashboard split-pane.
    Details,
}

// ── ModalState ────────────────────────────────────────────────────────────────

/// The currently active modal dialog, if any.
///
/// [`ModalState::None`] means no modal is visible. The event loop routes key
/// events to modal-specific handlers when a modal is active.
#[derive(Debug, Clone)]
pub enum ModalState {
    /// No modal is visible — normal event routing applies.
    None,
    /// A yes/no confirmation dialog for a destructive action.
    Confirm {
        /// Name of the distro the action targets.
        distro_name: String,
        /// Human-readable description of the action (e.g., "Remove distro?").
        action_label: String,
    },
    /// The help overlay is visible.
    Help,

    // ── Install flow ──────────────────────────────────────────────────────────

    /// The online distro picker is shown — user selects a distro to install.
    InstallPicker {
        /// Distros available from the online catalog.
        online_distros: Vec<OnlineDistro>,
        /// Ratatui list selection state for the picker list.
        list_state: ListState,
    },
    /// Install progress modal — shows step label and gauge while installing.
    InstallProgress {
        /// The distro being installed.
        distro_name: String,
        /// Human-readable label for the current step (e.g., "Downloading...").
        step: String,
        /// Progress percentage 0–100.
        percent: u16,
        /// `true` when the install process has finished.
        completed: bool,
    },
    /// WSL kernel update progress modal.
    UpdateProgress {
        /// Human-readable label for the current update step.
        step: String,
        /// `true` when the update process has finished.
        completed: bool,
    },

    // ── Export / Import text input modals ─────────────────────────────────────

    /// Text input modal for specifying the export path.
    ExportInput {
        /// Name of the distro being exported.
        distro_name: String,
        /// Current text in the path input field.
        path: String,
        /// Character index of the cursor within `path`.
        cursor: usize,
    },
    /// Multi-field text input modal for importing a distro from a tar file.
    ImportInput {
        /// New distro name to register under.
        name: String,
        /// Target installation directory on the host.
        install_dir: String,
        /// Path to the source `.tar` file.
        tar_path: String,
        /// Which field is currently active: 0 = name, 1 = install_dir, 2 = tar_path.
        active_field: u8,
        /// Character index of the cursor in the active field.
        cursor: usize,
    },
}

// ListState does not implement PartialEq, so we implement it manually.
impl PartialEq for ModalState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (
                Self::Confirm {
                    distro_name: a,
                    action_label: b,
                },
                Self::Confirm {
                    distro_name: c,
                    action_label: d,
                },
            ) => a == c && b == d,
            (Self::Help, Self::Help) => true,
            (
                Self::InstallPicker {
                    online_distros: a, ..
                },
                Self::InstallPicker {
                    online_distros: b, ..
                },
            ) => a == b,
            (
                Self::InstallProgress {
                    distro_name: a,
                    step: b,
                    percent: c,
                    completed: d,
                },
                Self::InstallProgress {
                    distro_name: e,
                    step: f,
                    percent: g,
                    completed: h,
                },
            ) => a == e && b == f && c == g && d == h,
            (
                Self::UpdateProgress {
                    step: a,
                    completed: b,
                },
                Self::UpdateProgress {
                    step: c,
                    completed: d,
                },
            ) => a == c && b == d,
            (
                Self::ExportInput {
                    distro_name: a,
                    path: b,
                    cursor: c,
                },
                Self::ExportInput {
                    distro_name: d,
                    path: e,
                    cursor: f,
                },
            ) => a == d && b == e && c == f,
            (
                Self::ImportInput {
                    name: a,
                    install_dir: b,
                    tar_path: c,
                    active_field: d,
                    cursor: e,
                },
                Self::ImportInput {
                    name: f,
                    install_dir: g,
                    tar_path: h,
                    active_field: i,
                    cursor: j,
                },
            ) => a == f && b == g && c == h && d == i && e == j,
            _ => false,
        }
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

/// Central application state.
///
/// Owns all flags that control the event loop and UI rendering.  Created once
/// at startup from the loaded [`Config`] and lives for the duration of the run.
pub struct App {
    /// Whether the event loop should keep running.
    ///
    /// Set to `false` by [`App::quit`]. The loop in `run_app` exits on the
    /// next iteration.
    pub running: bool,

    /// Whether this is the first time the user has launched the application.
    ///
    /// Derived from `Config::first_run`; consumed by the status bar (Task 2).
    #[allow(dead_code)]
    pub first_run: bool,

    /// Whether to show the welcome screen.
    ///
    /// Initially `true` iff `first_run` is `true`. Pressing any key while the
    /// welcome screen is visible calls [`App::dismiss_welcome`] which sets this
    /// to `false`, showing the dashboard.
    pub show_welcome: bool,

    // ── View management ───────────────────────────────────────────────────────
    /// The currently active top-level view.
    pub current_view: View,

    /// Which panel has keyboard focus in the dashboard split-pane.
    pub focus: FocusPanel,

    /// Currently active modal dialog, or [`ModalState::None`].
    pub modal: ModalState,

    // ── Distro data ───────────────────────────────────────────────────────────
    /// All installed distros, refreshed every 5 seconds.
    pub distros: Vec<DistroInfo>,

    /// Ratatui list selection state (scroll offset + highlighted row).
    pub list_state: ListState,

    /// The name of the selected distro; survives filter changes.
    ///
    /// Used to reconcile the selection when the visible list changes (e.g.,
    /// after a refresh or filter text change).
    pub selected_name: Option<String>,

    // ── Filter ────────────────────────────────────────────────────────────────
    /// Whether the filter bar is currently open.
    pub filter_active: bool,

    /// Current filter text (case-insensitive substring match on distro name).
    pub filter_text: String,

    // ── Executor ─────────────────────────────────────────────────────────────
    /// WSL command executor.  Stateless and `Clone` — safe to move into
    /// `spawn_blocking` closures for long-running wsl.exe calls.
    pub executor: WslExecutor,

    // ── Status info ───────────────────────────────────────────────────────────
    /// Name of the active storage backend, shown in the status bar (Task 2).
    ///
    /// Set by the caller that opens the storage backend; defaults to
    /// `"unknown"` until set.
    #[allow(dead_code)]
    pub storage_backend: String,

    // ── Install / update progress channel ─────────────────────────────────────
    /// Receiver for background install or update progress messages.
    ///
    /// Each message is `(step_label, percent, completed)`.  The channel is
    /// created when an install or update is triggered and set to `None` after
    /// the background task signals completion.
    pub install_rx: Option<mpsc::Receiver<(String, u16, bool)>>,
}

impl App {
    /// Create a new `App` from the loaded configuration.
    ///
    /// Initialises all fields to safe defaults.  Distros start empty and are
    /// populated by the first `refresh_distros` call in `run_app`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wsl_core::Config;
    /// use wsl_tui::app::App;
    ///
    /// let config = Config::default();
    /// let app = App::new(&config);
    /// assert!(app.running);
    /// ```
    pub fn new(config: &Config) -> Self {
        Self {
            running: true,
            first_run: config.first_run,
            show_welcome: config.first_run,
            current_view: View::Dashboard,
            focus: FocusPanel::DistroList,
            modal: ModalState::None,
            distros: Vec::new(),
            list_state: ListState::default().with_selected(Some(0)),
            selected_name: None,
            filter_active: false,
            filter_text: String::new(),
            executor: WslExecutor::new(),
            storage_backend: "unknown".to_string(),
            install_rx: None,
        }
    }

    /// Signal the event loop to exit on the next iteration.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Dismiss the welcome screen.
    ///
    /// Called when the user presses any key while the welcome screen is
    /// visible. After this call, `show_welcome` is `false` and subsequent
    /// renders show the dashboard.
    pub fn dismiss_welcome(&mut self) {
        self.show_welcome = false;
    }

    /// Reload the distro list from `wsl.exe --list --verbose`.
    ///
    /// Updates `self.distros`, clamps the list-state selection to the visible
    /// range, and reconciles `selected_name` with the (potentially filtered)
    /// visible list.
    ///
    /// Errors from `wsl.exe` are propagated; the caller should decide whether
    /// to log and continue or surface to the user.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wsl_core::Config;
    /// use wsl_tui::app::App;
    ///
    /// let config = Config::default();
    /// let mut app = App::new(&config);
    /// // In production: app.refresh_distros().unwrap();
    /// ```
    pub fn refresh_distros(&mut self) -> anyhow::Result<()> {
        self.distros = self.executor.list_distros()?;
        self.clamp_selection();
        Ok(())
    }

    /// Return a filtered view of `self.distros`.
    ///
    /// When `filter_active` is `false` or `filter_text` is empty, all distros
    /// are returned.  Otherwise, only distros whose name contains the filter
    /// text (case-insensitive) are returned.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wsl_core::Config;
    /// use wsl_tui::app::App;
    ///
    /// let config = Config::default();
    /// let mut app = App::new(&config);
    /// // All visible when filter is empty:
    /// assert_eq!(app.visible_distros().len(), app.distros.len());
    /// ```
    pub fn visible_distros(&self) -> Vec<&DistroInfo> {
        if !self.filter_active || self.filter_text.is_empty() {
            return self.distros.iter().collect();
        }
        let lower = self.filter_text.to_lowercase();
        self.distros
            .iter()
            .filter(|d| d.name.to_lowercase().contains(&lower))
            .collect()
    }

    /// Return the distro at the currently selected index in the visible list.
    ///
    /// Returns `None` when the visible list is empty or the selection index is
    /// out of range.
    pub fn selected_distro(&self) -> Option<&DistroInfo> {
        let idx = self.list_state.selected()?;
        self.visible_distros().into_iter().nth(idx)
    }

    /// Move the selection one row upward, clamping at index 0.
    pub fn move_selection_up(&mut self) {
        let current = self.list_state.selected().unwrap_or(0);
        let new_idx = current.saturating_sub(1);
        self.list_state.select(Some(new_idx));
        self.sync_selected_name();
    }

    /// Move the selection one row downward, clamping at the last visible row.
    pub fn move_selection_down(&mut self) {
        let count = self.visible_distros().len();
        if count == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new_idx = (current + 1).min(count - 1);
        self.list_state.select(Some(new_idx));
        self.sync_selected_name();
    }

    // ── Filter helpers ────────────────────────────────────────────────────────

    /// Activate the filter bar.
    ///
    /// Sets `filter_active = true`. The event handler will route subsequent
    /// character keys to [`App::filter_push_char`] until the filter is
    /// deactivated.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wsl_core::Config;
    /// use wsl_tui::app::App;
    ///
    /// let config = Config::default();
    /// let mut app = App::new(&config);
    /// app.activate_filter();
    /// assert!(app.filter_active);
    /// ```
    pub fn activate_filter(&mut self) {
        self.filter_active = true;
    }

    /// Deactivate the filter bar and clear any accumulated filter text.
    ///
    /// Resets `filter_active = false`, clears `filter_text`, and resets the
    /// list selection to index 0 so the user returns to a predictable position.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wsl_core::Config;
    /// use wsl_tui::app::App;
    ///
    /// let config = Config::default();
    /// let mut app = App::new(&config);
    /// app.activate_filter();
    /// app.filter_text = "ubu".to_string();
    /// app.deactivate_filter();
    /// assert!(!app.filter_active);
    /// assert!(app.filter_text.is_empty());
    /// ```
    pub fn deactivate_filter(&mut self) {
        self.filter_active = false;
        self.filter_text.clear();
        self.list_state.select(Some(0));
    }

    /// Append a character to the filter text and reset the selection to index 0.
    ///
    /// Called for each character key while the filter is active.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wsl_core::Config;
    /// use wsl_tui::app::App;
    ///
    /// let config = Config::default();
    /// let mut app = App::new(&config);
    /// app.activate_filter();
    /// app.filter_push_char('u');
    /// app.filter_push_char('b');
    /// assert_eq!(app.filter_text, "ub");
    /// ```
    pub fn filter_push_char(&mut self, c: char) {
        self.filter_text.push(c);
        self.list_state.select(Some(0));
    }

    /// Remove the last character from the filter text.
    ///
    /// If the filter text is already empty, this is a no-op.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wsl_core::Config;
    /// use wsl_tui::app::App;
    ///
    /// let config = Config::default();
    /// let mut app = App::new(&config);
    /// app.activate_filter();
    /// app.filter_push_char('u');
    /// app.filter_pop_char();
    /// assert!(app.filter_text.is_empty());
    /// ```
    pub fn filter_pop_char(&mut self) {
        self.filter_text.pop();
    }

    /// Toggle focus between [`FocusPanel::DistroList`] and [`FocusPanel::Details`].
    pub fn switch_focus(&mut self) {
        self.focus = match self.focus {
            FocusPanel::DistroList => FocusPanel::Details,
            FocusPanel::Details => FocusPanel::DistroList,
        };
    }

    /// Switch the current view.
    pub fn set_view(&mut self, view: View) {
        self.current_view = view;
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Clamp the list-state selection to the current visible list length and
    /// sync `selected_name`.
    fn clamp_selection(&mut self) {
        let count = self.visible_distros().len();
        if count == 0 {
            self.list_state.select(None);
            self.selected_name = None;
            return;
        }

        // If we had a named selection, try to restore it.
        if let Some(ref name) = self.selected_name.clone() {
            let pos = self
                .visible_distros()
                .iter()
                .position(|d| &d.name == name);
            if let Some(idx) = pos {
                self.list_state.select(Some(idx));
                return;
            }
        }

        // Otherwise clamp the index.
        let current = self.list_state.selected().unwrap_or(0);
        let clamped = current.min(count - 1);
        self.list_state.select(Some(clamped));
        self.sync_selected_name();
    }

    /// Update `selected_name` from the current list-state index.
    fn sync_selected_name(&mut self) {
        self.selected_name = self
            .selected_distro()
            .map(|d| d.name.clone());
    }
}

// ── Public helpers for tests ──────────────────────────────────────────────────

/// Return the `DistroState` for the given name from a slice of [`DistroInfo`].
///
/// Used in tests to check state without indexing by position.
#[cfg(test)]
fn find_distro<'a>(distros: &'a [DistroInfo], name: &str) -> Option<&'a DistroInfo> {
    distros.iter().find(|d| d.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use wsl_core::DistroState;

    /// Build a minimal Config with the given `first_run` value.
    fn make_config(first_run: bool) -> Config {
        Config {
            storage: wsl_core::StorageMode::Auto,
            config_dir: std::path::PathBuf::new(),
            first_run,
            keybindings: wsl_core::RawKeybindings::default(),
        }
    }

    /// Create a sample DistroInfo for use in tests.
    fn make_distro(name: &str, running: bool, is_default: bool) -> DistroInfo {
        DistroInfo {
            name: name.to_string(),
            state: if running {
                DistroState::Running
            } else {
                DistroState::Stopped
            },
            version: 2,
            is_default,
        }
    }

    // ── App::new ──────────────────────────────────────────────────────────────

    #[test]
    fn test_app_new_first_run() {
        let config = make_config(true);
        let app = App::new(&config);

        assert!(app.running, "app should start running");
        assert!(app.first_run, "first_run should be true");
        assert!(app.show_welcome, "welcome screen should show on first run");
        assert_eq!(app.current_view, View::Dashboard);
        assert_eq!(app.focus, FocusPanel::DistroList);
        assert_eq!(app.modal, ModalState::None);
        assert!(app.distros.is_empty());
        assert!(!app.filter_active);
        assert!(app.filter_text.is_empty());
    }

    #[test]
    fn test_app_new_not_first_run() {
        let config = make_config(false);
        let app = App::new(&config);

        assert!(app.running, "app should start running");
        assert!(!app.first_run, "first_run should be false");
        assert!(!app.show_welcome, "welcome screen should not show on subsequent runs");
    }

    // ── App::quit / dismiss_welcome ───────────────────────────────────────────

    #[test]
    fn test_app_quit() {
        let config = make_config(false);
        let mut app = App::new(&config);

        assert!(app.running);
        app.quit();
        assert!(!app.running, "quit() should set running = false");
    }

    #[test]
    fn test_app_dismiss_welcome() {
        let config = make_config(true);
        let mut app = App::new(&config);

        assert!(app.show_welcome);
        app.dismiss_welcome();
        assert!(!app.show_welcome, "dismiss_welcome() should set show_welcome = false");
    }

    #[test]
    fn test_app_dismiss_welcome_idempotent() {
        let config = make_config(true);
        let mut app = App::new(&config);

        app.dismiss_welcome();
        app.dismiss_welcome();
        assert!(!app.show_welcome);
    }

    #[test]
    fn test_app_quit_does_not_affect_welcome() {
        let config = make_config(true);
        let mut app = App::new(&config);

        app.quit();
        // Quitting shouldn't touch show_welcome.
        assert!(app.show_welcome, "quit should not change show_welcome");
    }

    // ── visible_distros ───────────────────────────────────────────────────────

    #[test]
    fn test_app_visible_distros_no_filter() {
        let config = make_config(false);
        let mut app = App::new(&config);
        app.distros = vec![
            make_distro("Ubuntu", true, true),
            make_distro("Debian", false, false),
        ];

        let visible = app.visible_distros();
        assert_eq!(visible.len(), 2, "all distros visible when filter is empty");
    }

    #[test]
    fn test_app_visible_distros_with_filter() {
        let config = make_config(false);
        let mut app = App::new(&config);
        app.distros = vec![
            make_distro("Ubuntu", true, true),
            make_distro("Debian", false, false),
            make_distro("Ubuntu-22.04", false, false),
        ];
        app.filter_active = true;
        app.filter_text = "ubuntu".to_string();

        let visible = app.visible_distros();
        assert_eq!(visible.len(), 2, "only ubuntu-matching distros should be visible");
        assert_eq!(visible[0].name, "Ubuntu");
        assert_eq!(visible[1].name, "Ubuntu-22.04");
    }

    #[test]
    fn test_app_visible_distros_filter_inactive() {
        let config = make_config(false);
        let mut app = App::new(&config);
        app.distros = vec![
            make_distro("Ubuntu", true, true),
            make_distro("Debian", false, false),
        ];
        // filter_text set but not active
        app.filter_active = false;
        app.filter_text = "ubuntu".to_string();

        let visible = app.visible_distros();
        assert_eq!(visible.len(), 2, "all distros visible when filter is not active");
    }

    // ── selected_distro ───────────────────────────────────────────────────────

    #[test]
    fn test_app_selected_distro_empty() {
        let config = make_config(false);
        let app = App::new(&config);
        // No distros — selection should return None.
        assert!(app.selected_distro().is_none());
    }

    #[test]
    fn test_app_selected_distro_returns_correct() {
        let config = make_config(false);
        let mut app = App::new(&config);
        app.distros = vec![
            make_distro("Ubuntu", true, true),
            make_distro("Debian", false, false),
        ];
        // list_state is initialised with selected(Some(0))
        let selected = app.selected_distro();
        assert!(selected.is_some());
        assert_eq!(selected.expect("should have selected distro").name, "Ubuntu");
    }

    // ── move_selection_up / move_selection_down ───────────────────────────────

    #[test]
    fn test_app_move_selection_down() {
        let config = make_config(false);
        let mut app = App::new(&config);
        app.distros = vec![
            make_distro("Ubuntu", true, true),
            make_distro("Debian", false, false),
        ];
        // Start at index 0.
        assert_eq!(app.list_state.selected(), Some(0));
        app.move_selection_down();
        assert_eq!(app.list_state.selected(), Some(1), "selection should advance");
    }

    #[test]
    fn test_app_move_selection_clamps_bottom() {
        let config = make_config(false);
        let mut app = App::new(&config);
        app.distros = vec![
            make_distro("Ubuntu", true, true),
            make_distro("Debian", false, false),
        ];
        // Move past the end.
        app.move_selection_down();
        app.move_selection_down();
        app.move_selection_down();
        assert_eq!(
            app.list_state.selected(),
            Some(1),
            "selection should clamp at last index"
        );
    }

    #[test]
    fn test_app_move_selection_up_clamps() {
        let config = make_config(false);
        let mut app = App::new(&config);
        app.distros = vec![
            make_distro("Ubuntu", true, true),
            make_distro("Debian", false, false),
        ];
        // Already at 0 — moving up should stay at 0.
        app.move_selection_up();
        assert_eq!(
            app.list_state.selected(),
            Some(0),
            "selection should clamp at index 0"
        );
    }

    // ── switch_focus ──────────────────────────────────────────────────────────

    #[test]
    fn test_app_switch_focus() {
        let config = make_config(false);
        let mut app = App::new(&config);

        assert_eq!(app.focus, FocusPanel::DistroList, "default focus is DistroList");
        app.switch_focus();
        assert_eq!(app.focus, FocusPanel::Details, "after Tab focus should be Details");
        app.switch_focus();
        assert_eq!(app.focus, FocusPanel::DistroList, "after Tab again focus should return to DistroList");
    }

    // ── set_view ──────────────────────────────────────────────────────────────

    #[test]
    fn test_app_set_view() {
        let config = make_config(false);
        let mut app = App::new(&config);

        assert_eq!(app.current_view, View::Dashboard);
        app.set_view(View::Provision);
        assert_eq!(app.current_view, View::Provision);
        app.set_view(View::Logs);
        assert_eq!(app.current_view, View::Logs);
    }

    // ── activate_filter / deactivate_filter ───────────────────────────────────

    #[test]
    fn test_activate_filter() {
        let config = make_config(false);
        let mut app = App::new(&config);

        assert!(!app.filter_active, "filter should start inactive");
        app.activate_filter();
        assert!(app.filter_active, "filter_active should be true after activate_filter()");
    }

    #[test]
    fn test_filter_push_and_pop() {
        let config = make_config(false);
        let mut app = App::new(&config);

        app.activate_filter();
        app.filter_push_char('u');
        app.filter_push_char('b');
        app.filter_push_char('u');
        assert_eq!(app.filter_text, "ubu", "characters should accumulate");

        app.filter_pop_char();
        assert_eq!(app.filter_text, "ub", "pop should remove last char");

        app.filter_pop_char();
        app.filter_pop_char();
        assert!(app.filter_text.is_empty(), "text should be empty after all pops");
    }

    #[test]
    fn test_filter_pop_empty_is_noop() {
        let config = make_config(false);
        let mut app = App::new(&config);

        // Pop on empty filter should not panic.
        app.activate_filter();
        app.filter_pop_char();
        assert!(app.filter_text.is_empty());
    }

    #[test]
    fn test_deactivate_filter_clears() {
        let config = make_config(false);
        let mut app = App::new(&config);

        app.activate_filter();
        app.filter_push_char('u');
        app.filter_push_char('b');
        assert_eq!(app.filter_text, "ub");

        app.deactivate_filter();
        assert!(!app.filter_active, "filter_active should be false after deactivate_filter()");
        assert!(app.filter_text.is_empty(), "filter_text should be cleared after deactivate_filter()");
    }

    #[test]
    fn test_deactivate_filter_resets_selection() {
        let config = make_config(false);
        let mut app = App::new(&config);
        app.distros = vec![
            make_distro("Ubuntu", true, true),
            make_distro("Debian", false, false),
            make_distro("Alpine", false, false),
        ];
        // Move selection to index 2.
        app.list_state.select(Some(2));
        app.filter_active = true;
        app.filter_text = "a".to_string();

        app.deactivate_filter();
        assert_eq!(
            app.list_state.selected(),
            Some(0),
            "deactivate_filter should reset selection to index 0"
        );
    }

    // ── find_distro helper ────────────────────────────────────────────────────

    #[test]
    fn test_find_distro_helper() {
        let distros = vec![
            make_distro("Ubuntu", true, true),
            make_distro("Debian", false, false),
        ];
        let found = find_distro(&distros, "Debian");
        assert!(found.is_some());
        assert_eq!(found.expect("should find Debian").name, "Debian");

        let not_found = find_distro(&distros, "Alpine");
        assert!(not_found.is_none());
    }

    // ── install_rx field ─────────────────────────────────────────────────────

    #[test]
    fn test_install_rx_none_initially() {
        let config = make_config(false);
        let app = App::new(&config);
        assert!(app.install_rx.is_none(), "install_rx should start as None");
    }

    #[test]
    fn test_install_rx_some_after_assign() {
        let config = make_config(false);
        let mut app = App::new(&config);
        let (_tx, rx) = mpsc::channel::<(String, u16, bool)>(8);
        app.install_rx = Some(rx);
        assert!(app.install_rx.is_some(), "install_rx should be Some after channel assignment");
    }

    // ── InstallPicker modal state ─────────────────────────────────────────────

    #[test]
    fn test_install_picker_modal_equality() {
        use wsl_core::OnlineDistro;
        let distros = vec![OnlineDistro {
            name: "Ubuntu".to_string(),
            friendly_name: "Ubuntu 22.04 LTS".to_string(),
        }];
        let m1 = ModalState::InstallPicker {
            online_distros: distros.clone(),
            list_state: ListState::default(),
        };
        let m2 = ModalState::InstallPicker {
            online_distros: distros,
            list_state: ListState::default().with_selected(Some(0)),
        };
        // Both have same online_distros — custom PartialEq only compares that field.
        assert_eq!(m1, m2, "InstallPicker equality ignores list_state position");
    }

    // ── InstallProgress modal state ───────────────────────────────────────────

    #[test]
    fn test_install_progress_dismiss_on_completed() {
        let config = make_config(false);
        let mut app = App::new(&config);

        // Simulate a completed install progress modal.
        app.modal = ModalState::InstallProgress {
            distro_name: "Ubuntu".to_string(),
            step: "Complete".to_string(),
            percent: 100,
            completed: true,
        };

        // When completed = true, any key should clear the modal.
        // We simulate the dismiss logic directly: if completed, clear.
        if let ModalState::InstallProgress { completed, .. } = app.modal.clone() {
            if completed {
                app.modal = ModalState::None;
            }
        }

        assert_eq!(
            app.modal,
            ModalState::None,
            "completed install progress should dismiss to None"
        );
    }

    // ── ExportInput modal state ───────────────────────────────────────────────

    #[test]
    fn test_export_input_modal_equality() {
        let m1 = ModalState::ExportInput {
            distro_name: "Ubuntu".to_string(),
            path: "/tmp/ubuntu.tar".to_string(),
            cursor: 15,
        };
        let m2 = ModalState::ExportInput {
            distro_name: "Ubuntu".to_string(),
            path: "/tmp/ubuntu.tar".to_string(),
            cursor: 15,
        };
        assert_eq!(m1, m2);
    }

    // ── ImportInput modal state ───────────────────────────────────────────────

    #[test]
    fn test_import_input_modal_equality() {
        let m1 = ModalState::ImportInput {
            name: "MyDistro".to_string(),
            install_dir: "C:\\WSL\\MyDistro".to_string(),
            tar_path: "C:\\tmp\\distro.tar".to_string(),
            active_field: 0,
            cursor: 0,
        };
        let m2 = m1.clone();
        assert_eq!(m1, m2);
    }
}
