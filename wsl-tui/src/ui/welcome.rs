//! Welcome screen widget.
//!
//! Displayed on first launch (when no `config.toml` existed before this run).
//! The screen is centered both horizontally and vertically, and prompts the
//! user to press any key to continue.
//!
//! Per the locked product decision, this screen is the user's first
//! impression — it should feel polished.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Render the welcome screen into `frame`.
///
/// The welcome block is centered horizontally (60 % of the terminal width,
/// minimum 40 columns) and vertically (7 lines tall, plus borders = 9 rows).
/// A dim overlay of the remaining area is achieved by simply leaving it blank.
pub fn render_welcome(frame: &mut Frame) {
    let area = frame.area();

    // ── Vertical centering ────────────────────────────────────────────────────
    let box_height: u16 = 12;
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(box_height),
            Constraint::Fill(1),
        ])
        .split(area);

    let center_row = vertical[1];

    // ── Horizontal centering ──────────────────────────────────────────────────
    let box_width: u16 = area.width.clamp(44, 72);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(box_width),
            Constraint::Fill(1),
        ])
        .split(center_row);

    let popup = horizontal[1];

    render_welcome_box(frame, popup);
}

/// Draw the welcome box into a pre-computed [`Rect`].
fn render_welcome_box(frame: &mut Frame, area: Rect) {
    // Build content lines.
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Welcome to WSL TUI",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Configuration: "),
            Span::styled(
                "~/.wsl-tui/config.toml",
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  A fully-commented config file has been created for you.",
            Style::default().fg(Color::Green),
        )]),
        Line::from(vec![Span::raw(
            "  Customize settings by editing the config file.",
        )]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Press any key to continue...",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];

    let block = Block::default()
        .title(Span::styled(
            " WSL TUI — First Run ",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
