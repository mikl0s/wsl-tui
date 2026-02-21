//! Confirmation modal for destructive distro actions.
//!
//! Renders a centred popup with a y/N confirmation prompt and a clearly styled
//! "cannot be undone" warning.  Shown when [`ModalState::Confirm`] is active.
//!
//! # Usage
//!
//! Call [`render_confirm`] after the dashboard has been drawn but before
//! returning from the frame closure:
//!
//! ```text
//! dashboard::render_dashboard(app, frame);
//! if let ModalState::Confirm { ref distro_name, ref action_label } = app.modal {
//!     confirm_modal::render_confirm(frame, distro_name, action_label);
//! }
//! ```
//!
//! # Input handling
//!
//! Key routing when this modal is active (handled in `main.rs`):
//! - `y` / `Y` → call `executor.unregister()`, refresh list, close modal
//! - Any other key → close modal (cancel)

use ratatui::{
    Frame,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::theme;
use crate::ui::popup::popup_area;

/// Render the confirmation modal for a destructive action.
///
/// The popup is 60% wide and 30% tall, centred in the terminal area.
///
/// # Arguments
///
/// * `frame` — The ratatui frame to render into.
/// * `distro_name` — Name of the distro the action targets (shown in the message).
/// * `action_label` — Human-readable label for the action (e.g., `"Remove"`).
///
/// # Example
///
/// ```rust,no_run
/// use wsl_tui::ui::confirm_modal;
/// ```
pub fn render_confirm(frame: &mut Frame, distro_name: &str, action_label: &str) {
    let area = frame.area();
    let popup = popup_area(area, 60, 30);

    // Erase the background under the popup.
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(
            format!(" {action_label} "),
            Style::default()
                .fg(theme::RED)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme::RED));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let lines = build_confirm_content(distro_name);
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Build the confirmation modal content lines.
fn build_confirm_content(distro_name: &str) -> Vec<Line<'static>> {
    let name = distro_name.to_owned();

    vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Remove '", Style::default().fg(theme::TEXT)),
            Span::styled(
                name,
                Style::default()
                    .fg(theme::MAUVE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("' from WSL?", Style::default().fg(theme::TEXT)),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "  WARNING: All data will be permanently lost.",
            Style::default().fg(theme::YELLOW),
        )),
        Line::from(Span::styled(
            "  This action cannot be undone.",
            Style::default().fg(theme::YELLOW),
        )),
        Line::raw(""),
        Line::from(vec![
            Span::raw("  Press "),
            Span::styled(
                "[y]",
                Style::default()
                    .fg(theme::RED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to confirm, ", Style::default().fg(theme::TEXT)),
            Span::styled("any other key", Style::default().fg(theme::SUBTEXT1)),
            Span::styled(" to cancel.", Style::default().fg(theme::TEXT)),
        ]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the confirm content includes the distro name.
    #[test]
    fn test_confirm_content_contains_distro_name() {
        let lines = build_confirm_content("Ubuntu");
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<_>>()
            .join("");

        assert!(all_text.contains("Ubuntu"), "Confirm content should include the distro name");
    }

    /// Verify that the warning text is present in the confirm content.
    #[test]
    fn test_confirm_content_contains_warning() {
        let lines = build_confirm_content("SomeDistro");
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<Vec<_>>()
            .join("");

        assert!(
            all_text.contains("WARNING"),
            "Confirm content should include a WARNING"
        );
        assert!(
            all_text.contains("cannot be undone"),
            "Confirm content should include 'cannot be undone'"
        );
        assert!(all_text.contains("[y]"), "Confirm content should include [y] key hint");
    }

    /// Verify content is non-empty for any distro name.
    #[test]
    fn test_confirm_content_non_empty() {
        let lines = build_confirm_content("test-distro");
        assert!(!lines.is_empty(), "Confirm content should not be empty");
    }
}
