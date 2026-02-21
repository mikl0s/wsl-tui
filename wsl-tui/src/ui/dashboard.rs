//! Dashboard split-pane view for the WSL TUI.
//!
//! Renders the primary distro-management view consisting of:
//! - Left pane (40%): distro list with state indicators
//! - Right pane (60%): details panel for the selected distro
//! - Bottom row (1 line): status bar
//!
//! Layout adapts to narrow terminals:
//! - Width < 60: single-column (list only, no details panel)
//! - Width < 40: "Terminal too narrow" message

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};
use wsl_core::DistroInfo;
use wsl_core::DistroState;

use crate::app::{App, FocusPanel};
use crate::theme;
use crate::ui::status_bar;

/// Render the full dashboard into `frame`.
///
/// Layout:
/// ```text
/// ┌───────────────────────────────────────────────────────┐
/// │  Distros (40%)         │  Details (60%)               │
/// │                        │                              │
/// │                        │                              │
/// ├────────────────────────┴──────────────────────────────┤
/// │  Status bar (1 row)                                   │
/// └───────────────────────────────────────────────────────┘
/// ```
pub fn render_dashboard(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    // ── Responsive guard ──────────────────────────────────────────────────────
    if area.width < 40 {
        let msg = Paragraph::new("Terminal too narrow — resize to at least 40 columns")
            .style(Style::default().fg(theme::RED));
        frame.render_widget(msg, area);
        return;
    }

    // Vertical split: [Fill(1) content, Length(1) status bar]
    let [content_area, status_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    if area.width < 60 {
        // Single-column: list only.
        render_distro_list(app, frame, content_area);
    } else {
        // Two-column split: 40% list, 60% details.
        let [list_area, details_area] = Layout::horizontal([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .areas(content_area);

        render_distro_list(app, frame, list_area);
        render_details_panel(app, frame, details_area);
    }

    status_bar::render(app, frame, status_area);
}

/// Render the distro list into `area`.
fn render_distro_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let focused = app.focus == FocusPanel::DistroList;

    // Shrink area by 1 row for the filter bar if active.
    let (list_area, filter_area) = if app.filter_active && area.height > 2 {
        let [filter, list] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        (list, Some(filter))
    } else {
        (area, None)
    };

    // Render filter bar.
    if let Some(fa) = filter_area {
        let filter_line = Line::from(vec![
            Span::styled("/", Style::default().fg(theme::PEACH).add_modifier(Modifier::BOLD)),
            Span::styled(app.filter_text.clone(), Style::default().fg(theme::PEACH)),
        ]);
        frame.render_widget(Paragraph::new(filter_line), fa);
    }

    // Build list items.
    let visible = app.visible_distros();
    let items: Vec<ListItem> = visible
        .iter()
        .map(|d| build_list_item(d))
        .collect();

    let border_style = Style::default().fg(if focused { theme::MAUVE } else { theme::SURFACE1 });

    let block = Block::default()
        .title(Span::styled(
            " Distros ",
            Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let highlight_style = Style::default()
        .add_modifier(Modifier::BOLD | Modifier::REVERSED);

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style);

    frame.render_stateful_widget(list, list_area, &mut app.list_state);
}

/// Build a single [`ListItem`] for the given distro.
fn build_list_item(distro: &DistroInfo) -> ListItem<'static> {
    let (prefix, prefix_color) = if distro.is_default {
        ("▸ ", theme::MAUVE)
    } else {
        ("  ", theme::SURFACE1)
    };

    let (indicator, state_color) = match distro.state {
        DistroState::Running => ("● ", theme::GREEN),
        DistroState::Stopped => ("○ ", theme::RED),
    };

    let version_label = format!("  v{}", distro.version);

    let line = Line::from(vec![
        Span::styled(prefix.to_string(), Style::default().fg(prefix_color)),
        Span::styled(indicator.to_string(), Style::default().fg(state_color)),
        Span::styled(
            distro.name.clone(),
            Style::default().fg(theme::TEXT),
        ),
        Span::styled(version_label, Style::default().fg(theme::OVERLAY0)),
    ]);

    ListItem::new(line)
}

/// Render the details panel for the selected distro into `area`.
fn render_details_panel(app: &App, frame: &mut Frame, area: Rect) {
    let focused = app.focus == FocusPanel::Details;
    let border_style = Style::default().fg(if focused { theme::MAUVE } else { theme::SURFACE1 });

    let block = Block::default()
        .title(Span::styled(
            " Details ",
            Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(distro) = app.selected_distro() {
        render_distro_details(distro, frame, inner);
    } else {
        let msg = Paragraph::new(Span::styled(
            "No distro selected",
            Style::default().fg(theme::SUBTEXT0),
        ));
        frame.render_widget(msg, inner);
    }
}

/// Render the distro details content inside the details panel's inner area.
fn render_distro_details(distro: &DistroInfo, frame: &mut Frame, area: Rect) {
    use wsl_core::DistroState;

    let (state_indicator, state_color) = match distro.state {
        DistroState::Running => ("● Running", theme::GREEN),
        DistroState::Stopped => ("○ Stopped", theme::RED),
    };

    let default_text = if distro.is_default { "Yes" } else { "No" };

    let mut lines: Vec<Line> = Vec::new();

    // Blank line for padding.
    lines.push(Line::raw(""));

    // Name
    lines.push(Line::from(vec![
        Span::styled("  Name:    ", Style::default().fg(theme::SUBTEXT1)),
        Span::styled(
            distro.name.clone(),
            Style::default().fg(theme::TEXT).add_modifier(Modifier::BOLD),
        ),
    ]));

    // State
    lines.push(Line::from(vec![
        Span::styled("  State:   ", Style::default().fg(theme::SUBTEXT1)),
        Span::styled(state_indicator.to_string(), Style::default().fg(state_color)),
    ]));

    // WSL Version
    lines.push(Line::from(vec![
        Span::styled("  Version: ", Style::default().fg(theme::SUBTEXT1)),
        Span::styled(
            format!("WSL {}", distro.version),
            Style::default().fg(theme::LAVENDER),
        ),
    ]));

    // Default
    lines.push(Line::from(vec![
        Span::styled("  Default: ", Style::default().fg(theme::SUBTEXT1)),
        Span::styled(default_text.to_string(), Style::default().fg(theme::TEXT)),
    ]));

    // Blank separator.
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "  ─────────────────────────────",
        Style::default().fg(theme::SURFACE2),
    )));
    lines.push(Line::raw(""));

    // Action hints.
    lines.push(build_hint_line("[Enter]", "Attach shell"));
    lines.push(build_hint_line("[s]", "Start"));
    lines.push(build_hint_line("[t]", "Stop / Terminate"));
    lines.push(build_hint_line("[d]", "Set as Default"));
    lines.push(build_hint_line("[x]", "Remove"));
    lines.push(build_hint_line("[e]", "Export"));
    lines.push(build_hint_line("[i]", "Import"));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Build a key-hint line styled with yellow keys and subtext1 descriptions.
fn build_hint_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            key.to_string(),
            Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(desc.to_string(), Style::default().fg(theme::SUBTEXT1)),
    ])
}
