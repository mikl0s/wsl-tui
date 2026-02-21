//! Install flow modals for the WSL TUI.
//!
//! Provides two renderers for the distro install flow:
//! - [`render_install_picker`] — a scrollable list of available online distros.
//! - [`render_install_progress`] — a progress gauge shown while `wsl --install` runs.
//! - [`render_update_progress`] — a progress indicator for `wsl --update`.
//!
//! Both modals use double-border blocks and Catppuccin Mocha colours, consistent
//! with the rest of the modal layer.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph},
};
use wsl_core::OnlineDistro;

use crate::app::App;
use crate::theme;
use crate::ui::popup::popup_area;

/// Render the online distro picker modal.
///
/// Shows a scrollable list of available WSL distros from the online catalog.
/// Navigation: `j`/`k` or arrow keys. `Enter` starts install. `Esc` cancels.
///
/// # Example
///
/// ```rust,no_run
/// // Called from ui::render when ModalState::InstallPicker is active.
/// ```
pub fn render_install_picker(
    app: &mut App,
    frame: &mut Frame,
    online_distros: &[OnlineDistro],
    list_state: &mut ListState,
) {
    let area = popup_area(frame.area(), 60, 60);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(
            " Install Distribution ",
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

    // Split inner area: list body + bottom hint line.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    // Build list items.
    let items: Vec<ListItem> = online_distros
        .iter()
        .map(|d| {
            let line = Line::from(vec![
                Span::styled(
                    format!("{:<20}", d.name),
                    Style::default().fg(theme::TEXT),
                ),
                Span::raw("  "),
                Span::styled(
                    d.friendly_name.as_str(),
                    Style::default().fg(theme::SUBTEXT1),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(theme::SURFACE0)
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[0], list_state);

    // Bottom hint.
    let hint = Paragraph::new(Span::styled(
        " [Enter] Install  [j/k] Navigate  [Esc] Cancel",
        Style::default().fg(theme::SUBTEXT0),
    ));
    frame.render_widget(hint, chunks[1]);

    // Suppress unused App warning — the app reference is here for future use
    // (e.g., showing the selected distro's details).
    let _ = app;
}

/// Render the install progress modal with a gauge widget.
///
/// Shows the current step label and a smooth progress bar while `wsl --install`
/// runs in the background.  When `completed` is `true`, a dismissal hint is shown.
///
/// # Example
///
/// ```rust,no_run
/// // Called from ui::render when ModalState::InstallProgress is active.
/// ```
pub fn render_install_progress(
    frame: &mut Frame,
    distro_name: &str,
    step: &str,
    percent: u16,
    completed: bool,
) {
    let area = popup_area(frame.area(), 60, 30);
    frame.render_widget(Clear, area);

    let title = format!(" Installing {} ", distro_name);
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

    // Vertical split: gauge + optional completion line.
    let hint_height: u16 = if completed { 1 } else { 0 };
    let constraints = if completed {
        vec![
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(hint_height),
        ]
    } else {
        vec![Constraint::Min(1)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let label = format!("{}%  {}", percent, step);
    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(theme::GREEN)
                .bg(theme::SURFACE0),
        )
        .percent(percent.min(100))
        .label(label)
        .use_unicode(true);

    frame.render_widget(gauge, chunks[0]);

    if completed && chunks.len() >= 3 {
        let done_hint = Paragraph::new(Span::styled(
            " Press any key to continue",
            Style::default()
                .fg(theme::GREEN)
                .add_modifier(Modifier::ITALIC),
        ));
        frame.render_widget(done_hint, chunks[2]);
    }
}

/// Render the WSL update progress modal.
///
/// Similar to [`render_install_progress`] but titled "Updating WSL".
///
/// # Example
///
/// ```rust,no_run
/// // Called from ui::render when ModalState::UpdateProgress is active.
/// ```
pub fn render_update_progress(frame: &mut Frame, step: &str, completed: bool) {
    let area = popup_area(frame.area(), 60, 25);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(
            " Updating WSL ",
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

    let constraints = if completed {
        vec![Constraint::Min(1), Constraint::Length(1)]
    } else {
        vec![Constraint::Min(1)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // Display step text since wsl --update doesn't have a numeric progress value.
    let step_para = Paragraph::new(Span::styled(
        format!("  {}", step),
        Style::default().fg(theme::TEXT),
    ));
    frame.render_widget(step_para, chunks[0]);

    if completed && chunks.len() >= 2 {
        let done_hint = Paragraph::new(Span::styled(
            " Press any key to continue",
            Style::default()
                .fg(theme::GREEN)
                .add_modifier(Modifier::ITALIC),
        ));
        frame.render_widget(done_hint, chunks[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_functions_exist() {
        // Verify render function signatures compile — actual rendering requires
        // a ratatui backend so full render tests are integration-level.
        // This test ensures the function signatures match what callers expect.
        let _: fn(&mut App, &mut Frame, &[OnlineDistro], &mut ListState) =
            render_install_picker;
        let _: fn(&mut Frame, &str, &str, u16, bool) = render_install_progress;
        let _: fn(&mut Frame, &str, bool) = render_update_progress;
    }
}
