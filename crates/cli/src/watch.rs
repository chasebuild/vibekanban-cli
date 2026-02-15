use anyhow::{Context, Result, anyhow};
use futures_util::StreamExt;
use json_patch::Patch;
use tokio::select;
use tokio_tungstenite::connect_async;

use crate::{
    render::{render_view, render_header, draw_screen, tasks_from_state},
    resolve::tasks_ws_url,
    utils::task_slug,
    VibeKanbanClient,
};
use vibe_kanban_cli::types::{Project, TaskWithAttemptStatus};

#[derive(Clone, Debug)]
pub enum WatchFilter {
    None,
    TaskId(uuid::Uuid),
    Slug(String),
}

pub async fn watch_tasks(
    client: &VibeKanbanClient,
    server: &str,
    filter: WatchFilter,
    project: Option<Project>,
) -> Result<()> {
    let project = match (&filter, project) {
        (WatchFilter::TaskId(task_id), None) => {
            let task = client.get_task(*task_id).await?;
            client.get_project(task.project_id).await?
        }
        (_, Some(project)) => project,
        _ => return Err(anyhow!("Project could not be resolved")),
    };

    let ws_url = tasks_ws_url(server, project.id)?;
    let (ws_stream, _) = connect_async(ws_url.to_string())
        .await
        .context("Failed to connect to task stream")?;
    let (_, mut read) = ws_stream.split();

    let mut state = serde_json::json!({ "tasks": {} });
    let mut last_render = String::new();

    draw_screen(&render_header(&project.name, "Connecting..."))?;

    loop {
        select! {
            _ = tokio::signal::ctrl_c() => {
                break;
            }
            message = read.next() => {
                let Some(message) = message else { break };
                let message = message?;
                if !message.is_text() {
                    continue;
                }

                let text = message.to_text()?;
                let value: serde_json::Value = serde_json::from_str(text)
                    .context("Failed to parse stream message")?;

                let mut updated = false;

                if value.get("Ready").and_then(|v| v.as_bool()).unwrap_or(false) {
                    updated = true;
                } else if value.get("finished").and_then(|v| v.as_bool()).unwrap_or(false) {
                    break;
                } else if let Some(patch_value) = value.get("JsonPatch") {
                    let patch: Patch = serde_json::from_value(patch_value.clone())
                        .context("Failed to parse JSON patch")?;
                    json_patch::patch(&mut state, &patch)
                        .context("Failed to apply JSON patch")?;
                    updated = true;
                }

                if updated {
                    let tasks = tasks_from_state(&state);
                    let output = render_view(&project.name, &tasks, &filter);
                    if output != last_render {
                        draw_screen(&output)?;
                        last_render = output;
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn select_task_by_filter<'a>(
    tasks: &'a [TaskWithAttemptStatus],
    filter: &WatchFilter,
) -> Option<&'a TaskWithAttemptStatus> {
    match filter {
        WatchFilter::TaskId(task_id) => tasks.iter().find(|t| t.task.id == *task_id),
        WatchFilter::Slug(slug) => tasks
            .iter()
            .find(|t| task_slug(&t.task.title) == slug.as_str()),
        WatchFilter::None => None,
    }
}
