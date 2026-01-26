//! Tasks kanban board view.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::{
    app::{App, TaskColumn},
    types::TaskStatus,
    ui::components::{
        focused_border_style, render_header, render_hints, render_status_bar, selected_style,
        unfocused_border_style,
    },
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header
            Constraint::Min(10),    // Kanban board
            Constraint::Length(2),  // Hints
            Constraint::Length(2),  // Status
        ])
        .split(frame.area());

    // Header with project name
    let title = if let Some(ref project) = app.selected_project {
        format!("Tasks - {}", project.name)
    } else {
        "Tasks".to_string()
    };
    render_header(frame, chunks[0], &title);

    // Kanban board (4 columns)
    let board_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);

    render_column(frame, board_chunks[0], app, TaskColumn::Todo);
    render_column(frame, board_chunks[1], app, TaskColumn::InProgress);
    render_column(frame, board_chunks[2], app, TaskColumn::InReview);
    render_column(frame, board_chunks[3], app, TaskColumn::Done);

    // Hints
    render_hints(
        frame,
        chunks[2],
        &[
            ("←/→", "Column"),
            ("↑/↓", "Task"),
            ("Enter", "View"),
            ("n", "New Task"),
            ("m", "Move"),
            ("Esc", "Back"),
        ],
    );

    // Status bar
    render_status_bar(frame, chunks[3], app);
}

fn render_column(frame: &mut Frame, area: Rect, app: &App, column: TaskColumn) {
    let is_focused = app.selected_column == column;
    let column_index = match column {
        TaskColumn::Todo => 0,
        TaskColumn::InProgress => 1,
        TaskColumn::InReview => 2,
        TaskColumn::Done => 3,
    };
    let selected_index = app.selected_task_indices[column_index];

    let tasks = app.tasks_for_column(column);

    let items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let is_selected = is_focused && i == selected_index;
            let style = if is_selected {
                selected_style()
            } else {
                Style::default()
            };

            let marker = if is_selected { "▸ " } else { "  " };

            // Status indicator
            let status_indicator = if task.has_in_progress_attempt {
                Span::styled("● ", Style::default().fg(Color::Green))
            } else if task.last_attempt_failed {
                Span::styled("✗ ", Style::default().fg(Color::Red))
            } else {
                Span::raw("  ")
            };

            // Truncate title if too long
            let max_len = area.width.saturating_sub(8) as usize;
            let title = if task.task.title.len() > max_len {
                format!("{}...", &task.task.title[..max_len.saturating_sub(3)])
            } else {
                task.task.title.clone()
            };

            ListItem::new(Line::from(vec![
                Span::styled(marker, style),
                status_indicator,
                Span::styled(title, style),
            ]))
        })
        .collect();

    let border_style = if is_focused {
        focused_border_style()
    } else {
        unfocused_border_style()
    };

    let title_style = if is_focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let title = format!(" {} ({}) ", column.title(), tasks.len());

    let list = List::new(items).block(
        Block::default()
            .title(Span::styled(title, title_style))
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    frame.render_widget(list, area);
}

/// Get color for task status.
#[allow(dead_code)]
fn status_color(status: TaskStatus) -> Color {
    match status {
        TaskStatus::Todo => Color::Gray,
        TaskStatus::Inprogress => Color::Yellow,
        TaskStatus::Inreview => Color::Magenta,
        TaskStatus::Done => Color::Green,
        TaskStatus::Cancelled => Color::Red,
    }
}
