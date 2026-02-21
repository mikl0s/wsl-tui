//! Text input modals for export and import operations.
//!
//! Provides two renderers:
//! - [`render_export_input`] — single text field for specifying the export `.tar` path.
//! - [`render_import_input`] — three-field form for name, install directory, and tar path.
//!
//! Both modals use double-border blocks and Catppuccin Mocha colours consistent
//! with the rest of the modal layer.  Active fields are highlighted in MAUVE;
//! inactive fields use SURFACE1.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::theme;
use crate::ui::popup::popup_area;

/// Render a cursor indicator by inserting an underscore at the cursor position.
///
/// If `cursor` is past the end, the underscore is appended.
fn render_with_cursor(text: &str, cursor: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let pos = cursor.min(chars.len());
    let mut result: Vec<char> = chars[..pos].to_vec();
    result.push('_');
    result.extend_from_slice(&chars[pos..]);
    result.iter().collect()
}

/// Render the export path input modal.
///
/// Shows a single text field for the user to enter the `.tar` export path.
/// The field supports character insertion, backspace, and cursor movement.
///
/// # Example
///
/// ```rust,no_run
/// // Called from ui::render when ModalState::ExportInput is active.
/// ```
pub fn render_export_input(frame: &mut Frame, distro_name: &str, path: &str, cursor: usize) {
    let area = popup_area(frame.area(), 60, 30);
    frame.render_widget(Clear, area);

    let title = format!(" Export '{}' ", distro_name);
    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(theme::BLUE)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme::BLUE))
        .style(Style::default().bg(theme::BASE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout: label row + input row + empty spacer + hint row.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner);

    // Label.
    let label = Paragraph::new(Span::styled(
        " Export path (.tar):",
        Style::default().fg(theme::SUBTEXT1),
    ));
    frame.render_widget(label, chunks[0]);

    // Input field with cursor.
    let display_text = render_with_cursor(path, cursor);
    let input_field = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled(
            display_text,
            Style::default()
                .fg(theme::TEXT)
                .bg(theme::SURFACE0),
        ),
    ]));
    frame.render_widget(input_field, chunks[1]);

    // Bottom hint.
    let hint = Paragraph::new(Span::styled(
        " [Enter] Export  [←→] Move cursor  [Backspace] Delete  [Esc] Cancel",
        Style::default().fg(theme::SUBTEXT0),
    ));
    frame.render_widget(hint, chunks[3]);
}

/// Render the import distro multi-field input modal.
///
/// Shows three stacked text input fields:
/// - Distribution name
/// - Install directory
/// - Tar file path
///
/// The active field is highlighted with a MAUVE border; inactive fields use SURFACE1.
/// Press `Tab` to cycle between fields.
///
/// # Example
///
/// ```rust,no_run
/// // Called from ui::render when ModalState::ImportInput is active.
/// ```
pub fn render_import_input(
    frame: &mut Frame,
    name: &str,
    install_dir: &str,
    tar_path: &str,
    active_field: u8,
    cursor: usize,
) {
    let area = popup_area(frame.area(), 60, 50);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(
            " Import Distribution ",
            Style::default()
                .fg(theme::BLUE)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme::BLUE))
        .style(Style::default().bg(theme::BASE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout: label + field for each of the three inputs + spacer + hint.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // label 0
            Constraint::Length(1), // field 0
            Constraint::Length(1), // spacer
            Constraint::Length(1), // label 1
            Constraint::Length(1), // field 1
            Constraint::Length(1), // spacer
            Constraint::Length(1), // label 2
            Constraint::Length(1), // field 2
            Constraint::Min(0),    // spacer
            Constraint::Length(1), // hint
        ])
        .split(inner);

    let field_data: [(&str, &str, u8); 3] = [
        ("Distribution name:", name, 0),
        ("Install directory:", install_dir, 1),
        ("Tar file path:", tar_path, 2),
    ];

    let label_slots = [0, 3, 6];
    let field_slots = [1, 4, 7];

    for (i, ((label_text, field_value, field_id), (label_slot, field_slot))) in field_data
        .iter()
        .zip(label_slots.iter().zip(field_slots.iter()))
        .enumerate()
    {
        let is_active = *field_id == active_field;

        // Label.
        let label_color = if is_active { theme::MAUVE } else { theme::SUBTEXT1 };
        let label = Paragraph::new(Span::styled(
            format!(" {}:", label_text),
            Style::default().fg(label_color),
        ));
        frame.render_widget(label, chunks[*label_slot]);

        // Field value — show cursor only on active field.
        let display_text = if is_active {
            render_with_cursor(field_value, cursor)
        } else {
            field_value.to_string()
        };

        let field_bg = if is_active { theme::SURFACE0 } else { theme::BASE };
        let field_fg = if is_active { theme::TEXT } else { theme::SUBTEXT0 };

        let field_widget = Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(
                display_text,
                Style::default().fg(field_fg).bg(field_bg),
            ),
        ]));
        frame.render_widget(field_widget, chunks[*field_slot]);

        // Suppress unused variable warning for `i` — it identifies the field but
        // we use `field_id` for comparison. Keep `i` for possible future use.
        let _ = i;
    }

    // Bottom hint.
    let hint = Paragraph::new(Span::styled(
        " [Enter] Import  [Tab] Next field  [Backspace] Delete  [Esc] Cancel",
        Style::default().fg(theme::SUBTEXT0),
    ));
    frame.render_widget(hint, chunks[9]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_with_cursor_at_end() {
        let result = render_with_cursor("hello", 5);
        assert_eq!(result, "hello_", "cursor at end appends underscore");
    }

    #[test]
    fn test_render_with_cursor_at_start() {
        let result = render_with_cursor("hello", 0);
        assert_eq!(result, "_hello", "cursor at start prepends underscore");
    }

    #[test]
    fn test_render_with_cursor_middle() {
        let result = render_with_cursor("hello", 2);
        assert_eq!(result, "he_llo", "cursor in middle inserts underscore");
    }

    #[test]
    fn test_render_with_cursor_empty_string() {
        let result = render_with_cursor("", 0);
        assert_eq!(result, "_", "cursor on empty string shows single underscore");
    }

    #[test]
    fn test_render_with_cursor_past_end_clamps() {
        let result = render_with_cursor("ab", 100);
        // Should clamp to end — 3 chars: a, b, _
        assert_eq!(result, "ab_", "cursor past end should clamp to end");
    }

    #[test]
    fn test_render_functions_exist() {
        let _: fn(&mut Frame, &str, &str, usize) = render_export_input;
        let _: fn(&mut Frame, &str, &str, &str, u8, usize) = render_import_input;
    }
}
