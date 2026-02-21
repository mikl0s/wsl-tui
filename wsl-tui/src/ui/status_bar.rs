//! Status bar renderer for the WSL TUI.
//!
//! Renders a single-row bar at the bottom of the screen showing:
//! - Left: selected distro name and state indicator
//! - Centre: current view name
//! - Right: storage backend indicator and clock (HH:MM)

use chrono::Local;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;
use crate::theme;

/// Render the status bar into `area`.
///
/// The bar uses the Catppuccin Mocha Mantle colour as its background.
/// Left, centre, and right sections are rendered as separate [`Paragraph`]
/// widgets so each can have its own alignment without fighting over the same
/// string width.
///
/// # Example
///
/// ```rust,no_run
/// // Called from dashboard::render_dashboard after the split-pane.
/// // status_bar::render(app, frame, status_area);
/// ```
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    // Background style — Mantle for the full bar.
    let bg = Style::default()
        .bg(theme::MANTLE)
        .fg(theme::SUBTEXT1);

    // ── Left section: selected distro + state ─────────────────────────────────
    let left_text = build_left_section(app);
    let left = Paragraph::new(left_text)
        .style(bg)
        .alignment(Alignment::Left);
    frame.render_widget(left, area);

    // ── Centre section: view name ─────────────────────────────────────────────
    let view_name = app.current_view.display_name();
    let centre_text = Line::from(vec![
        Span::styled(view_name, Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD)),
    ]);
    let centre = Paragraph::new(centre_text)
        .style(bg)
        .alignment(Alignment::Center);
    frame.render_widget(centre, area);

    // ── Right section: storage backend + clock ────────────────────────────────
    let right_text = build_right_section(app);
    let right = Paragraph::new(right_text)
        .style(bg)
        .alignment(Alignment::Right);
    frame.render_widget(right, area);
}

/// Build the left status bar section.
fn build_left_section(app: &App) -> Line<'static> {
    if let Some(distro) = app.selected_distro() {
        use wsl_core::DistroState;
        let (indicator, state_color) = match distro.state {
            DistroState::Running => ("● Running", theme::GREEN),
            DistroState::Stopped => ("○ Stopped", theme::RED),
        };
        let default_marker = if distro.is_default { " [default]" } else { "" };
        Line::from(vec![
            Span::raw(" "),
            Span::styled(distro.name.clone(), Style::default().fg(theme::TEXT).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(indicator.to_string(), Style::default().fg(state_color)),
            Span::styled(default_marker.to_string(), Style::default().fg(theme::MAUVE)),
        ])
    } else {
        Line::from(vec![
            Span::raw(" "),
            Span::styled("No distro selected", Style::default().fg(theme::SUBTEXT0)),
        ])
    }
}

/// Build the right status bar section.
fn build_right_section(app: &App) -> Line<'static> {
    let clock = Local::now().format("%H:%M").to_string();
    Line::from(vec![
        Span::styled(
            app.storage_backend.clone(),
            Style::default().fg(theme::SAPPHIRE),
        ),
        Span::raw("  "),
        Span::styled(clock, Style::default().fg(theme::OVERLAY0)),
        Span::raw(" "),
    ])
}
