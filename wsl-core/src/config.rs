use std::path::PathBuf;
use std::str::FromStr;

use serde::Deserialize;

use crate::error::CoreError;

/// Default keybinding values — used by serde `default` attributes.
fn default_quit() -> String {
    "q".into()
}
fn default_help() -> String {
    "?".into()
}
fn default_filter() -> String {
    "/".into()
}
fn default_up() -> String {
    "k".into()
}
fn default_down() -> String {
    "j".into()
}
fn default_left() -> String {
    "h".into()
}
fn default_right() -> String {
    "l".into()
}
fn default_attach() -> String {
    "enter".into()
}
fn default_start() -> String {
    "s".into()
}
fn default_stop() -> String {
    "t".into()
}
fn default_set_default() -> String {
    "d".into()
}
fn default_remove() -> String {
    "x".into()
}
fn default_export() -> String {
    "e".into()
}
fn default_import_distro() -> String {
    "i".into()
}

/// Raw keybinding strings from the `[keybindings]` config section.
///
/// Each field is a key string such as `"q"`, `"ctrl+d"`, `"enter"`, or `"f1"`.
/// Parsed into [`crossterm`] key codes by the TUI's `keybindings` module.
#[derive(Debug, Clone, Deserialize)]
pub struct RawKeybindings {
    /// Key to quit the application. Default: `"q"`.
    #[serde(default = "default_quit")]
    pub quit: String,
    /// Key to open the help overlay. Default: `"?"`.
    #[serde(default = "default_help")]
    pub help: String,
    /// Key to open the distro filter/search bar. Default: `"/"`.
    #[serde(default = "default_filter")]
    pub filter: String,
    /// Key to move selection up. Default: `"k"`.
    #[serde(default = "default_up")]
    pub up: String,
    /// Key to move selection down. Default: `"j"`.
    #[serde(default = "default_down")]
    pub down: String,
    /// Key to move focus left. Default: `"h"`.
    #[serde(default = "default_left")]
    pub left: String,
    /// Key to move focus right. Default: `"l"`.
    #[serde(default = "default_right")]
    pub right: String,
    /// Key to attach a shell to the selected distro. Default: `"enter"`.
    #[serde(default = "default_attach")]
    pub attach: String,
    /// Key to start the selected distro. Default: `"s"`.
    #[serde(default = "default_start")]
    pub start: String,
    /// Key to stop the selected distro. Default: `"t"`.
    #[serde(default = "default_stop")]
    pub stop: String,
    /// Key to set the selected distro as the WSL default. Default: `"d"`.
    #[serde(default = "default_set_default")]
    pub set_default: String,
    /// Key to remove/unregister the selected distro. Default: `"x"`.
    #[serde(default = "default_remove")]
    pub remove: String,
    /// Key to export the selected distro to a `.tar.gz`. Default: `"e"`.
    #[serde(default = "default_export")]
    pub export: String,
    /// Key to import a distro from a `.tar.gz` file. Default: `"i"`.
    #[serde(default = "default_import_distro")]
    pub import_distro: String,
}

impl Default for RawKeybindings {
    fn default() -> Self {
        Self {
            quit: default_quit(),
            help: default_help(),
            filter: default_filter(),
            up: default_up(),
            down: default_down(),
            left: default_left(),
            right: default_right(),
            attach: default_attach(),
            start: default_start(),
            stop: default_stop(),
            set_default: default_set_default(),
            remove: default_remove(),
            export: default_export(),
            import_distro: default_import_distro(),
        }
    }
}

/// The fully-commented default config that is written to disk on first run.
///
/// Acts as living documentation for all available settings. All options are
/// commented out so the defaults stay in code — users uncomment and change
/// only what they need.
pub const DEFAULT_CONFIG_TOML: &str = r#"# WSL TUI Configuration
# Location: ~/.wsl-tui/config.toml
#
# All settings below show their default values.
# Uncomment and modify to customize.
#
# Environment variable overrides (take precedence over file values):
#   WSL_TUI_STORAGE=auto|libsql|json

# ── Storage ─────────────────────────────────────────────────────────────────
#
# Storage backend for distro state, pack history, and settings.
#   "auto"   — Try libsql first; fall back to JSON if libsql fails to open.
#   "libsql" — Require libsql; refuse to start if unavailable.
#   "json"   — Always use JSON (no SQLite dependency).
#
# Can also be set via WSL_TUI_STORAGE environment variable.
# storage = "auto"

