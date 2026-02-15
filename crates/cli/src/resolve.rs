use anyhow::{Context, Result, anyhow};
use url::Url;
use uuid::Uuid;

use crate::VibeKanbanClient;
use vibe_kanban_cli::types::{GitBranch, Project, Repo, WorkspaceRepoInput};

pub fn parse_uuid(input: &str) -> Result<Uuid> {
    Uuid::parse_str(input).context("Invalid UUID")
}

pub async fn resolve_project(client: &VibeKanbanClient, project_ref: &str) -> Result<Project> {
    let projects = client.list_projects().await?;
    if let Ok(id) = Uuid::parse_str(project_ref) {
        if let Some(project) = projects.into_iter().find(|p| p.id == id) {
            return Ok(project);
        }
        return Err(anyhow!("Project ID not found: {}", project_ref));
    }

    let lower = project_ref.to_lowercase();
    if let Some(project) = projects.into_iter().find(|p| p.name.to_lowercase() == lower) {
        return Ok(project);
    }

    Err(anyhow!(
        "Project '{}' not found. Use a project ID or exact project name.",
        project_ref
    ))
}

pub async fn resolve_repo_inputs(
    client: &VibeKanbanClient,
    project_id: Uuid,
    repo_refs: Vec<String>,
    global_branch: Option<&str>,
) -> Result<Vec<WorkspaceRepoInput>> {
    let repos = client.get_project_repositories(project_id).await?;

    let repo_refs = if repo_refs.is_empty() {
        if repos.len() == 1 {
            vec![repos[0].id.to_string()]
        } else {
            let repo_list = repos
                .iter()
                .map(|r| format!("{} ({})", r.display_name, r.id))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(anyhow!(
                "Project has multiple repos. Specify --repo. Available: {}",
                repo_list
            ));
        }
    } else {
        repo_refs
    };

    let mut inputs = Vec::new();
    for repo_ref in repo_refs {
        let (repo_key, branch_override) = split_repo_branch(&repo_ref);
        let repo = find_repo(&repos, &repo_key).ok_or_else(|| {
            anyhow!(
                "Repo '{}' not found for project. Use repo name, display name, or ID.",
                repo_key
            )
        })?;

        let branch = if let Some(branch) = branch_override.or(global_branch.map(|b| b.to_string())) {
            branch
        } else {
            pick_default_branch(client, repo.id).await?
        };

        inputs.push(WorkspaceRepoInput {
            repo_id: repo.id,
            target_branch: branch,
        });
    }

    Ok(inputs)
}

pub fn split_repo_branch(input: &str) -> (String, Option<String>) {
    if let Some(idx) = input.rfind('@') {
        let (left, right) = input.split_at(idx);
        let branch = right.trim_start_matches('@');
        if !branch.is_empty() {
            return (left.to_string(), Some(branch.to_string()));
        }
    }
    (input.to_string(), None)
}

pub fn find_repo<'a>(repos: &'a [Repo], key: &str) -> Option<&'a Repo> {
    if let Ok(id) = Uuid::parse_str(key) {
        return repos.iter().find(|r| r.id == id);
    }

    let lower = key.to_lowercase();
    repos.iter().find(|r| {
        r.name.to_lowercase() == lower
            || r.display_name.to_lowercase() == lower
            || r.path.to_lowercase().ends_with(&lower)
    })
}

pub async fn pick_default_branch(client: &VibeKanbanClient, repo_id: Uuid) -> Result<String> {
    let branches = client.get_repo_branches(repo_id).await?;
    Ok(default_branch_from_list(&branches).unwrap_or_else(|| "main".to_string()))
}

pub fn default_branch_from_list(branches: &[GitBranch]) -> Option<String> {
    let preferred = ["main", "master"];
    for name in preferred {
        if let Some(branch) = branches.iter().find(|b| b.name == name) {
            return Some(branch.name.clone());
        }
    }
    branches.first().map(|b| b.name.clone())
}

pub fn tasks_ws_url(base_url: &str, project_id: Uuid) -> Result<Url> {
    let mut url = Url::parse(base_url).context("Invalid server URL")?;
    let scheme = match url.scheme() {
        "https" => "wss",
        "http" => "ws",
        other => return Err(anyhow!("Unsupported URL scheme: {}", other)),
    };
    url.set_scheme(scheme).ok();
    url.set_path("/api/tasks/stream/ws");
    url.set_query(Some(&format!("project_id={}", project_id)));
    Ok(url)
}
