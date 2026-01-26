//! Projects list view.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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

    // Header
    render_header(frame, chunks[0], "Projects");

    // Content area with project list and details
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    render_project_list(frame, content_chunks[0], app);
    render_project_details(frame, content_chunks[1], app);

    // Hints
    render_hints(
        frame,
        chunks[2],
        &[
            ("↑/↓", "Navigate"),
            ("Enter", "Select"),
            ("n", "New Project"),
            ("q", "Quit"),
            ("?", "Help"),
        ],
    );

    // Status bar
    render_status_bar(frame, chunks[3], app);
}

fn render_project_list(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .projects
        .iter()
        .enumerate()
        .map(|(i, project)| {
            let style = if i == app.selected_project_index {
                selected_style()
            } else {
                Style::default()
            };

            let marker = if i == app.selected_project_index {
                "▸ "
            } else {
                "  "
            };

            ListItem::new(Line::from(vec![
                Span::styled(marker, style),
                Span::styled(&project.name, style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Projects ")
                .borders(Borders::ALL)
                .border_style(focused_border_style()),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Rgb(40, 40, 60)),
        );

    frame.render_widget(list, area);
}

fn render_project_details(frame: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(project) = app.projects.get(app.selected_project_index) {
        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Gray)),
                Span::styled(&project.name, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    project.id.to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Created: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    project.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Updated: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    project.updated_at.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No project selected",
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
