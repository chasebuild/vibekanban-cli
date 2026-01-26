//! Reusable UI components.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

/// Render the header bar.
pub fn render_header(frame: &mut Frame, area: Rect, title: &str) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " Vibe Kanban CLI ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("│ "),
        Span::styled(title, Style::default().fg(Color::White)),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(header, area);
}

/// Render the status bar at the bottom.
pub fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let (message, style) = if let Some(ref err) = app.error_message {
        (err.as_str(), Style::default().fg(Color::Red))
    } else if let Some(ref status) = app.status_message {
        (status.as_str(), Style::default().fg(Color::Yellow))
    } else {
        ("Press ? for help", Style::default().fg(Color::DarkGray))
    };

    let status = Paragraph::new(Line::from(vec![Span::styled(message, style)]))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(status, area);
}

/// Render keyboard hints at the bottom.
pub fn render_hints(frame: &mut Frame, area: Rect, hints: &[(&str, &str)]) {
    let hint_spans: Vec<Span> = hints
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut spans = vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {} ", desc), Style::default().fg(Color::Gray)),
            ];
            if i < hints.len() - 1 {
                spans.push(Span::raw("│"));
            }
            spans
        })
        .collect();

    let hints_bar = Paragraph::new(Line::from(hint_spans))
        .alignment(Alignment::Center);

    frame.render_widget(hints_bar, area);
}

/// Style for selected items.
pub fn selected_style() -> Style {
    Style::default()
        .bg(Color::Rgb(40, 40, 60))
        .add_modifier(Modifier::BOLD)
}

/// Style for normal items.
pub fn normal_style() -> Style {
    Style::default()
}

/// Style for focused borders.
pub fn focused_border_style() -> Style {
    Style::default().fg(Color::Cyan)
}

/// Style for unfocused borders.
pub fn unfocused_border_style() -> Style {
    Style::default().fg(Color::DarkGray)
}
