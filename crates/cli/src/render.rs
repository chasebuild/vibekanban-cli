use std::io::{self, Write};

use anyhow::Result;
use crossterm::{
    cursor,
    execute,
    terminal::{Clear, ClearType},
};

use crate::{
    watch::{WatchFilter, select_task_by_filter},
    utils::{pad_truncate, task_slug, yes_no},
};
use vibe_kanban_cli::types::{TaskStatus, TaskWithAttemptStatus};

pub fn render_view(project_name: &str, tasks: &[TaskWithAttemptStatus], filter: &WatchFilter) -> String {
    match filter {
        WatchFilter::TaskId(_) | WatchFilter::Slug(_) => {
            let match_task = select_task_by_filter(tasks, filter);
            let label = match filter {
                WatchFilter::TaskId(task_id) => Some(format!("task id {}", task_id)),
                WatchFilter::Slug(slug) => Some(format!("slug {}", slug)),
                _ => None,
            };
            render_task_detail(project_name, match_task, label)
        }
        WatchFilter::None => render_board(project_name, tasks),
    }
}

pub fn render_task_detail(
    project_name: &str,
    task: Option<&TaskWithAttemptStatus>,
    target: Option<String>,
) -> String {
    let mut out = String::new();
    out.push_str(&render_header(project_name, "Watching task"));
    if let Some(target) = target {
        out.push_str(&format!("Target: {}\n\n", target));
    }

    match task {
        Some(task) => {
            out.push_str(&format!("Title: {}\n", task.task.title));
            out.push_str(&format!("Task ID: {}\n", task.task.id));
            out.push_str(&format!("Status: {}\n", task.task.status.display_name()));
            out.push_str(&format!(
                "Executor: {}\n",
                if task.executor.is_empty() { "unknown" } else { task.executor.as_str() }
            ));
            out.push_str(&format!(
                "Has running attempt: {}\n",
                yes_no(task.has_in_progress_attempt)
            ));
            out.push_str(&format!(
                "Last attempt failed: {}\n",
                yes_no(task.last_attempt_failed)
            ));
            out.push_str(&format!("Created: {}\n", task.task.created_at));
            out.push_str(&format!("Updated: {}\n", task.task.updated_at));
            out.push_str(&format!("Slug: {}\n", task_slug(&task.task.title)));
        }
        None => {
            out.push_str("Waiting for matching task...\n");
        }
    }

    out.push_str("\nPress Ctrl+C to exit.\n");
    out
}

pub fn render_board(project_name: &str, tasks: &[TaskWithAttemptStatus]) -> String {
    let (width, _) = crossterm::terminal::size().unwrap_or((120, 40));
    let width = width as usize;
    let min_col_width = 20usize;
    let gap = 2usize;
    let col_width = width.saturating_sub(gap * 3) / 4;

    if col_width < min_col_width {
        return render_board_stacked(project_name, tasks);
    }

    let mut columns: Vec<(&str, TaskStatus, Vec<String>)> = vec![
        ("To Do", TaskStatus::Todo, Vec::new()),
        ("In Progress", TaskStatus::Inprogress, Vec::new()),
        ("In Review", TaskStatus::Inreview, Vec::new()),
        ("Done", TaskStatus::Done, Vec::new()),
    ];

    for task in tasks {
        for (_, status, lines) in &mut columns {
            if task.task.status == *status {
                lines.push(format_task_line(task));
            }
        }
    }

    let mut out = String::new();
    out.push_str(&render_header(project_name, "Live Board"));

    let headers: Vec<String> = columns
        .iter()
        .map(|(name, _, lines)| format!("{} ({})", name, lines.len()))
        .map(|s| pad_truncate(&s, col_width))
        .collect();
    out.push_str(&headers.join(&" ".repeat(gap)));
    out.push('\n');
    out.push_str(&headers.iter().map(|_| "-".repeat(col_width)).collect::<Vec<_>>().join(&" ".repeat(gap)));
    out.push('\n');

    let max_lines = columns.iter().map(|(_, _, lines)| lines.len()).max().unwrap_or(0);
    for i in 0..max_lines {
        let row: Vec<String> = columns
            .iter()
            .map(|(_, _, lines)| {
                lines
                    .get(i)
                    .map(|line| pad_truncate(line, col_width))
                    .unwrap_or_else(|| " ".repeat(col_width))
            })
            .collect();
        out.push_str(&row.join(&" ".repeat(gap)));
        out.push('\n');
    }

    out.push_str("\nPress Ctrl+C to exit.\n");
    out
}

pub fn render_board_stacked(project_name: &str, tasks: &[TaskWithAttemptStatus]) -> String {
    let mut out = String::new();
    out.push_str(&render_header(project_name, "Live Board"));

    let statuses = [
        ("To Do", TaskStatus::Todo),
        ("In Progress", TaskStatus::Inprogress),
        ("In Review", TaskStatus::Inreview),
        ("Done", TaskStatus::Done),
    ];

    for (name, status) in statuses {
        let mut section: Vec<&TaskWithAttemptStatus> =
            tasks.iter().filter(|t| t.task.status == status).collect();
        section.sort_by(|a, b| a.task.title.cmp(&b.task.title));
        out.push_str(&format!("{} ({})\n", name, section.len()));
        out.push_str(&"-".repeat(name.len().max(4) + 4));
        out.push('\n');
        if section.is_empty() {
            out.push_str("  (none)\n\n");
            continue;
        }
        for task in section {
            out.push_str(&format!("  {}\n", format_task_line(task)));
        }
        out.push('\n');
    }

    out.push_str("Press Ctrl+C to exit.\n");
    out
}

pub fn render_header(project_name: &str, subtitle: &str) -> String {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let mut out = String::new();
    out.push_str(&format!(
        "Vibe Kanban  |  {}  |  {}  |  {}\n",
        project_name, subtitle, now
    ));
    out.push_str(&"=".repeat(80));
    out.push('\n');
    out
}

pub fn format_task_line(task: &TaskWithAttemptStatus) -> String {
    let mut flags = Vec::new();
    if task.has_in_progress_attempt {
        flags.push("RUN");
    }
    if task.last_attempt_failed {
        flags.push("FAIL");
    }
    let flag_str = if flags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", flags.join("|"))
    };
    let executor = if task.executor.is_empty() {
        String::new()
    } else {
        format!(" ({})", task.executor)
    };
    format!("{}{}{}", task.task.title, executor, flag_str)
}

pub fn draw_screen(output: &str) -> Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    stdout.write_all(output.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

pub fn tasks_from_state(state: &serde_json::Value) -> Vec<TaskWithAttemptStatus> {
    let mut tasks = Vec::new();
    if let Some(map) = state.get("tasks").and_then(|v| v.as_object()) {
        for value in map.values() {
            if let Ok(task) = serde_json::from_value::<TaskWithAttemptStatus>(value.clone()) {
                tasks.push(task);
            }
        }
    }
    tasks
}