# ── Connection ───────────────────────────────────────────────────────────────
#
# (Phase 2) SSH/terminal connection preferences.
# [connection]
# default_terminal = "wt"   # "wt" (Windows Terminal), "external", "termius"
# termius_profile  = ""     # Termius profile name for WSL connections

# ── Theme ────────────────────────────────────────────────────────────────────
#
# (Phase 2) TUI colour theme.  Only Catppuccin Mocha is shipped in v1.
# [theme]
# name = "catppuccin-mocha"

# ── Keybindings ──────────────────────────────────────────────────────────────
#
# (Phase 2) Override default keybindings.
# [keybindings]
# quit         = "q"
# help         = "?"
# filter       = "/"
# up           = "k"
# down         = "j"
# left         = "h"
# right        = "l"
# attach       = "enter"
# start        = "s"
# stop         = "t"
# set_default  = "d"
# remove       = "x"
# export       = "e"
# import_distro = "i"
"#;

/// Which storage backend the application should use.
///
/// `Auto` tries libsql and transparently falls back to JSON on failure.
/// When a fallback occurs, the TUI status bar reflects the active backend.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StorageMode {
    /// Try libsql first; fall back to JSON if libsql fails to open.
    #[default]
    Auto,
    /// Require libsql. Refuse to start if libsql cannot be opened.
    Libsql,
    /// Always use JSON storage. No SQLite dependency.
    Json,
}

impl std::fmt::Display for StorageMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageMode::Auto => write!(f, "auto"),
            StorageMode::Libsql => write!(f, "libsql"),
            StorageMode::Json => write!(f, "json"),
        }
    }
}

impl FromStr for StorageMode {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(StorageMode::Auto),
            "libsql" => Ok(StorageMode::Libsql),
            "json" => Ok(StorageMode::Json),
            other => Err(CoreError::ConfigParse(format!(
                "invalid storage mode '{}': expected 'auto', 'libsql', or 'json'",
                other
            ))),
        }
    }
}

/// Raw deserialization target — only fields that appear in config.toml.
///
/// Computed fields (`config_dir`, `first_run`) are not deserialized; they are
/// set by `Config::load()` after reading the file.
#[derive(Debug, Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    storage: StorageMode,
    #[serde(default)]
    keybindings: RawKeybindings,
}

/// Application configuration.
///
/// Loaded from `~/.wsl-tui/config.toml` via [`Config::load`].  All fields
/// have sensible defaults so the binary starts correctly with no config file.
///
/// `WSL_TUI_*` environment variables take precedence over file values.
#[derive(Debug, Clone)]
pub struct Config {
    /// Which storage backend to use.
    pub storage: StorageMode,

    /// Resolved path to the config directory (`~/.wsl-tui/`).
    ///
    /// This directory is auto-created on first run and is guaranteed to exist
    /// after `Config::load()` returns `Ok(…)`.
    pub config_dir: PathBuf,

    /// `true` when `config.toml` did not exist before this run.
    ///
    /// The TUI uses this flag to decide whether to show the welcome screen.
    pub first_run: bool,

    /// Raw keybinding strings from the `[keybindings]` config section.
    ///
    /// The TUI's `keybindings` module parses these strings into crossterm key
    /// codes at startup via `KeyBindings::from_config`.
    pub keybindings: RawKeybindings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageMode::Auto,
            config_dir: PathBuf::new(),
            first_run: false,
            keybindings: RawKeybindings::default(),
        }
    }
}

impl Config {
    /// Load configuration from `~/.wsl-tui/config.toml`.
    ///
    /// Steps:
    /// 1. Resolve config dir via `dirs::home_dir()` + `.wsl-tui/`.
    /// 2. Create the directory if it does not exist.
    /// 3. If `config.toml` is absent, write a fully-commented default and set
    ///    `first_run = true`.
    /// 4. Parse the existing (or freshly-written) `config.toml`.
    /// 5. Apply `WSL_TUI_*` environment variable overrides.
    /// 6. Return the fully-resolved `Config`.
    pub fn load() -> Result<Self, CoreError> {
        let config_dir = dirs::home_dir()
            .ok_or(CoreError::NoHomeDir)?
            .join(".wsl-tui");

        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.toml");
        let first_run = !config_path.exists();

        if first_run {
            std::fs::write(&config_path, DEFAULT_CONFIG_TOML)?;
        }

        let text = std::fs::read_to_string(&config_path)?;
        let raw: RawConfig = toml::from_str(&text)?;

        let mut config = Config {
            storage: raw.storage,
            config_dir,
            first_run,
            keybindings: raw.keybindings,
        };

        // Environment variable overrides (locked decision — WSL_TUI_* prefix).
        if let Ok(val) = std::env::var("WSL_TUI_STORAGE") {
            config.storage = val.parse()?;
        }

        Ok(config)
    }

