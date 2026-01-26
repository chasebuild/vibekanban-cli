//! Workspaces (task attempts) list view.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::{
    app::App,
    ui::components::{
        focused_border_style, render_header, render_hints, render_status_bar, selected_style,
    },
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header
            Constraint::Min(10),    // Content
            Constraint::Length(2),  // Hints
            Constraint::Length(2),  // Status
        ])
        .split(frame.area());

    // Header with task name
    let title = if let Some(ref task) = app.selected_task {
        format!("Workspaces - {}", task.task.title)
    } else {
        "Workspaces".to_string()
    };
    render_header(frame, chunks[0], &title);

    // Content area with workspace list and details
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    render_workspace_list(frame, content_chunks[0], app);
    render_workspace_details(frame, content_chunks[1], app);

    // Hints
    render_hints(
        frame,
        chunks[2],
        &[
            ("↑/↓", "Navigate"),
            ("Enter", "View Details"),
            ("n", "New Attempt"),
            ("s", "Stop"),
            ("Esc", "Back"),
        ],
    );

    // Status bar
    render_status_bar(frame, chunks[3], app);
}

fn render_workspace_list(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .workspaces
        .iter()
        .enumerate()
        .map(|(i, workspace)| {
            let style = if i == app.selected_workspace_index {
                selected_style()
            } else {
                Style::default()
            };

            let marker = if i == app.selected_workspace_index {
                "▸ "
            } else {
                "  "
            };

            // Status indicator
            let status_icon = if workspace.archived {
                Span::styled("⊘ ", Style::default().fg(Color::DarkGray))
            } else if workspace.pinned {
                Span::styled("★ ", Style::default().fg(Color::Yellow))
            } else {
                Span::styled("● ", Style::default().fg(Color::Green))
            };

            // Workspace name or branch
            let name = workspace
                .name
                .as_ref()
                .map(|n| n.as_str())
                .unwrap_or(&workspace.branch);

            // Truncate if too long
            let max_len = area.width.saturating_sub(10) as usize;
            let display_name = if name.len() > max_len {
                format!("{}...", &name[..max_len.saturating_sub(3)])
            } else {
                name.to_string()
            };

            ListItem::new(Line::from(vec![
                Span::styled(marker, style),
                status_icon,
                Span::styled(display_name, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Workspaces ({}) ", app.workspaces.len()))
            .borders(Borders::ALL)
            .border_style(focused_border_style()),
    );

    frame.render_widget(list, area);
}

fn render_workspace_details(frame: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(workspace) = app.workspaces.get(app.selected_workspace_index) {
        vec![
            Line::from(vec![
                Span::styled("Branch: ", Style::default().fg(Color::Gray)),
                Span::styled(&workspace.branch, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    workspace.id.to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Gray)),
                if workspace.archived {
                    Span::styled("Archived", Style::default().fg(Color::DarkGray))
                } else if workspace.pinned {
                    Span::styled("Pinned", Style::default().fg(Color::Yellow))
                } else {
                    Span::styled("Active", Style::default().fg(Color::Green))
                },
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Created: ", Style::default().fg(Color::Gray)),
                Span::styled(&workspace.created_at, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            if let Some(ref container) = workspace.container_ref {
                Line::from(vec![
                    Span::styled("Container: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        if container.len() > 40 {
                            format!("...{}", &container[container.len() - 37..])
                        } else {
                            container.clone()
                        },
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled("Container: ", Style::default().fg(Color::Gray)),
                    Span::styled("Not initialized", Style::default().fg(Color::DarkGray)),
                ])
            },
        ]
    } else {
        vec![Line::from(Span::styled(
            "No workspace selected",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let paragraph = Paragraph::new(content).block(
        Block::default()
            .title(" Details ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(paragraph, area);
}
