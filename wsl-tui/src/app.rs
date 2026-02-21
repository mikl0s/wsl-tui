//! Application state for the WSL TUI.
//!
//! [`App`] is the central state struct driving the event loop.  It is passed
//! to [`crate::ui::render`] for rendering and mutated by the event handler in
//! `main.rs`.

use wsl_core::Config;

/// Central application state.
///
/// Owns the flags that control the event loop and UI rendering.  Created once
/// at startup from the loaded [`Config`] and lives for the duration of the run.
pub struct App {
    /// Whether the event loop should keep running.
    ///
    /// Set to `false` by [`App::quit`].  The loop in `run_app` exits on the
    /// next iteration.
    pub running: bool,

    /// Whether this is the first time the user has launched the application.
    ///
    /// Derived from `Config::first_run`; `true` when no `config.toml` existed
    /// before this run.
    ///
    /// Phase 1: stored for completeness; used by Phase 2 status bar and
    /// analytics.  Suppressing the dead_code lint so it does not appear in
    /// `-D warnings` CI runs before Phase 2 adds a consumer.
    #[allow(dead_code)]
    pub first_run: bool,

    /// Whether to show the welcome screen.
    ///
    /// Initially `true` iff `first_run` is `true`.  Pressing any key while the
    /// welcome screen is visible calls [`App::dismiss_welcome`] which sets this
    /// to `false`, showing the main placeholder UI.
    pub show_welcome: bool,
}

impl App {
    /// Create a new `App` from the loaded configuration.
    ///
    /// Sets `running = true`, `first_run` from `config.first_run`, and
    /// `show_welcome = config.first_run`.
    pub fn new(config: &Config) -> Self {
        Self {
            running: true,
            first_run: config.first_run,
            show_welcome: config.first_run,
        }
    }

    /// Signal the event loop to exit on the next iteration.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Dismiss the welcome screen.
    ///
    /// Called when the user presses any key while the welcome screen is
    /// visible.  After this call, `show_welcome` is `false` and subsequent
    /// renders show the main placeholder UI.
    pub fn dismiss_welcome(&mut self) {
        self.show_welcome = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal Config with the given `first_run` value.
    fn make_config(first_run: bool) -> Config {
        Config {
            storage: wsl_core::StorageMode::Auto,
            config_dir: std::path::PathBuf::new(),
            first_run,
        }
    }

    #[test]
    fn test_app_new_first_run() {
        let config = make_config(true);
        let app = App::new(&config);

        assert!(app.running, "app should start running");
        assert!(app.first_run, "first_run should be true");
        assert!(app.show_welcome, "welcome screen should show on first run");
    }

    #[test]
    fn test_app_new_not_first_run() {
        let config = make_config(false);
        let app = App::new(&config);

        assert!(app.running, "app should start running");
        assert!(!app.first_run, "first_run should be false");
        assert!(!app.show_welcome, "welcome screen should not show on subsequent runs");
    }

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
}
