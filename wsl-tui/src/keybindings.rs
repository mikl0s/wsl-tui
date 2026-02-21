//! Configurable keybindings for WSL TUI.
//!
//! Parses key string notation (e.g. `"q"`, `"ctrl+d"`, `"enter"`, `"f1"`) into
//! crossterm [`KeyCode`] + [`KeyModifiers`] pairs, then groups them into a
//! [`KeyBindings`] struct that is constructed once at startup from
//! [`wsl_core::Config`].
//!
//! # Usage
//!
//! ```rust,no_run
//! use wsl_core::Config;
//! use wsl_tui::keybindings::{KeyBindings, KeyAction};
//!
//! let config = Config::default();
//! let kb = KeyBindings::from_config(&config);
//! // In an event loop:
//! // if kb.matches(&key_event, KeyAction::Quit) { app.quit(); }
//! ```

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use wsl_core::Config;

/// All user-facing actions that can be triggered by a key press.
///
/// One variant per configurable keybinding in [`KeyBindings`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Quit the application.
    Quit,
    /// Open the help overlay.
    Help,
    /// Open the distro filter / search bar.
    Filter,
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Move focus to the left panel.
    Left,
    /// Move focus to the right panel.
    Right,
    /// Attach a shell to the selected distro.
    Attach,
    /// Start the selected distro.
    Start,
    /// Stop the selected distro.
    Stop,
    /// Set the selected distro as the WSL default.
    SetDefault,
    /// Remove / unregister the selected distro.
    Remove,
    /// Export the selected distro to a `.tar.gz` file.
    Export,
    /// Import a distro from a `.tar.gz` file.
    Import,
}

/// Parsed keybindings, ready for use in the event loop.
///
/// Each field holds the `(KeyCode, KeyModifiers)` pair for one action.
/// Constructed once at startup via [`KeyBindings::from_config`].
///
/// # Example
///
/// ```rust,no_run
/// use wsl_core::Config;
/// use wsl_tui::keybindings::{KeyBindings, KeyAction};
///
/// let kb = KeyBindings::from_config(&Config::default());
/// ```
#[derive(Debug, Clone)]
pub struct KeyBindings {
    quit: (KeyCode, KeyModifiers),
    help: (KeyCode, KeyModifiers),
    filter: (KeyCode, KeyModifiers),
    up: (KeyCode, KeyModifiers),
    down: (KeyCode, KeyModifiers),
    left: (KeyCode, KeyModifiers),
    right: (KeyCode, KeyModifiers),
    attach: (KeyCode, KeyModifiers),
    start: (KeyCode, KeyModifiers),
    stop: (KeyCode, KeyModifiers),
    set_default: (KeyCode, KeyModifiers),
    remove: (KeyCode, KeyModifiers),
    export: (KeyCode, KeyModifiers),
    import: (KeyCode, KeyModifiers),
}

impl KeyBindings {
    /// Construct [`KeyBindings`] from the application [`Config`].
    ///
    /// Parses every keybinding string with [`parse_key_str`]. Panics at startup
    /// if any configured key string is invalid — this is intentional config
    /// validation that fails fast rather than silently ignoring a bad binding.
    ///
    /// # Panics
    ///
    /// Panics when a keybinding string in `config.keybindings` cannot be
    /// parsed by [`parse_key_str`] (e.g. `"foobar"`).
    pub fn from_config(config: &Config) -> Self {
        let kb = &config.keybindings;

        let parse = |s: &str| {
            parse_key_str(s)
                .unwrap_or_else(|| panic!("keybinding '{}' is not a valid key string", s))
        };

        Self {
            quit: parse(&kb.quit),
            help: parse(&kb.help),
            filter: parse(&kb.filter),
            up: parse(&kb.up),
            down: parse(&kb.down),
            left: parse(&kb.left),
            right: parse(&kb.right),
            attach: parse(&kb.attach),
            start: parse(&kb.start),
            stop: parse(&kb.stop),
            set_default: parse(&kb.set_default),
            remove: parse(&kb.remove),
            export: parse(&kb.export),
            import: parse(&kb.import_distro),
        }
    }

    /// Returns `true` when `key` matches the binding for `action`.
    ///
    /// Uses `contains` for modifier comparison so that e.g. CONTROL matches
    /// even when CAPS_LOCK is active.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wsl_core::Config;
    /// use wsl_tui::keybindings::{KeyBindings, KeyAction};
    /// use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    ///
    /// let kb = KeyBindings::from_config(&Config::default());
    /// let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
    /// assert!(kb.matches(&key, KeyAction::Quit));
    /// ```
    pub fn matches(&self, key: &KeyEvent, action: KeyAction) -> bool {
        let (code, mods) = self.binding(action);
        key.code == code && key.modifiers.contains(mods)
    }

