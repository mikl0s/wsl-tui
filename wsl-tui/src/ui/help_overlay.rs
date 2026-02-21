//! Context-aware help overlay for the WSL TUI.
//!
//! Renders a centred modal popup listing all keybindings for the current view.
//! The popup is drawn on top of the existing dashboard using the Clear + Block
//! pattern — the background is erased under the popup before drawing the
//! content.
//!
//! # Usage
//!
//! Call [`render_help`] after the dashboard has been drawn but before returning
//! from the frame closure:
//!
//! ```text
//! dashboard::render_dashboard(app, frame);
//! if app.modal == ModalState::Help {
//!     help_overlay::render_help(app, frame);
//! }
//! ```

use ratatui::{
    Frame,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::app::{App, View};
use crate::theme;
use crate::ui::popup::popup_area;

/// Render the help overlay on top of the current frame.
///
/// The popup is 70% wide and 80% tall, centred in the terminal area.
/// Keybindings shown are context-aware based on [`App::current_view`].
///
/// # Example
///
/// ```rust,no_run
/// use wsl_tui::app::{App, ModalState};
/// use wsl_tui::ui::help_overlay;
/// ```
pub fn render_help(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let popup = popup_area(area, 70, 80);

    // Erase the background under the popup.
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(
            " Help ",
            Style::default()
                .fg(theme::LAVENDER)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme::MAUVE));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let content = build_help_content(app.current_view);
    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, inner);
}

/// Build the keybinding content lines for the given view.
fn build_help_content(view: View) -> Vec<Line<'static>> {
    match view {
        View::Dashboard => build_dashboard_help(),
        _ => build_dashboard_help(), // All other views show dashboard help for now
    }
}

/// Build keybinding lines for the Dashboard view.
fn build_dashboard_help() -> Vec<Line<'static>> {
    vec![
        // Blank top padding.
        Line::raw(""),
        // ── Navigation section ─────────────────────────────────────────────────
        section_header("  Navigation"),
        binding("j / ↓", "Move down"),
        binding("k / ↑", "Move up"),
        binding("Tab", "Switch panel focus"),
        binding("1-5", "Switch views"),
        Line::raw(""),
        // ── Distro Actions section ─────────────────────────────────────────────
        section_header("  Distro Actions"),
        binding("Enter", "Attach shell (starts distro if stopped)"),
        binding("s", "Start distro"),
        binding("t", "Stop / terminate distro"),
        binding("d", "Set as default"),
        binding("x", "Remove distro (with confirmation)"),
        binding("e", "Export to .tar"),
        binding("i", "Import from .tar"),
        Line::raw(""),
        // ── Search & UI section ────────────────────────────────────────────────
        section_header("  Search & UI"),
        binding("/", "Filter distros"),
        binding("?", "Toggle this help"),
        binding("q", "Quit"),
    ]
}

/// Build a section header line styled with LAVENDER + BOLD.
fn section_header(title: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        title,
        Style::default()
            .fg(theme::LAVENDER)
            .add_modifier(Modifier::BOLD),
    ))
}

/// Build a keybinding line: YELLOW + BOLD key, SUBTEXT1 description.
fn binding(key: &'static str, desc: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw("    "),
        Span::styled(
            key,
            Style::default()
                .fg(theme::YELLOW)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("      "),
        Span::styled(desc, Style::default().fg(theme::SUBTEXT1)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that build_help_content returns a non-empty Vec for the Dashboard view.
    #[test]
    fn test_help_content_dashboard_non_empty() {
        let lines = build_help_content(View::Dashboard);
        assert!(!lines.is_empty(), "Dashboard help should produce content lines");
    }

    /// Verify that the content contains at least the expected key labels.
    #[test]
    fn test_help_content_contains_key_labels() {
        let lines = build_help_content(View::Dashboard);
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<_>>()
            .join("");

        assert!(all_text.contains("Enter"), "Should contain Enter keybinding");
        assert!(all_text.contains("Navigation"), "Should contain Navigation section");
        assert!(all_text.contains("Distro Actions"), "Should contain Distro Actions section");
        assert!(all_text.contains("Search & UI"), "Should contain Search & UI section");
    }

    /// Verify that non-Dashboard views fall back to dashboard help (same non-empty content).
    #[test]
    fn test_help_content_other_views_fallback() {
        let lines = build_help_content(View::Provision);
        assert!(!lines.is_empty(), "Provision view should fall back to non-empty help");
    }
}
