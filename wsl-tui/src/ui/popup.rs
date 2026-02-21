//! Shared popup area helper for modal overlays.
//!
//! Provides [`popup_area`] — a pure function that returns a [`Rect`] centred
//! within a parent area at the requested percentage dimensions.  Used by both
//! [`super::help_overlay`] and [`super::confirm_modal`] to avoid code
//! duplication.
//!
//! # Pattern
//!
//! ```text
//! let popup = popup_area(frame.area(), 70, 70);
//! frame.render_widget(Clear, popup);        // erase background
//! frame.render_widget(block, popup);        // draw modal
//! ```

use ratatui::layout::{Constraint, Flex, Layout, Rect};

/// Return a [`Rect`] centred in `area` at the given percentage dimensions.
///
/// Uses `Flex::Center` via `Layout::vertical` / `Layout::horizontal` so the
/// popup is always centred regardless of terminal size.
///
/// # Example
///
/// ```rust,no_run
/// use ratatui::layout::Rect;
/// use wsl_tui::ui::popup::popup_area;
///
/// let area = Rect::new(0, 0, 80, 24);
/// let popup = popup_area(area, 70, 70);
/// // popup is centred within area, ~56 cols wide, ~16 rows tall
/// ```
pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);

    let [popup_vert] = vertical.areas(area);
    let [popup] = horizontal.areas(popup_vert);
    popup
}