    /// Return the `(KeyCode, KeyModifiers)` pair for the given action.
    fn binding(&self, action: KeyAction) -> (KeyCode, KeyModifiers) {
        match action {
            KeyAction::Quit => self.quit,
            KeyAction::Help => self.help,
            KeyAction::Filter => self.filter,
            KeyAction::Up => self.up,
            KeyAction::Down => self.down,
            KeyAction::Left => self.left,
            KeyAction::Right => self.right,
            KeyAction::Attach => self.attach,
            KeyAction::Start => self.start,
            KeyAction::Stop => self.stop,
            KeyAction::SetDefault => self.set_default,
            KeyAction::Remove => self.remove,
            KeyAction::Export => self.export,
            KeyAction::Import => self.import,
        }
    }
}

/// Parse a key string into a `(KeyCode, KeyModifiers)` pair.
///
/// Supported formats:
/// - Single printable character: `"j"`, `"?"`, `"/"`, `"1"`.
/// - Special keys: `"enter"`, `"esc"` / `"escape"`, `"tab"`, `"backspace"`,
///   `"up"`, `"down"`, `"left"`, `"right"`, `"space"`.
/// - Function keys: `"f1"` … `"f12"`.
/// - Modified keys: `"ctrl+<char>"`, `"alt+<char>"`.
///
/// Returns `None` for unrecognised strings.
///
/// # Example
///
/// ```rust
/// use wsl_tui::keybindings::parse_key_str;
/// use crossterm::event::{KeyCode, KeyModifiers};
///
/// assert_eq!(parse_key_str("j"), Some((KeyCode::Char('j'), KeyModifiers::NONE)));
/// assert_eq!(parse_key_str("ctrl+d"), Some((KeyCode::Char('d'), KeyModifiers::CONTROL)));
/// assert_eq!(parse_key_str("enter"), Some((KeyCode::Enter, KeyModifiers::NONE)));
/// assert_eq!(parse_key_str("foobar"), None);
/// ```
pub fn parse_key_str(s: &str) -> Option<(KeyCode, KeyModifiers)> {
    let s = s.trim();

    // ── Modified keys: ctrl+<char> / alt+<char> ──────────────────────────────
    if let Some(rest) = s.strip_prefix("ctrl+") {
        let chars: Vec<char> = rest.chars().collect();
        if chars.len() == 1 {
            return Some((KeyCode::Char(chars[0]), KeyModifiers::CONTROL));
        }
        return None;
    }
    if let Some(rest) = s.strip_prefix("alt+") {
        let chars: Vec<char> = rest.chars().collect();
        if chars.len() == 1 {
            return Some((KeyCode::Char(chars[0]), KeyModifiers::ALT));
        }
        return None;
    }

    // ── Special named keys ────────────────────────────────────────────────────
    match s {
        "enter" => return Some((KeyCode::Enter, KeyModifiers::NONE)),
        "esc" | "escape" => return Some((KeyCode::Esc, KeyModifiers::NONE)),
        "tab" => return Some((KeyCode::Tab, KeyModifiers::NONE)),
        "backspace" => return Some((KeyCode::Backspace, KeyModifiers::NONE)),
        "up" => return Some((KeyCode::Up, KeyModifiers::NONE)),
        "down" => return Some((KeyCode::Down, KeyModifiers::NONE)),
        "left" => return Some((KeyCode::Left, KeyModifiers::NONE)),
        "right" => return Some((KeyCode::Right, KeyModifiers::NONE)),
        "space" => return Some((KeyCode::Char(' '), KeyModifiers::NONE)),
        _ => {}
    }

    // ── Function keys: f1..f12 ────────────────────────────────────────────────
    if let Some(n_str) = s.strip_prefix('f') {
        if let Ok(n) = n_str.parse::<u8>() {
            if (1..=12).contains(&n) {
                return Some((KeyCode::F(n), KeyModifiers::NONE));
            }
        }
    }

    // ── Single printable character ────────────────────────────────────────────
    let chars: Vec<char> = s.chars().collect();
    if chars.len() == 1 {
        return Some((KeyCode::Char(chars[0]), KeyModifiers::NONE));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: mods,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    // ── parse_key_str ────────────────────────────────────────────────────────

    #[test]
    fn test_parse_single_char() {
        let result = parse_key_str("j");
        assert_eq!(result, Some((KeyCode::Char('j'), KeyModifiers::NONE)));
    }

    #[test]
    fn test_parse_enter() {
        let result = parse_key_str("enter");
        assert_eq!(result, Some((KeyCode::Enter, KeyModifiers::NONE)));
    }

    #[test]
    fn test_parse_ctrl_modifier() {
        let result = parse_key_str("ctrl+d");
        assert_eq!(result, Some((KeyCode::Char('d'), KeyModifiers::CONTROL)));
    }

    #[test]
    fn test_parse_alt_modifier() {
        let result = parse_key_str("alt+x");
        assert_eq!(result, Some((KeyCode::Char('x'), KeyModifiers::ALT)));
    }

    #[test]
    fn test_parse_special_keys() {
        assert_eq!(
            parse_key_str("esc"),
            Some((KeyCode::Esc, KeyModifiers::NONE))
        );
        assert_eq!(
            parse_key_str("escape"),
            Some((KeyCode::Esc, KeyModifiers::NONE))
        );
        assert_eq!(
            parse_key_str("tab"),
            Some((KeyCode::Tab, KeyModifiers::NONE))
        );
        assert_eq!(
            parse_key_str("backspace"),
            Some((KeyCode::Backspace, KeyModifiers::NONE))
        );
        assert_eq!(
            parse_key_str("space"),
            Some((KeyCode::Char(' '), KeyModifiers::NONE))
        );
    }

    #[test]
    fn test_parse_function_keys() {
        assert_eq!(
            parse_key_str("f1"),
            Some((KeyCode::F(1), KeyModifiers::NONE))
        );
        assert_eq!(
            parse_key_str("f12"),
            Some((KeyCode::F(12), KeyModifiers::NONE))
        );
        assert_eq!(
            parse_key_str("f6"),
            Some((KeyCode::F(6), KeyModifiers::NONE))
        );
    }

    #[test]
    fn test_parse_arrow_keys() {
        assert_eq!(
            parse_key_str("up"),
            Some((KeyCode::Up, KeyModifiers::NONE))
        );
        assert_eq!(
            parse_key_str("down"),
            Some((KeyCode::Down, KeyModifiers::NONE))
        );
        assert_eq!(
            parse_key_str("left"),
            Some((KeyCode::Left, KeyModifiers::NONE))
        );
        assert_eq!(
            parse_key_str("right"),
            Some((KeyCode::Right, KeyModifiers::NONE))
        );
    }

    #[test]
    fn test_parse_question_mark() {
        let result = parse_key_str("?");
        assert_eq!(result, Some((KeyCode::Char('?'), KeyModifiers::NONE)));
    }

    #[test]
    fn test_parse_invalid() {
        assert_eq!(parse_key_str("foobar"), None);
        // "f0" and "f13" are out of range
        assert_eq!(parse_key_str("f0"), None);
        assert_eq!(parse_key_str("f13"), None);
        // ctrl with no char
        assert_eq!(parse_key_str("ctrl+"), None);
        // multi-char without modifier
        assert_eq!(parse_key_str("ab"), None);
    }

    // ── KeyBindings ──────────────────────────────────────────────────────────

    #[test]
    fn test_keybindings_from_default_config() {
        let config = Config::default();
        let kb = KeyBindings::from_config(&config);

        // Verify a sample of default bindings parsed correctly.
        let quit_key = make_key(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(kb.matches(&quit_key, KeyAction::Quit));

        let down_key = make_key(KeyCode::Char('j'), KeyModifiers::NONE);
        assert!(kb.matches(&down_key, KeyAction::Down));

        let enter_key = make_key(KeyCode::Enter, KeyModifiers::NONE);
        assert!(kb.matches(&enter_key, KeyAction::Attach));

        let help_key = make_key(KeyCode::Char('?'), KeyModifiers::NONE);
        assert!(kb.matches(&help_key, KeyAction::Help));
    }

    #[test]
    fn test_keybindings_no_false_positive() {
        let config = Config::default();
        let kb = KeyBindings::from_config(&config);

        // 'q' should NOT match Down.
        let quit_key = make_key(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(!kb.matches(&quit_key, KeyAction::Down));
    }

    #[test]
    fn test_keybindings_all_actions_parseable() {
        // Construct from default config — all actions must be parseable.
        // This will panic (fail the test) if any default string is invalid.
        let config = Config::default();
        let kb = KeyBindings::from_config(&config);

        // Exhaustively probe all actions with a non-matching key — just to
        // exercise the binding() match arm for every variant.
        let dummy = make_key(KeyCode::Null, KeyModifiers::NONE);
        let actions = [
            KeyAction::Quit,
            KeyAction::Help,
            KeyAction::Filter,
            KeyAction::Up,
            KeyAction::Down,
            KeyAction::Left,
            KeyAction::Right,
            KeyAction::Attach,
            KeyAction::Start,
            KeyAction::Stop,
            KeyAction::SetDefault,
            KeyAction::Remove,
            KeyAction::Export,
            KeyAction::Import,
        ];
        for action in actions {
            // Just call matches — we don't care about the result, only that it
            // doesn't panic.
            let _ = kb.matches(&dummy, action);
        }
    }
}
