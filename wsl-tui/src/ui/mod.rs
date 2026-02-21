//! UI rendering module for the WSL TUI.
//!
//! The [`render`] function is the single entry point called from the event
//! loop on every frame.  It dispatches to sub-renderers based on application
//! state.
//!
//! # Module structure
//!
//! - [`welcome`] — first-run welcome screen
//! - [`dashboard`] — primary distro-management split-pane view
//! - [`status_bar`] — status bar rendered at the bottom of the dashboard

pub mod dashboard;
pub mod status_bar;
pub mod welcome;

use ratatui::{
    Frame,
    layout::Alignment,
    style::{Modifier, Style},
    widgets::Paragraph,
};

use crate::app::{App, View};
use crate::theme;

/// Render the current application state into `frame`.
///
/// Dispatch logic:
/// - If `app.show_welcome` is `true`, shows the first-run welcome screen.
/// - Otherwise dispatches to the view-specific renderer based on `app.current_view`.
pub fn render(app: &mut App, frame: &mut Frame) {
    if app.show_welcome {
        welcome::render_welcome(frame);
    } else {
        match app.current_view {
            View::Dashboard => dashboard::render_dashboard(app, frame),
            _ => render_view_placeholder(app, frame),
        }
    }
}

/// Render a placeholder for views not yet implemented (Phase 3+).
///
/// Shows the view name and a "not yet implemented" message centred on screen.
fn render_view_placeholder(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let view_name = app.current_view.display_name();

    let text = format!("{view_name} — not yet implemented (use number keys 1–5 to switch views)");

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme::SUBTEXT0)
                .add_modifier(Modifier::ITALIC),
        );

    // Centre vertically.
    let vertical_mid = area.height / 2;
    let centered = ratatui::layout::Rect {
        x: area.x,
        y: area.y + vertical_mid,
        width: area.width,
        height: 1,
    };

    frame.render_widget(paragraph, centered);
}
