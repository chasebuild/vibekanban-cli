//! Create attempt view.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::{
    app::App,
    ui::components::{focused_border_style, render_header, render_hints, render_status_bar, selected_style},
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
    let title = if let Some(ref task) = app.selected_task {
        format!("Create Attempt - {}", task.task.title)
    } else {
        "Create Attempt".to_string()
    };
    render_header(frame, chunks[0], &title);

    // Content area
    render_form(frame, chunks[1], app);

    // Hints
    render_hints(
        frame,
        chunks[2],
        &[
            ("↑/↓", "Navigate"),
            ("Enter", "Select/Edit"),
            ("Tab", "Next Field"),
            ("Esc", "Cancel"),
        ],
    );

    // Status bar
    render_status_bar(frame, chunks[3], app);
}

fn render_form(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Executor
            Constraint::Length(3),  // Variant
            Constraint::Min(5),     // Repo branches
        ])
        .split(area);

    // Executor selection
    let executors = App::available_executors();
    let executor_items: Vec<ListItem> = executors
        .iter()
        .enumerate()
        .map(|(i, exec)| {
            let style = if i == app.attempt_executor_index && app.attempt_selected_field == 0 {
                selected_style()
            } else {
                Style::default()
            };
            let marker = if i == app.attempt_executor_index && app.attempt_selected_field == 0 {
                "▸ "
            } else {
                "  "
            };
            ListItem::new(Line::from(vec![
                Span::styled(marker, style),
                Span::styled(format!("{:?}", exec), style),
            ]))
        })
        .collect();

    let executor_list = List::new(executor_items)
        .block(
            Block::default()
                .title(if app.attempt_selected_field == 0 {
                    " Executor * "
                } else {
                    " Executor * "
                })
                .borders(Borders::ALL)
                .border_style(if app.attempt_selected_field == 0 {
                    focused_border_style()
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
        );

    frame.render_widget(executor_list, chunks[0]);

    // Variant input
    let variant_text = app.attempt_variant.as_deref().unwrap_or("(optional)");
    let variant_style = if app.attempt_selected_field == 1 {
        focused_border_style()
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let variant_paragraph = Paragraph::new(variant_text)
        .block(
            Block::default()
                .title(" Variant ")
                .borders(Borders::ALL)
                .border_style(variant_style),
        )
        .style(if app.attempt_selected_field == 1 {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    frame.render_widget(variant_paragraph, chunks[1]);

    // Repo branches
    let repo_items: Vec<ListItem> = app
        .attempt_repo_branches
        .iter()
        .enumerate()
        .map(|(i, (repo_id, branch))| {
            let repo_name = app
                .project_repos
                .iter()
                .find(|r| r.id == *repo_id)
                .map(|r| r.name.as_str())
                .unwrap_or("Unknown");
            
            let field_index = 2 + i;
            let style = if field_index == app.attempt_selected_field {
                selected_style()
            } else {
                Style::default()
            };
            let marker = if field_index == app.attempt_selected_field {
                "▸ "
            } else {
                "  "
            };
            
            // Find available branches for this repo
            let empty_branches = Vec::new();
            let branches = app
                .repo_branches_cache
                .iter()
                .find(|(id, _)| *id == *repo_id)
                .map(|(_, branches)| branches)
                .unwrap_or(&empty_branches);
            
            let branch_display = if branches.iter().any(|b| b.name == *branch) {
                branch.clone()
            } else {
                format!("{} (custom)", branch)
            };

            ListItem::new(Line::from(vec![
                Span::styled(marker, style),
                Span::styled(format!("{}: ", repo_name), Style::default().fg(Color::Gray)),
                Span::styled(branch_display, style),
            ]))
        })
        .collect();

    let repo_list = List::new(repo_items)
        .block(
            Block::default()
                .title(if app.attempt_selected_field >= 2 {
                    " Base Branches * "
                } else {
                    " Base Branches * "
                })
                .borders(Borders::ALL)
                .border_style(if app.attempt_selected_field >= 2 {
                    focused_border_style()
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
        );

    frame.render_widget(repo_list, chunks[2]);
}

