//! HTTP client for the Vibe Kanban API.

use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use uuid::Uuid;

use crate::types::*;

/// Client for interacting with the Vibe Kanban server API.
#[derive(Clone)]
pub struct VibeKanbanClient {
    client: Client,
    base_url: String,
}

impl VibeKanbanClient {
    /// Create a new API client.
    pub fn new(base_url: &str) -> Result<Self> {
        let client = Client::builder()
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// Build the full URL for an API endpoint.
    fn url(&self, path: &str) -> String {
        format!("{}/api{}", self.base_url, path)
    }

    /// Extract data from an API response or return an error.
    fn extract_data<T>(response: ApiResponse<T>) -> Result<T> {
        if response.success {
            response.data.ok_or_else(|| anyhow!("Response success but no data"))
        } else {
            Err(anyhow!(
                "API error: {}",
                response.message.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    // =========================================================================
    // Projects
    // =========================================================================

    /// List all projects.
    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        let response = self
            .client
            .get(self.url("/projects"))
            .send()
            .await
            .context("Failed to fetch projects")?
            .json::<ApiResponse<Vec<Project>>>()
            .await
            .context("Failed to parse projects response")?;

        Self::extract_data(response)
    }

    /// Get a project by ID.
    pub async fn get_project(&self, project_id: Uuid) -> Result<Project> {
        let response = self
            .client
            .get(self.url(&format!("/projects/{}", project_id)))
            .send()
            .await
            .context("Failed to fetch project")?
            .json::<ApiResponse<Project>>()
            .await
            .context("Failed to parse project response")?;

        Self::extract_data(response)
    }

    /// Create a new project.
    pub async fn create_project(&self, payload: &CreateProject) -> Result<Project> {
        let response = self
            .client
            .post(self.url("/projects"))
            .json(payload)
            .send()
            .await
            .context("Failed to create project")?
            .json::<ApiResponse<Project>>()
            .await
            .context("Failed to parse create project response")?;

        Self::extract_data(response)
    }

    /// Get repositories for a project.
    pub async fn get_project_repositories(&self, project_id: Uuid) -> Result<Vec<Repo>> {
        let response = self
            .client
            .get(self.url(&format!("/projects/{}/repositories", project_id)))
            .send()
            .await
            .context("Failed to fetch repositories")?
            .json::<ApiResponse<Vec<Repo>>>()
            .await
            .context("Failed to parse repositories response")?;

        Self::extract_data(response)
    }

    // =========================================================================
    // Tasks
    // =========================================================================

    /// List tasks for a project.
    pub async fn list_tasks(&self, project_id: Uuid) -> Result<Vec<TaskWithAttemptStatus>> {
        let response = self
            .client
            .get(self.url("/tasks"))
            .query(&[("project_id", project_id.to_string())])
            .send()
            .await
            .context("Failed to fetch tasks")?
            .json::<ApiResponse<Vec<TaskWithAttemptStatus>>>()
            .await
            .context("Failed to parse tasks response")?;

        Self::extract_data(response)
    }

    /// Get a task by ID.
    pub async fn get_task(&self, task_id: Uuid) -> Result<Task> {
        let response = self
            .client
            .get(self.url(&format!("/tasks/{}", task_id)))
            .send()
            .await
            .context("Failed to fetch task")?
            .json::<ApiResponse<Task>>()
            .await
            .context("Failed to parse task response")?;

        Self::extract_data(response)
    }

    /// Create a new task.
    pub async fn create_task(&self, payload: &CreateTask) -> Result<Task> {
        let response = self
            .client
            .post(self.url("/tasks"))
            .json(payload)
            .send()
            .await
            .context("Failed to create task")?
            .json::<ApiResponse<Task>>()
            .await
            .context("Failed to parse create task response")?;

        Self::extract_data(response)
    }

    /// Update a task.
    pub async fn update_task(&self, task_id: Uuid, payload: &UpdateTask) -> Result<Task> {
        let response = self
            .client
            .put(self.url(&format!("/tasks/{}", task_id)))
            .json(payload)
            .send()
            .await
            .context("Failed to update task")?
            .json::<ApiResponse<Task>>()
            .await
            .context("Failed to parse update task response")?;

        Self::extract_data(response)
    }

    /// Delete a task.
    pub async fn delete_task(&self, task_id: Uuid) -> Result<()> {
        let response = self
            .client
            .delete(self.url(&format!("/tasks/{}", task_id)))
            .send()
            .await
            .context("Failed to delete task")?
            .json::<ApiResponse<()>>()
            .await
            .context("Failed to parse delete task response")?;

        Self::extract_data(response)
    }

    /// Create a task and start it immediately.
    pub async fn create_and_start_task(
        &self,
        payload: &CreateAndStartTaskRequest,
    ) -> Result<TaskWithAttemptStatus> {
        let response = self
            .client
            .post(self.url("/tasks/create-and-start"))
            .json(payload)
            .send()
            .await
            .context("Failed to create and start task")?
            .json::<ApiResponse<TaskWithAttemptStatus>>()
            .await
            .context("Failed to parse create and start task response")?;

        Self::extract_data(response)
    }

    // =========================================================================
    // Workspaces (Task Attempts)
    // =========================================================================

    /// List workspaces (task attempts).
    pub async fn list_workspaces(&self, task_id: Option<Uuid>) -> Result<Vec<Workspace>> {
        let mut request = self.client.get(self.url("/task-attempts"));

        if let Some(task_id) = task_id {
            request = request.query(&[("task_id", task_id.to_string())]);
        }

        let response = request
            .send()
            .await
            .context("Failed to fetch workspaces")?
            .json::<ApiResponse<Vec<Workspace>>>()
            .await
            .context("Failed to parse workspaces response")?;

        Self::extract_data(response)
    }

    /// Get a workspace by ID.
    pub async fn get_workspace(&self, workspace_id: Uuid) -> Result<Workspace> {
        let response = self
            .client
            .get(self.url(&format!("/task-attempts/{}", workspace_id)))
            .send()
            .await
            .context("Failed to fetch workspace")?
            .json::<ApiResponse<Workspace>>()
            .await
            .context("Failed to parse workspace response")?;

        Self::extract_data(response)
    }

    /// Create a task attempt (workspace).
    pub async fn create_task_attempt(&self, payload: &CreateTaskAttemptBody) -> Result<Workspace> {
        let response = self
            .client
            .post(self.url("/task-attempts"))
            .json(payload)
            .send()
            .await
            .context("Failed to create task attempt")?
            .json::<ApiResponse<Workspace>>()
            .await
            .context("Failed to parse create task attempt response")?;

        Self::extract_data(response)
    }

    /// Get branch status for a workspace.
    pub async fn get_branch_status(&self, workspace_id: Uuid) -> Result<Vec<RepoBranchStatus>> {
        let response = self
            .client
            .get(self.url(&format!("/task-attempts/{}/branch-status", workspace_id)))
            .send()
            .await
            .context("Failed to fetch branch status")?
            .json::<ApiResponse<Vec<RepoBranchStatus>>>()
            .await
            .context("Failed to parse branch status response")?;

        Self::extract_data(response)
    }

    /// Get repositories for a workspace.
    pub async fn get_workspace_repos(&self, workspace_id: Uuid) -> Result<Vec<RepoWithTargetBranch>> {
        let response = self
            .client
            .get(self.url(&format!("/task-attempts/{}/repos", workspace_id)))
            .send()
            .await
            .context("Failed to fetch workspace repos")?
            .json::<ApiResponse<Vec<RepoWithTargetBranch>>>()
            .await
            .context("Failed to parse workspace repos response")?;

        Self::extract_data(response)
    }

    /// Stop a workspace execution.
    pub async fn stop_workspace(&self, workspace_id: Uuid) -> Result<()> {
        let response = self
            .client
            .post(self.url(&format!("/task-attempts/{}/stop", workspace_id)))
            .send()
            .await
            .context("Failed to stop workspace")?
            .json::<ApiResponse<()>>()
            .await
            .context("Failed to parse stop workspace response")?;

        Self::extract_data(response)
    }

    // =========================================================================
    // Git Operations
    // =========================================================================

    /// Merge changes for a workspace.
    pub async fn merge_workspace(&self, workspace_id: Uuid, repo_id: Uuid) -> Result<()> {
        let payload = MergeTaskAttemptRequest { repo_id };
        let response = self
            .client
            .post(self.url(&format!("/task-attempts/{}/merge", workspace_id)))
            .json(&payload)
            .send()
            .await
            .context("Failed to merge workspace")?
            .json::<ApiResponse<()>>()
            .await
            .context("Failed to parse merge response")?;

        Self::extract_data(response)
    }

    /// Push workspace branch.
    pub async fn push_workspace(&self, workspace_id: Uuid, repo_id: Uuid) -> Result<()> {
        let payload = PushTaskAttemptRequest { repo_id };
        let response = self
            .client
            .post(self.url(&format!("/task-attempts/{}/push", workspace_id)))
            .json(&payload)
            .send()
            .await
            .context("Failed to push workspace")?
            .json::<ApiResponse<()>>()
            .await
            .context("Failed to parse push response")?;

        Self::extract_data(response)
    }

    /// Rebase workspace branch.
    pub async fn rebase_workspace(
        &self,
        workspace_id: Uuid,
        repo_id: Uuid,
        old_base: Option<String>,
        new_base: Option<String>,
    ) -> Result<()> {
        let payload = RebaseTaskAttemptRequest {
            repo_id,
            old_base_branch: old_base,
            new_base_branch: new_base,
        };
        let response = self
            .client
            .post(self.url(&format!("/task-attempts/{}/rebase", workspace_id)))
            .json(&payload)
            .send()
            .await
            .context("Failed to rebase workspace")?
            .json::<ApiResponse<()>>()
            .await
            .context("Failed to parse rebase response")?;

        Self::extract_data(response)
    }

    // =========================================================================
    // Sessions
    // =========================================================================

    /// List sessions for a workspace.
    pub async fn list_sessions(&self, workspace_id: Uuid) -> Result<Vec<Session>> {
        let response = self
            .client
            .get(self.url("/sessions"))
            .query(&[("workspace_id", workspace_id.to_string())])
            .send()
            .await
            .context("Failed to fetch sessions")?
            .json::<ApiResponse<Vec<Session>>>()
            .await
            .context("Failed to parse sessions response")?;

        Self::extract_data(response)
    }

    /// Send a follow-up message to a session.
    pub async fn send_follow_up(
        &self,
        session_id: Uuid,
        payload: &CreateFollowUpAttempt,
    ) -> Result<ExecutionProcess> {
        let response = self
            .client
            .post(self.url(&format!("/sessions/{}/follow-up", session_id)))
            .json(payload)
            .send()
            .await
            .context("Failed to send follow-up")?
            .json::<ApiResponse<ExecutionProcess>>()
            .await
            .context("Failed to parse follow-up response")?;

        Self::extract_data(response)
    }

    // =========================================================================
    // Repositories
    // =========================================================================

    /// List all repositories.
    pub async fn list_repos(&self) -> Result<Vec<Repo>> {
        let response = self
            .client
            .get(self.url("/repos"))
            .send()
            .await
            .context("Failed to fetch repos")?
            .json::<ApiResponse<Vec<Repo>>>()
            .await
            .context("Failed to parse repos response")?;

        Self::extract_data(response)
    }

    /// Get branches for a repository.
    pub async fn get_repo_branches(&self, repo_id: Uuid) -> Result<Vec<GitBranch>> {
        let response = self
            .client
            .get(self.url(&format!("/repos/{}/branches", repo_id)))
            .send()
            .await
            .context("Failed to fetch branches")?
            .json::<ApiResponse<Vec<GitBranch>>>()
            .await
            .context("Failed to parse branches response")?;

        Self::extract_data(response)
    }

    // =========================================================================
    // Health Check
    // =========================================================================

    /// Check if the server is healthy.
    pub async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(self.url("/health"))
            .send()
            .await
            .context("Failed to reach server")?;

        Ok(response.status().is_success())
    }
}