    /// Load from an explicit directory path.
    ///
    /// Used in tests to avoid touching `~/.wsl-tui/`.
    pub fn load_from(config_dir: PathBuf) -> Result<Self, CoreError> {
        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.toml");
        let first_run = !config_path.exists();

        if first_run {
            std::fs::write(&config_path, DEFAULT_CONFIG_TOML)?;
        }

        let text = std::fs::read_to_string(&config_path)?;
        let raw: RawConfig = toml::from_str(&text)?;

        let mut config = Config {
            storage: raw.storage,
            config_dir,
            first_run,
            keybindings: raw.keybindings,
        };

        if let Ok(val) = std::env::var("WSL_TUI_STORAGE") {
            config.storage = val.parse()?;
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serialise all tests that touch `WSL_TUI_*` env vars or call
    /// `Config::load_from`.  Rust runs tests in the same process with shared
    /// env, so any test that reads env vars is affected by tests that write
    /// them.  A single mutex ensures the env is clean for every test.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Helper: acquire the env lock, clear `WSL_TUI_STORAGE`, return the guard.
    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        let guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Clear any leftover env var from a previously panicked test.
        std::env::remove_var("WSL_TUI_STORAGE");
        guard
    }

    // ── StorageMode parsing ──────────────────────────────────────────────────

    #[test]
    fn test_default_config_storage_is_auto() {
        let cfg = Config::default();
        assert_eq!(cfg.storage, StorageMode::Auto);
    }

    #[test]
    fn test_storage_mode_from_str_auto() {
        let mode: StorageMode = "auto".parse().unwrap();
        assert_eq!(mode, StorageMode::Auto);
    }

    #[test]
    fn test_storage_mode_from_str_libsql() {
        let mode: StorageMode = "libsql".parse().unwrap();
        assert_eq!(mode, StorageMode::Libsql);
    }

    #[test]
    fn test_storage_mode_from_str_json() {
        let mode: StorageMode = "json".parse().unwrap();
        assert_eq!(mode, StorageMode::Json);
    }

    #[test]
    fn test_storage_mode_from_str_case_insensitive() {
        let mode: StorageMode = "LIBSQL".parse().unwrap();
        assert_eq!(mode, StorageMode::Libsql);

        let mode: StorageMode = "Auto".parse().unwrap();
        assert_eq!(mode, StorageMode::Auto);
    }

    #[test]
    fn test_storage_mode_from_str_invalid() {
        let err = "postgres".parse::<StorageMode>().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("invalid storage mode"));
        assert!(msg.contains("postgres"));
    }

    #[test]
    fn test_storage_mode_display() {
        assert_eq!(StorageMode::Auto.to_string(), "auto");
        assert_eq!(StorageMode::Libsql.to_string(), "libsql");
        assert_eq!(StorageMode::Json.to_string(), "json");
    }

    // ── Config loading ───────────────────────────────────────────────────────

    #[test]
    fn test_config_dir_creation() {
        let _guard = env_guard();
        let tmp = tempfile::tempdir().unwrap();
        let config_dir = tmp.path().join("wsl-tui-test");

        // Directory should not exist yet.
        assert!(!config_dir.exists());

        let cfg = Config::load_from(config_dir.clone()).unwrap();

        // Directory must be created.
        assert!(config_dir.exists());
        // config.toml must be written.
        assert!(config_dir.join("config.toml").exists());
        // first_run must be true on fresh load.
        assert!(cfg.first_run);
    }

    #[test]
    fn test_first_run_true_on_missing_config() {
        let _guard = env_guard();
        let tmp = tempfile::tempdir().unwrap();
        let config_dir = tmp.path().join("wsl-tui-test");

        let cfg = Config::load_from(config_dir).unwrap();
        assert!(cfg.first_run);
    }

