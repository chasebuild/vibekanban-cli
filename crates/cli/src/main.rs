//! Vibe Kanban CLI - Main entry point.

use std::io::stdout;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use vibe_kanban_cli::{
    App, VibeKanbanClient,
    app::{InputMode, View},
};

/// Vibe Kanban CLI - Interactive terminal interface
#[derive(Parser, Debug)]
#[command(name = "vibe-kanban-cli")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Vibe Kanban server URL
    #[arg(short, long, default_value = "http://localhost:5173")]
    server: String,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install rustls crypto provider before any TLS operations
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let args = Args::parse();

    // Initialize logging
    if args.debug {
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::new("debug"))
            .init();
    }

    // Create API client
    let client = VibeKanbanClient::new(&args.server).context("Failed to create API client")?;

    // Check server health
    if !client.health_check().await.unwrap_or(false) {
        eprintln!(
            "Warning: Could not connect to Vibe Kanban server at {}",
            args.server
        );
        eprintln!("Make sure the server is running and try again.");
        eprintln!();
        eprintln!("Starting anyway - you can retry with 'r' to refresh.");
    }

    // Create app state
    let mut app = App::new(client);

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Load initial data
    if let Err(e) = app.load_projects().await {
        app.set_error(format!("Failed to load projects: {}", e));
    }

    // Main event loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Render UI
        terminal.draw(|frame| vibe_kanban_cli::ui::render(frame, app))?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle Ctrl+C globally
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    app.should_quit = true;
                }

                // Handle input mode
                if app.input_mode == InputMode::Editing {
                    handle_editing_input(app, key.code).await?;
                } else {
                    handle_normal_input(app, key.code).await?;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

async fn handle_normal_input(app: &mut App, key: KeyCode) -> Result<()> {
    match key {
        // Global keys
        KeyCode::Char('q') => {
            if app.view == View::Projects {
                app.should_quit = true;
            } else {
                app.go_back();
            }
        }
        KeyCode::Char('?') => {
            app.navigate_to(View::Help);
        }
        KeyCode::Char('r') => {
            // Refresh current view
            match app.view {
                View::Projects => {
                    if let Err(e) = app.load_projects().await {
                        app.set_error(format!("Failed to refresh: {}", e));
                    }
                }
                View::Tasks => {
                    if let Err(e) = app.load_tasks().await {
                        app.set_error(format!("Failed to refresh: {}", e));
                    }
                }
                View::Workspaces => {
                    if let Err(e) = app.load_workspaces().await {
                        app.set_error(format!("Failed to refresh: {}", e));
                    }
                }
                View::WorkspaceDetail => {
                    if let Err(e) = app.load_workspace_details().await {
                        app.set_error(format!("Failed to refresh: {}", e));
                    }
                }
                _ => {}
            }
        }
        KeyCode::Esc => {
            app.go_back();
        }

        // Navigation
        KeyCode::Up | KeyCode::Char('k') => {
            if app.view == View::CreateAttempt {
                handle_create_attempt_navigation(app, -1);
            } else {
                app.move_up();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.view == View::CreateAttempt {
                handle_create_attempt_navigation(app, 1);
            } else {
                app.move_down();
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.move_left();
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.move_right();
        }
        KeyCode::Tab => {
            if app.view == View::CreateAttempt {
                handle_create_attempt_tab(app);
            }
        }

        // View-specific keys
        KeyCode::Enter => {
            if app.view == View::CreateAttempt {
                handle_create_attempt_enter(app).await?;
            } else {
                handle_enter(app).await?;
            }
        }
        KeyCode::Char('n') => {
            handle_new(app).await?;
        }
        KeyCode::Char('d') => {
            handle_delete(app).await?;
        }
        KeyCode::Char('m') => {
            handle_merge_or_move(app).await?;
        }
        KeyCode::Char('p') => {
            handle_push(app).await?;
        }
        KeyCode::Char('b') => {
            handle_rebase(app).await?;
        }
        KeyCode::Char('s') => {
            handle_stop(app).await?;
        }

        _ => {}
    }
    Ok(())
}

async fn handle_editing_input(app: &mut App, key: KeyCode) -> Result<()> {
    match key {
        KeyCode::Esc => {
            if app.view == View::CreateAttempt {
                app.input_mode = InputMode::Normal;
            } else {
                app.input_mode = InputMode::Normal;
            }
        }
        KeyCode::Enter => {
            if app.view == View::CreateTask {
                if let Err(e) = app.create_task().await {
                    app.set_error(format!("Failed to create task: {}", e));
                }
                app.input_mode = InputMode::Normal;
            } else if app.view == View::CreateAttempt {
                if app.attempt_selected_field == 1 {
                    // Variant field - create attempt
                    if let Err(e) = app.create_attempt().await {
                        app.set_error(format!("Failed to create attempt: {}", e));
                    }
                } else if app.attempt_selected_field >= 2 {
                    // Branch field - finish editing
                    app.input_mode = InputMode::Normal;
                }
            }
        }
        KeyCode::Backspace => {
            if app.view == View::CreateTask {
                app.new_task_title.pop();
            } else if app.view == View::CreateAttempt {
                if app.attempt_selected_field == 1 {
                    // Variant field
                    if let Some(ref mut variant) = app.attempt_variant {
                        variant.pop();
                        if variant.is_empty() {
                            app.attempt_variant = None;
                        }
                    }
                } else if app.attempt_selected_field >= 2 {
                    // Branch field
                    let repo_index = app.attempt_selected_field - 2;
                    if repo_index < app.attempt_repo_branches.len() {
                        app.attempt_repo_branches[repo_index].1.pop();
                    }
                }
            }
        }
        KeyCode::Char(c) => {
            if app.view == View::CreateTask {
                app.new_task_title.push(c);
            } else if app.view == View::CreateAttempt {
                if app.attempt_selected_field == 1 {
                    // Variant field
                    if app.attempt_variant.is_none() {
                        app.attempt_variant = Some(String::new());
                    }
                    if let Some(ref mut variant) = app.attempt_variant {
                        variant.push(c);
                    }
                } else if app.attempt_selected_field >= 2 {
                    // Branch field
                    let repo_index = app.attempt_selected_field - 2;
                    if repo_index < app.attempt_repo_branches.len() {
                        app.attempt_repo_branches[repo_index].1.push(c);
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_enter(app: &mut App) -> Result<()> {
    match app.view {
        View::Projects => {
            if let Err(e) = app.select_project().await {
                app.set_error(format!("Failed to select project: {}", e));
            }
        }
        View::Tasks => {
            if let Err(e) = app.select_task().await {
                app.set_error(format!("Failed to select task: {}", e));
            }
        }
        View::Workspaces => {
            if let Err(e) = app.select_workspace().await {
                app.set_error(format!("Failed to select workspace: {}", e));
            }
        }
        View::CreateTask => {
            app.input_mode = InputMode::Editing;
        }
        View::CreateAttempt => {
            // Handled in handle_create_attempt_enter
        }
        View::Help => {
            app.go_back();
        }
        _ => {}
    }
    Ok(())
}

async fn handle_new(app: &mut App) -> Result<()> {
    match app.view {
        View::Tasks => {
            app.navigate_to(View::CreateTask);
            app.input_mode = InputMode::Editing;
        }
        View::Workspaces => {
            if app.selected_task.is_some() {
                if let Err(e) = app.init_create_attempt().await {
                    app.set_error(format!("Failed to initialize: {}", e));
                } else {
                    app.navigate_to(View::CreateAttempt);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_delete(app: &mut App) -> Result<()> {
    match app.view {
        View::Tasks => {
            if let Err(e) = app.delete_selected_task().await {
                app.set_error(format!("Failed to delete task: {}", e));
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_merge_or_move(app: &mut App) -> Result<()> {
    match app.view {
        View::Tasks => {
            // Move task to next status
            if let Some(task) = app.current_column_selected_task() {
                let next_status = match task.task.status {
                    vibe_kanban_cli::types::TaskStatus::Todo => {
                        vibe_kanban_cli::types::TaskStatus::Inprogress
                    }
                    vibe_kanban_cli::types::TaskStatus::Inprogress => {
                        vibe_kanban_cli::types::TaskStatus::Inreview
                    }
                    vibe_kanban_cli::types::TaskStatus::Inreview => {
                        vibe_kanban_cli::types::TaskStatus::Done
                    }
                    vibe_kanban_cli::types::TaskStatus::Done => {
                        vibe_kanban_cli::types::TaskStatus::Done
                    }
                    vibe_kanban_cli::types::TaskStatus::Cancelled => {
                        vibe_kanban_cli::types::TaskStatus::Cancelled
                    }
                };
                let task_id = task.task.id;
                if let Err(e) = app.update_task_status(task_id, next_status).await {
                    app.set_error(format!("Failed to move task: {}", e));
                }
            }
        }
        View::WorkspaceDetail => {
            if let Err(e) = app.merge_workspace().await {
                app.set_error(format!("Failed to merge: {}", e));
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_push(app: &mut App) -> Result<()> {
    if app.view == View::WorkspaceDetail {
        if let Err(e) = app.push_workspace().await {
            app.set_error(format!("Failed to push: {}", e));
        }
    }
    Ok(())
}

async fn handle_rebase(app: &mut App) -> Result<()> {
    if app.view == View::WorkspaceDetail {
        if let Err(e) = app.rebase_workspace().await {
            app.set_error(format!("Failed to rebase: {}", e));
        }
    }
    Ok(())
}

async fn handle_stop(app: &mut App) -> Result<()> {
    match app.view {
        View::Workspaces | View::WorkspaceDetail => {
            if let Err(e) = app.stop_workspace().await {
                app.set_error(format!("Failed to stop: {}", e));
            }
        }
        _ => {}
    }
    Ok(())
}

// =========================================================================
// Create Attempt Handlers
// =========================================================================

fn handle_create_attempt_navigation(app: &mut App, delta: i32) {
    match app.attempt_selected_field {
        0 => {
            // Executor selection
            let executors = App::available_executors();
            let new_index = (app.attempt_executor_index as i32 + delta)
                .max(0)
                .min(executors.len() as i32 - 1) as usize;
            app.attempt_executor_index = new_index;
        }
        1 => {
            // Variant field - can't navigate up/down, use Tab
        }
        _ => {
            // Repo branch selection
            let repo_index = app.attempt_selected_field - 2;
            if repo_index < app.attempt_repo_branches.len() {
                let (repo_id, current_branch) = &app.attempt_repo_branches[repo_index];

                // Find branches for this repo
                if let Some((_, branches)) = app
                    .repo_branches_cache
                    .iter()
                    .find(|(id, _)| *id == *repo_id)
                {
                    if let Some(current_pos) =
                        branches.iter().position(|b| b.name == *current_branch)
                    {
                        let new_pos = (current_pos as i32 + delta)
                            .max(0)
                            .min(branches.len() as i32 - 1)
                            as usize;
                        if let Some(new_branch) = branches.get(new_pos) {
                            app.attempt_repo_branches[repo_index].1 = new_branch.name.clone();
                        }
                    } else if !branches.is_empty() {
                        // Current branch not in list, select first
                        app.attempt_repo_branches[repo_index].1 = branches[0].name.clone();
                    }
                }
            }
        }
    }
}

fn handle_create_attempt_tab(app: &mut App) {
    let max_field = 1 + app.attempt_repo_branches.len(); // executor + variant + repos
    app.attempt_selected_field = (app.attempt_selected_field + 1) % max_field;

    // If moving to variant field, enter editing mode
    if app.attempt_selected_field == 1 {
        app.input_mode = InputMode::Editing;
    } else {
        app.input_mode = InputMode::Normal;
    }
}

async fn handle_create_attempt_enter(app: &mut App) -> Result<()> {
    match app.attempt_selected_field {
        0 => {
            // Executor selected - move to variant
            handle_create_attempt_tab(app);
        }
        1 => {
            // Variant field - create attempt
            if let Err(e) = app.create_attempt().await {
                app.set_error(format!("Failed to create attempt: {}", e));
            }
        }
        _ => {
            // Repo branch selected - allow editing branch name
            app.input_mode = InputMode::Editing;
        }
    }
    Ok(())
}
