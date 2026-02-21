//! User-initiated action types for the WSL TUI event loop.
//!
//! [`Action`] represents the set of all operations a user can trigger via
//! keyboard. The event loop maps raw [`crossterm::event::KeyEvent`]s to
//! `Action` variants using [`crate::keybindings::KeyBindings`], then calls
//! the relevant [`crate::app::App`] method or executes the operation inline.

use crate::app::View;

/// Every user-initiated operation in the WSL TUI.
///
/// Key events are translated to `Action` variants in `resolve_action` in
/// `main.rs`. This keeps the event loop thin: input → Action → App mutation.
///
/// Note: Some variants (e.g., `TerminateDistro`) are defined for completeness
/// of the Phase 2 action surface but not wired as a separate key binding.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Exit the application.
    Quit,

    // ── Navigation ────────────────────────────────────────────────────────────
    /// Move the selection upward (vim `k` or arrow up).
    MoveUp,
    /// Move the selection downward (vim `j` or arrow down).
    MoveDown,
    /// Move focus left (vim `h` or arrow left).
    MoveLeft,
    /// Move focus right (vim `l` or arrow right).
    MoveRight,
    /// Cycle focus between panels (Tab key).
    SwitchFocus,

    // ── View switching ────────────────────────────────────────────────────────
    /// Switch to the named view (number keys 1–5).
    SwitchView(View),

    // ── Distro lifecycle actions ──────────────────────────────────────────────
    /// Start the selected (stopped) distro.
    StartDistro,
    /// Stop / terminate the selected (running) distro.
    StopDistro,
    /// Terminate the selected distro (force-kill, equivalent to `wsl --terminate`).
    TerminateDistro,
    /// Set the selected distro as the WSL default.
    SetDefault,
    /// Remove / unregister the selected distro (triggers confirm modal).
    RemoveDistro,
    /// Attach an interactive shell to the selected distro.
    AttachShell,
    /// Export the selected distro to a `.tar` file (triggers path-input modal).
    ExportDistro,
    /// Import a distro from a `.tar` file (triggers path-input modal).
    ImportDistro,
    /// Install a new distro from the online catalog.
    InstallDistro,
    /// Update the WSL kernel to the latest version.
    UpdateWsl,

    // ── UI toggles ────────────────────────────────────────────────────────────
    /// Toggle the help overlay.
    ToggleHelp,
    /// Activate / deactivate the distro filter bar.
    ToggleFilter,

    // ── Filter input (active while filter bar is open) ────────────────────────
    /// Append a character to the filter text.
    FilterChar(char),
    /// Delete the last character from the filter text (Backspace).
    FilterBackspace,
    /// Close the filter bar and clear the filter (Escape).
    FilterEscape,

    // ── Modal text input ──────────────────────────────────────────────────────
    /// Insert a character at the cursor position in the active modal input field.
    ModalInputChar(char),
    /// Delete the character before the cursor in the active modal input field.
    ModalInputBackspace,
    /// Move the cursor left in the active modal input field.
    ModalInputLeft,
    /// Move the cursor right in the active modal input field.
    ModalInputRight,
    /// Cycle to the next field in a multi-field modal (Tab key).
    ModalInputTab,

    // ── Modal responses ───────────────────────────────────────────────────────
    /// Confirm a destructive action or submit a modal form (e.g., distro removal, install, export/import).
    ConfirmYes,
    /// Cancel a modal dialog without taking action.
    ConfirmNo,

    // ── Sentinel ──────────────────────────────────────────────────────────────
    /// No-op — used when a key press does not map to any bound action.
    None,
}
