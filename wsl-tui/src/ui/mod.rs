//! UI rendering module for the WSL TUI.
//!
//! The [`render`] function is the single entry point called from the event
//! loop on every frame.  It dispatches to sub-renderers based on application
//! state.

pub mod welcome;

use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::app::App;

/// Render the current application state into `frame`.
///
/// - If `app.show_welcome` is `true`, shows the first-run welcome screen.
/// - Otherwise shows a placeholder main screen with a quit hint.
pub fn render(app: &App, frame: &mut Frame) {
    if app.show_welcome {
        welcome::render_welcome(frame);
    } else {
        render_placeholder(frame);
    }
}

/// Render the placeholder main screen.
///
/// Phase 1 stub — replaced with the full distro list UI in Phase 2.
fn render_placeholder(frame: &mut Frame) {
    let area = frame.area();

    let text = Paragraph::new("WSL TUI v0.1.0 — Press q to quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));

    // Center vertically by placing the text in the middle row.
    let vertical_mid = area.height / 2;
    let centered = ratatui::layout::Rect {
        x: area.x,
        y: area.y + vertical_mid,
        width: area.width,
        height: 1,
    };

    frame.render_widget(text, centered);
}
