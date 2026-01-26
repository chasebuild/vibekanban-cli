//! Workspace detail view with git operations.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::{
    app::App,
    ui::components::{render_header, render_hints, render_status_bar},
};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header
            Constraint::Length(3),  // Tab bar
            Constraint::Min(10),    // Content
            Constraint::Length(2),  // Hints
            Constraint::Length(2),  // Status
        ])
        .split(frame.area());

    // Header with workspace branch
    let title = if let Some(ref workspace) = app.selected_workspace {
        format!("Workspace - {}", workspace.branch)
    } else {
        "Workspace Detail".to_string()
    };
    render_header(frame, chunks[0], &title);

    // Tab bar
    render_tabs(frame, chunks[1]);

    // Content area
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    render_branch_status(frame, content_chunks[0], app);
    render_session_info(frame, content_chunks[1], app);

    // Hints
    render_hints(
        frame,
        chunks[3],
        &[
            ("m", "Merge"),
            ("p", "Push"),
            ("r", "Rebase"),
            ("s", "Stop"),
            ("f", "Follow-up"),
            ("Esc", "Back"),
        ],
    );

    // Status bar
    render_status_bar(frame, chunks[4], app);
}

fn render_tabs(frame: &mut Frame, area: Rect) {
    let titles = vec!["Overview", "Diff", "Sessions", "Branches"];
    let tabs = Tabs::new(titles)
        .select(0)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider("│");

    frame.render_widget(tabs, area);
}

fn render_branch_status(frame: &mut Frame, area: Rect, app: &App) {
    let mut content = vec![];

    if let Some(ref workspace) = app.selected_workspace {
        content.push(Line::from(vec![
            Span::styled("Branch: ", Style::default().fg(Color::Gray)),
            Span::styled(&workspace.branch, Style::default().fg(Color::Cyan)),
        ]));
        content.push(Line::from(""));
    }

    // Branch statuses for each repo
    for status in &app.branch_statuses {
        content.push(Line::from(vec![
            Span::styled("Repo: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &status.repo_name,
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]));

        content.push(Line::from(vec![
            Span::styled("  Target: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &status.status.target_branch_name,
                Style::default().fg(Color::Yellow),
            ),
        ]));

        // Commit status
        if let (Some(ahead), Some(behind)) = (
            status.status.commits_ahead,
            status.status.commits_behind,
        ) {
            let ahead_style = if ahead > 0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let behind_style = if behind > 0 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            content.push(Line::from(vec![
                Span::styled("  Commits: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("+{}", ahead), ahead_style),
                Span::raw(" / "),
                Span::styled(format!("-{}", behind), behind_style),
            ]));
        }

        // Uncommitted changes
        if let Some(uncommitted) = status.status.uncommitted_count {
            let style = if uncommitted > 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            content.push(Line::from(vec![
                Span::styled("  Uncommitted: ", Style::default().fg(Color::Gray)),
                Span::styled(uncommitted.to_string(), style),
            ]));
        }

        // Conflicts
        if !status.status.conflicted_files.is_empty() {
            content.push(Line::from(vec![
                Span::styled("  ⚠ Conflicts: ", Style::default().fg(Color::Red)),
                Span::styled(
                    status.status.conflicted_files.len().to_string(),
                    Style::default().fg(Color::Red),
                ),
            ]));
        }

        // Rebase in progress
        if status.status.is_rebase_in_progress {
            content.push(Line::from(Span::styled(
                "  ⚠ Rebase in progress",
                Style::default().fg(Color::Yellow),
            )));
        }

        content.push(Line::from(""));
    }

    if app.branch_statuses.is_empty() {
        content.push(Line::from(Span::styled(
            "No repository information available",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(content).block(
        Block::default()
            .title(" Git Status ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(paragraph, area);
}

fn render_session_info(frame: &mut Frame, area: Rect, app: &App) {
    let mut content = vec![];

    content.push(Line::from(vec![
        Span::styled("Sessions: ", Style::default().fg(Color::Gray)),
        Span::styled(
            app.sessions.len().to_string(),
            Style::default().fg(Color::White),
        ),
    ]));
    content.push(Line::from(""));

    // List sessions
    for (i, session) in app.sessions.iter().enumerate().take(10) {
        let executor = session.executor.as_deref().unwrap_or("unknown");
        content.push(Line::from(vec![
            Span::styled(
                format!("  {}. ", i + 1),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(executor, Style::default().fg(Color::Cyan)),
        ]));
        content.push(Line::from(vec![
            Span::styled("     Created: ", Style::default().fg(Color::Gray)),
            Span::styled(&session.created_at, Style::default().fg(Color::DarkGray)),
        ]));
    }

    if app.sessions.len() > 10 {
        content.push(Line::from(Span::styled(
            format!("  ... and {} more", app.sessions.len() - 10),
            Style::default().fg(Color::DarkGray),
        )));
    }

    if app.sessions.is_empty() {
        content.push(Line::from(Span::styled(
            "No sessions yet",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Workspace repos
    content.push(Line::from(""));
    content.push(Line::from(vec![
        Span::styled("Repositories: ", Style::default().fg(Color::Gray)),
        Span::styled(
            app.workspace_repos.len().to_string(),
            Style::default().fg(Color::White),
        ),
    ]));

    for repo in &app.workspace_repos {
        content.push(Line::from(vec![
            Span::styled("  • ", Style::default().fg(Color::DarkGray)),
            Span::styled(&repo.repo.display_name, Style::default().fg(Color::White)),
            Span::styled(" → ", Style::default().fg(Color::DarkGray)),
            Span::styled(&repo.target_branch, Style::default().fg(Color::Yellow)),
        ]));
    }

    let paragraph = Paragraph::new(content).block(
        Block::default()
            .title(" Session Info ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(paragraph, area);
}
