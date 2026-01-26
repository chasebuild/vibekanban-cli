//! Help view with keyboard shortcuts.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    app::App,
    ui::components::{render_header, render_status_bar},
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header
            Constraint::Min(10),    // Content
            Constraint::Length(2),  // Status
        ])
        .split(frame.area());

    // Header
    render_header(frame, chunks[0], "Help");

    // Help content
    let help_area = centered_rect(80, 80, chunks[1]);
    render_help_content(frame, help_area);

    // Status bar
    render_status_bar(frame, chunks[2], app);
}

fn render_help_content(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .margin(1)
        .split(area);

    let outer_block = Block::default()
        .title(" Keyboard Shortcuts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(outer_block, area);

    // Navigation shortcuts
    let nav_content = vec![
        section_header("Navigation"),
        shortcut("↑/k", "Move up"),
        shortcut("↓/j", "Move down"),
        shortcut("←/h", "Move left / Previous column"),
        shortcut("→/l", "Move right / Next column"),
        shortcut("Enter", "Select / Confirm"),
        shortcut("Esc", "Go back / Cancel"),
        shortcut("Tab", "Next field (in forms)"),
        Line::from(""),
        section_header("Global"),
        shortcut("?", "Show this help"),
        shortcut("q", "Quit application"),
        shortcut("r", "Refresh current view"),
    ];

    let nav_paragraph = Paragraph::new(nav_content);
    frame.render_widget(nav_paragraph, chunks[0]);

    // Action shortcuts
    let action_content = vec![
        section_header("Projects"),
        shortcut("n", "Create new project"),
        shortcut("Enter", "Select project"),
        Line::from(""),
        section_header("Tasks"),
        shortcut("n", "Create new task"),
        shortcut("m", "Move task to next status"),
        shortcut("d", "Delete task"),
        shortcut("Enter", "View task workspaces"),
        Line::from(""),
        section_header("Git Operations"),
        shortcut("m", "Merge to target branch"),
        shortcut("p", "Push to remote"),
        shortcut("P", "Force push to remote"),
        shortcut("b", "Rebase on target branch"),
        shortcut("s", "Stop running process"),
        shortcut("f", "Send follow-up message"),
    ];

    let action_paragraph = Paragraph::new(action_content);
    frame.render_widget(action_paragraph, chunks[1]);
}

fn section_header(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        title.to_string(),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
}

fn shortcut(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:12}", key),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc.to_string(), Style::default().fg(Color::White)),
    ])
}

/// Helper function to create a centered rect.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
