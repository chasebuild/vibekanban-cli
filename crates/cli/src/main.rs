//! Vibe Kanban CLI - Terminal-first, real-time task viewer and creator.

mod cli_args;
mod render;
mod resolve;
mod utils;
mod watch;

use anyhow::{Context, Result, anyhow};
use clap::Parser;

use vibe_kanban_cli::{
    VibeKanbanClient,
    types::{CreateAndStartTaskRequest, CreateTask, ExecutorProfileId},
};

use crate::{
    cli_args::{Args, Command},
    resolve::{parse_uuid, resolve_project, resolve_repo_inputs},
    utils::{truncate_title},
    watch::{WatchFilter, watch_tasks},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Install rustls crypto provider before any TLS operations
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let args = Args::parse();

    // Initialize logging
    if args.debug {
        tracing_subscriber::fmt().with_env_filter("debug").init();
    }

    let client = VibeKanbanClient::new(&args.server).context("Failed to create API client")?;

    match args.command {
        Command::Create {
            project,
            prompt,
            title,
            status,
            tool,
            model,
            repos,
            branch,
            watch,
        } => {
            let project = resolve_project(&client, &project).await?;
            let executor = parse_executor(&tool)?;
            let status = parse_status(&status)?;
            let repo_inputs =
                resolve_repo_inputs(&client, project.id, repos, branch.as_deref()).await?;

            let task_title = title.unwrap_or_else(|| truncate_title(&prompt));
            let task = CreateTask {
                project_id: project.id,
                title: task_title,
                description: Some(prompt),
                status: Some(status),
                parent_workspace_id: None,
                image_ids: None,
                is_epic: None,
                complexity: None,
                metadata: None,
            };

            let executor_profile_id = ExecutorProfileId {
                executor,
                variant: model,
            };

            let request = CreateAndStartTaskRequest {
                task,
                executor_profile_id,
                repos: repo_inputs,
            };

            let created = client.create_and_start_task(&request).await?;
            let project_name = project.name.clone();
            println!(
                "Created task {} in project {}",
                created.task.id, project_name
            );

            if watch {
                watch_tasks(
                    &client,
                    &args.server,
                    WatchFilter::TaskId(created.task.id),
                    Some(project),
                )
                .await?;
            }
        }
        Command::Watch { project, task, slug } => {
            let filter = match (task, slug) {
                (Some(task_id), None) => WatchFilter::TaskId(parse_uuid(&task_id)?),
                (None, Some(slug)) => WatchFilter::Slug(slug),
                (None, None) => WatchFilter::None,
                _ => {
                    return Err(anyhow!(
                        "Use only one of --task or --slug when watching"
                    ));
                }
            };

            let project = match (&filter, project) {
                (WatchFilter::TaskId(task_id), _) => {
                    let task = client.get_task(*task_id).await?;
                    let project = client.get_project(task.project_id).await?;
                    Some(project)
                }
                (WatchFilter::Slug(_), Some(project_ref))
                | (WatchFilter::None, Some(project_ref)) => {
                    Some(resolve_project(&client, &project_ref).await?)
                }
                _ => None,
            };

            if matches!(filter, WatchFilter::Slug(_) | WatchFilter::None) && project.is_none() {
                return Err(anyhow!(
                    "--project is required when watching by slug or showing the board"
                ));
            }

            watch_tasks(&client, &args.server, filter, project).await?;
        }
    }

    Ok(())
}

fn parse_executor(input: &str) -> Result<vibe_kanban_cli::types::BaseCodingAgent> {
    let normalized = input.trim().to_lowercase();
    let executor = match normalized.as_str() {
        "claude" | "claude-code" | "claude_code" => vibe_kanban_cli::types::BaseCodingAgent::ClaudeCode,
        "amp" => vibe_kanban_cli::types::BaseCodingAgent::Amp,
        "gemini" => vibe_kanban_cli::types::BaseCodingAgent::Gemini,
        "codex" => vibe_kanban_cli::types::BaseCodingAgent::Codex,
        "opencode" | "open-code" | "open_code" => vibe_kanban_cli::types::BaseCodingAgent::Opencode,
        "cursor" | "cursor-agent" | "cursor_agent" => vibe_kanban_cli::types::BaseCodingAgent::CursorAgent,
        "qwen" | "qwen-code" | "qwen_code" => vibe_kanban_cli::types::BaseCodingAgent::QwenCode,
        "copilot" => vibe_kanban_cli::types::BaseCodingAgent::Copilot,
        "droid" => vibe_kanban_cli::types::BaseCodingAgent::Droid,
        _ => {
            return Err(anyhow!(
                "Unknown tool '{}'. Try codex, claude-code, cursor, gemini, opencode, qwen-code, amp, copilot, droid.",
                input
            ))
        }
    };
    Ok(executor)
}

fn parse_status(input: &str) -> Result<vibe_kanban_cli::types::TaskStatus> {
    let normalized = input.trim().to_lowercase();
    let status = match normalized.as_str() {
        "todo" => vibe_kanban_cli::types::TaskStatus::Todo,
        "inprogress" | "in-progress" => vibe_kanban_cli::types::TaskStatus::Inprogress,
        "inreview" | "in-review" => vibe_kanban_cli::types::TaskStatus::Inreview,
        "done" => vibe_kanban_cli::types::TaskStatus::Done,
        "cancelled" | "canceled" => vibe_kanban_cli::types::TaskStatus::Cancelled,
        _ => {
            return Err(anyhow!(
                "Unknown status '{}'. Try todo, inprogress, inreview, done, cancelled.",
                input
            ))
        }
    };
    Ok(status)
}