    #[test]
    fn test_first_run_false_when_config_exists() {
        let _guard = env_guard();
        let tmp = tempfile::tempdir().unwrap();
        let config_dir = tmp.path().join("wsl-tui-test");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), DEFAULT_CONFIG_TOML).unwrap();

        let cfg = Config::load_from(config_dir).unwrap();
        assert!(!cfg.first_run);
    }

    #[test]
    fn test_default_storage_mode_is_auto_from_commented_toml() {
        let _guard = env_guard();
        let tmp = tempfile::tempdir().unwrap();
        let config_dir = tmp.path().join("wsl-tui-test");

        // Load once to write the default config.toml.
        let cfg = Config::load_from(config_dir.clone()).unwrap();
        assert_eq!(cfg.storage, StorageMode::Auto);

        // Load again; still Auto since the default TOML has storage commented out.
        let cfg2 = Config::load_from(config_dir).unwrap();
        assert_eq!(cfg2.storage, StorageMode::Auto);
    }

    #[test]
    fn test_storage_mode_from_toml_explicit() {
        let _guard = env_guard();
        let tmp = tempfile::tempdir().unwrap();
        let config_dir = tmp.path().join("wsl-tui-test");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), "storage = \"json\"\n").unwrap();

        let cfg = Config::load_from(config_dir).unwrap();
        assert_eq!(cfg.storage, StorageMode::Json);
    }

    // ── Environment variable overrides ───────────────────────────────────────

    #[test]
    fn test_env_override_storage() {
        let _guard = env_guard();

        let tmp = tempfile::tempdir().unwrap();
        let config_dir = tmp.path().join("wsl-tui-test");
        std::fs::create_dir_all(&config_dir).unwrap();
        // Config file says auto.
        std::fs::write(config_dir.join("config.toml"), "storage = \"auto\"\n").unwrap();

        std::env::set_var("WSL_TUI_STORAGE", "json");
        let result = Config::load_from(config_dir);
        std::env::remove_var("WSL_TUI_STORAGE");

        let cfg = result.unwrap();
        // Env var overrides the file value.
        assert_eq!(cfg.storage, StorageMode::Json);
    }

    #[test]
    fn test_env_override_invalid_value() {
        let _guard = env_guard();

        let tmp = tempfile::tempdir().unwrap();
        let config_dir = tmp.path().join("wsl-tui-test");

        std::env::set_var("WSL_TUI_STORAGE", "redis");
        let result = Config::load_from(config_dir);
        std::env::remove_var("WSL_TUI_STORAGE");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid storage mode"));
    }

    // ── DEFAULT_CONFIG_TOML ──────────────────────────────────────────────────

    #[test]
    fn test_default_config_toml_is_valid_toml() {
        // The default template must parse without error.
        let result = toml::from_str::<toml::Value>(DEFAULT_CONFIG_TOML);
        assert!(result.is_ok(), "DEFAULT_CONFIG_TOML is not valid TOML: {:?}", result);
    }

    // ── Keybindings ──────────────────────────────────────────────────────────

    #[test]
    fn test_config_default_keybindings() {
        let cfg = Config::default();
        assert_eq!(cfg.keybindings.quit, "q");
        assert_eq!(cfg.keybindings.help, "?");
        assert_eq!(cfg.keybindings.filter, "/");
        assert_eq!(cfg.keybindings.up, "k");
        assert_eq!(cfg.keybindings.down, "j");
        assert_eq!(cfg.keybindings.left, "h");
        assert_eq!(cfg.keybindings.right, "l");
        assert_eq!(cfg.keybindings.attach, "enter");
        assert_eq!(cfg.keybindings.start, "s");
        assert_eq!(cfg.keybindings.stop, "t");
        assert_eq!(cfg.keybindings.set_default, "d");
        assert_eq!(cfg.keybindings.remove, "x");
        assert_eq!(cfg.keybindings.export, "e");
        assert_eq!(cfg.keybindings.import_distro, "i");
    }

    #[test]
    fn test_config_custom_keybindings_from_toml() {
        let _guard = env_guard();
        let tmp = tempfile::tempdir().expect("tempdir created");
        let config_dir = tmp.path().join("wsl-tui-keybind-test");
        std::fs::create_dir_all(&config_dir).expect("dir created");

        // Write a TOML that overrides the quit keybinding.
        std::fs::write(
            config_dir.join("config.toml"),
            "[keybindings]\nquit = \"ctrl+q\"\n",
        )
        .expect("config written");

        let cfg = Config::load_from(config_dir).expect("config loaded");

        // Overridden key.
        assert_eq!(cfg.keybindings.quit, "ctrl+q");
        // Unspecified keys must still have their defaults.
        assert_eq!(cfg.keybindings.help, "?");
        assert_eq!(cfg.keybindings.up, "k");
        assert_eq!(cfg.keybindings.down, "j");
        assert_eq!(cfg.keybindings.attach, "enter");
    }
}
