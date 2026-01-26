//! Create task form view.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    app::{App, InputMode},
    ui::components::{render_header, render_hints, render_status_bar},
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header
            Constraint::Min(10),    // Form
            Constraint::Length(2),  // Hints
            Constraint::Length(2),  // Status
        ])
        .split(frame.area());

    // Header
    render_header(frame, chunks[0], "Create New Task");

    // Form area
    let form_area = centered_rect(60, 50, chunks[1]);
    render_form(frame, form_area, app);

    // Hints
    let hints = if app.input_mode == InputMode::Editing {
        vec![
            ("Enter", "Save"),
            ("Esc", "Cancel Edit"),
            ("Tab", "Next Field"),
        ]
    } else {
        vec![
            ("e", "Edit"),
            ("Enter", "Create"),
            ("Esc", "Cancel"),
        ]
    };
    render_hints(frame, chunks[2], &hints);

    // Status bar
    render_status_bar(frame, chunks[3], app);
}

fn render_form(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title field
            Constraint::Length(1),  // Spacer
            Constraint::Min(5),     // Description field
        ])
        .split(area);

    let outer_block = Block::default()
        .title(" New Task ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(outer_block, area);

    // Title field
    let title_style = if app.input_mode == InputMode::Editing {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let title_content = if app.new_task_title.is_empty() {
        Line::from(Span::styled(
            "Enter task title...",
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(Span::styled(&app.new_task_title, title_style))
    };

    let title_block = Block::default()
        .title(Span::styled(" Title ", Style::default().fg(Color::Cyan)))
        .borders(Borders::ALL)
        .border_style(if app.input_mode == InputMode::Editing {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let title_paragraph = Paragraph::new(title_content).block(title_block);
    frame.render_widget(title_paragraph, chunks[0]);

    // Description field
    let desc_content = if app.new_task_description.is_empty() {
        Line::from(Span::styled(
            "Enter task description (optional)...",
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(Span::styled(
            &app.new_task_description,
            Style::default().fg(Color::White),
        ))
    };

    let desc_block = Block::default()
        .title(Span::styled(" Description ", Style::default().fg(Color::Gray)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let desc_paragraph = Paragraph::new(desc_content).block(desc_block);
    frame.render_widget(desc_paragraph, chunks[2]);

    // Show cursor when editing
    if app.input_mode == InputMode::Editing {
        let cursor_x = chunks[0].x + 1 + app.new_task_title.len() as u16;
        let cursor_y = chunks[0].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
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
