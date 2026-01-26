//! Application state and logic.

use anyhow::Result;
use uuid::Uuid;

use crate::{
    api::VibeKanbanClient,
    types::*,
};

/// View modes for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Projects,
    Tasks,
    Workspaces,
    WorkspaceDetail,
    CreateTask,
    Help,
}

/// Input mode for text fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

/// Task column in the kanban board
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskColumn {
    Todo,
    InProgress,
    InReview,
    Done,
}

impl TaskColumn {
    pub fn status(&self) -> TaskStatus {
        match self {
            TaskColumn::Todo => TaskStatus::Todo,
            TaskColumn::InProgress => TaskStatus::Inprogress,
            TaskColumn::InReview => TaskStatus::Inreview,
            TaskColumn::Done => TaskStatus::Done,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            TaskColumn::Todo => TaskColumn::InProgress,
            TaskColumn::InProgress => TaskColumn::InReview,
            TaskColumn::InReview => TaskColumn::Done,
            TaskColumn::Done => TaskColumn::Done,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            TaskColumn::Todo => TaskColumn::Todo,
            TaskColumn::InProgress => TaskColumn::Todo,
            TaskColumn::InReview => TaskColumn::InProgress,
            TaskColumn::Done => TaskColumn::InReview,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            TaskColumn::Todo => "To Do",
            TaskColumn::InProgress => "In Progress",
            TaskColumn::InReview => "In Review",
            TaskColumn::Done => "Done",
        }
    }
}

/// Main application state
pub struct App {
    /// API client
    pub client: VibeKanbanClient,
    /// Current view
    pub view: View,
    /// Previous view (for back navigation)
    pub previous_view: Option<View>,
    /// Input mode for text editing
    pub input_mode: InputMode,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Status message to display
    pub status_message: Option<String>,
    /// Error message to display
    pub error_message: Option<String>,

    // Projects
    pub projects: Vec<Project>,
    pub selected_project_index: usize,
    pub selected_project: Option<Project>,

    // Tasks
    pub tasks: Vec<TaskWithAttemptStatus>,
    pub selected_column: TaskColumn,
    pub selected_task_indices: [usize; 4], // Index for each column
    pub selected_task: Option<TaskWithAttemptStatus>,

    // Workspaces
    pub workspaces: Vec<Workspace>,
    pub selected_workspace_index: usize,
    pub selected_workspace: Option<Workspace>,
    pub workspace_repos: Vec<RepoWithTargetBranch>,
    pub branch_statuses: Vec<RepoBranchStatus>,

    // Project repositories
    pub project_repos: Vec<Repo>,

    // Sessions
    pub sessions: Vec<Session>,

    // Create task form
    pub new_task_title: String,
    pub new_task_description: String,

    // Follow-up input
    pub follow_up_input: String,
}

impl App {
    /// Create a new application with the given API client.
    pub fn new(client: VibeKanbanClient) -> Self {
        Self {
            client,
            view: View::Projects,
            previous_view: None,
            input_mode: InputMode::Normal,
            should_quit: false,
            status_message: None,
            error_message: None,

            projects: Vec::new(),
            selected_project_index: 0,
            selected_project: None,

            tasks: Vec::new(),
            selected_column: TaskColumn::Todo,
            selected_task_indices: [0; 4],
            selected_task: None,

            workspaces: Vec::new(),
            selected_workspace_index: 0,
            selected_workspace: None,
            workspace_repos: Vec::new(),
            branch_statuses: Vec::new(),

            project_repos: Vec::new(),

            sessions: Vec::new(),

            new_task_title: String::new(),
            new_task_description: String::new(),

            follow_up_input: String::new(),
        }
    }

    /// Set a status message.
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
        self.error_message = None;
    }

    /// Set an error message.
    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error_message = Some(message.into());
        self.status_message = None;
    }

    /// Clear status and error messages.
    pub fn clear_messages(&mut self) {
        self.status_message = None;
        self.error_message = None;
    }

    /// Go back to the previous view.
    pub fn go_back(&mut self) {
        if let Some(prev) = self.previous_view.take() {
            self.view = prev;
        }
    }

    /// Navigate to a new view.
    pub fn navigate_to(&mut self, view: View) {
        self.previous_view = Some(self.view);
        self.view = view;
    }

    // =========================================================================
    // Data Loading
    // =========================================================================

    /// Load projects from the server.
    pub async fn load_projects(&mut self) -> Result<()> {
        self.set_status("Loading projects...");
        self.projects = self.client.list_projects().await?;
        self.selected_project_index = 0.min(self.projects.len().saturating_sub(1));
        self.clear_messages();
        Ok(())
    }

    /// Load tasks for the selected project.
    pub async fn load_tasks(&mut self) -> Result<()> {
        let project_id = self.selected_project.as_ref().map(|p| p.id);
        if let Some(id) = project_id {
            self.set_status("Loading tasks...");
            self.tasks = self.client.list_tasks(id).await?;
            self.clear_messages();
        }
        Ok(())
    }

    /// Load repositories for the selected project.
    pub async fn load_project_repos(&mut self) -> Result<()> {
        let project_id = self.selected_project.as_ref().map(|p| p.id);
        if let Some(id) = project_id {
            self.project_repos = self.client.get_project_repositories(id).await?;
        }
        Ok(())
    }

    /// Load workspaces for the selected task.
    pub async fn load_workspaces(&mut self) -> Result<()> {
        let task_id = self.selected_task.as_ref().map(|t| t.task.id);
        if let Some(id) = task_id {
            self.set_status("Loading workspaces...");
            self.workspaces = self.client.list_workspaces(Some(id)).await?;
            self.selected_workspace_index = 0.min(self.workspaces.len().saturating_sub(1));
            self.clear_messages();
        }
        Ok(())
    }

    /// Load details for the selected workspace.
    pub async fn load_workspace_details(&mut self) -> Result<()> {
        let workspace_id = self.selected_workspace.as_ref().map(|w| w.id);
        if let Some(id) = workspace_id {
            self.set_status("Loading workspace details...");
            self.workspace_repos = self.client.get_workspace_repos(id).await?;
            self.branch_statuses = self.client.get_branch_status(id).await?;
            self.sessions = self.client.list_sessions(id).await?;
            self.clear_messages();
        }
        Ok(())
    }

    // =========================================================================
    // Project Actions
    // =========================================================================

    /// Select a project and navigate to tasks view.
    pub async fn select_project(&mut self) -> Result<()> {
        if let Some(project) = self.projects.get(self.selected_project_index).cloned() {
            self.selected_project = Some(project);
            self.load_tasks().await?;
            self.load_project_repos().await?;
            self.navigate_to(View::Tasks);
        }
        Ok(())
    }

    // =========================================================================
    // Task Actions
    // =========================================================================

    /// Get tasks filtered by status for a column.
    pub fn tasks_for_column(&self, column: TaskColumn) -> Vec<&TaskWithAttemptStatus> {
        self.tasks
            .iter()
            .filter(|t| t.task.status == column.status())
            .collect()
    }

    /// Get the currently selected task in the current column.
    pub fn current_column_selected_task(&self) -> Option<&TaskWithAttemptStatus> {
        let column_index = match self.selected_column {
            TaskColumn::Todo => 0,
            TaskColumn::InProgress => 1,
            TaskColumn::InReview => 2,
            TaskColumn::Done => 3,
        };
        let tasks = self.tasks_for_column(self.selected_column);
        let index = self.selected_task_indices[column_index];
        tasks.get(index).copied()
    }

    /// Select the current task and navigate to workspaces view.
    pub async fn select_task(&mut self) -> Result<()> {
        if let Some(task) = self.current_column_selected_task().cloned() {
            self.selected_task = Some(task);
            self.load_workspaces().await?;
            self.navigate_to(View::Workspaces);
        }
        Ok(())
    }

    /// Create a new task.
    pub async fn create_task(&mut self) -> Result<()> {
        if self.new_task_title.trim().is_empty() {
            self.set_error("Task title cannot be empty");
            return Ok(());
        }

        let project_id = self.selected_project.as_ref().map(|p| p.id);
        if let Some(id) = project_id {
            self.set_status("Creating task...");
            let payload = CreateTask {
                project_id: id,
                title: self.new_task_title.clone(),
                description: if self.new_task_description.is_empty() {
                    None
                } else {
                    Some(self.new_task_description.clone())
                },
                status: None,
                parent_workspace_id: None,
                image_ids: None,
                is_epic: None,
                complexity: None,
                metadata: None,
            };

            self.client.create_task(&payload).await?;
            self.new_task_title.clear();
            self.new_task_description.clear();
            self.load_tasks().await?;
            self.set_status("Task created successfully");
            self.go_back();
        }
        Ok(())
    }

    /// Update a task's status.
    pub async fn update_task_status(&mut self, task_id: Uuid, status: TaskStatus) -> Result<()> {
        self.set_status("Updating task...");
        let payload = UpdateTask {
            title: None,
            description: None,
            status: Some(status),
            parent_workspace_id: None,
            image_ids: None,
            is_epic: None,
            complexity: None,
            metadata: None,
        };
        self.client.update_task(task_id, &payload).await?;
        self.load_tasks().await?;
        self.set_status("Task updated");
        Ok(())
    }

    /// Delete the selected task.
    pub async fn delete_selected_task(&mut self) -> Result<()> {
        let task_id = self.current_column_selected_task().map(|t| t.task.id);
        if let Some(id) = task_id {
            self.set_status("Deleting task...");
            self.client.delete_task(id).await?;
            self.load_tasks().await?;
            self.set_status("Task deleted");
        }
        Ok(())
    }

    // =========================================================================
    // Workspace Actions
    // =========================================================================

    /// Select a workspace and show details.
    pub async fn select_workspace(&mut self) -> Result<()> {
        if let Some(workspace) = self.workspaces.get(self.selected_workspace_index).cloned() {
            self.selected_workspace = Some(workspace);
            self.load_workspace_details().await?;
            self.navigate_to(View::WorkspaceDetail);
        }
        Ok(())
    }

    /// Stop the selected workspace execution.
    pub async fn stop_workspace(&mut self) -> Result<()> {
        let workspace_id = self.selected_workspace.as_ref().map(|w| w.id);
        if let Some(id) = workspace_id {
            self.set_status("Stopping workspace...");
            self.client.stop_workspace(id).await?;
            self.load_workspace_details().await?;
            self.set_status("Workspace stopped");
        }
        Ok(())
    }

    // =========================================================================
    // Git Actions
    // =========================================================================

    /// Merge the selected workspace.
    pub async fn merge_workspace(&mut self) -> Result<()> {
        let workspace_id = self.selected_workspace.as_ref().map(|w| w.id);
        let repo_id = self.branch_statuses.first().map(|s| s.repo_id);
        if let (Some(ws_id), Some(r_id)) = (workspace_id, repo_id) {
            self.set_status("Merging...");
            self.client.merge_workspace(ws_id, r_id).await?;
            self.load_workspace_details().await?;
            self.set_status("Merged successfully");
        }
        Ok(())
    }

    /// Push the selected workspace branch.
    pub async fn push_workspace(&mut self) -> Result<()> {
        let workspace_id = self.selected_workspace.as_ref().map(|w| w.id);
        let repo_id = self.branch_statuses.first().map(|s| s.repo_id);
        if let (Some(ws_id), Some(r_id)) = (workspace_id, repo_id) {
            self.set_status("Pushing...");
            self.client.push_workspace(ws_id, r_id).await?;
            self.load_workspace_details().await?;
            self.set_status("Pushed successfully");
        }
        Ok(())
    }

    /// Rebase the selected workspace branch.
    pub async fn rebase_workspace(&mut self) -> Result<()> {
        let workspace_id = self.selected_workspace.as_ref().map(|w| w.id);
        let repo_id = self.branch_statuses.first().map(|s| s.repo_id);
        if let (Some(ws_id), Some(r_id)) = (workspace_id, repo_id) {
            self.set_status("Rebasing...");
            self.client.rebase_workspace(ws_id, r_id, None, None).await?;
            self.load_workspace_details().await?;
            self.set_status("Rebased successfully");
        }
        Ok(())
    }

    // =========================================================================
    // Navigation Helpers
    // =========================================================================

    /// Move selection up in the current list.
    pub fn move_up(&mut self) {
        match self.view {
            View::Projects => {
                if self.selected_project_index > 0 {
                    self.selected_project_index -= 1;
                }
            }
            View::Tasks => {
                let column_index = match self.selected_column {
                    TaskColumn::Todo => 0,
                    TaskColumn::InProgress => 1,
                    TaskColumn::InReview => 2,
                    TaskColumn::Done => 3,
                };
                if self.selected_task_indices[column_index] > 0 {
                    self.selected_task_indices[column_index] -= 1;
                }
            }
            View::Workspaces => {
                if self.selected_workspace_index > 0 {
                    self.selected_workspace_index -= 1;
                }
            }
            _ => {}
        }
    }

    /// Move selection down in the current list.
    pub fn move_down(&mut self) {
        match self.view {
            View::Projects => {
                if self.selected_project_index < self.projects.len().saturating_sub(1) {
                    self.selected_project_index += 1;
                }
            }
            View::Tasks => {
                let column_index = match self.selected_column {
                    TaskColumn::Todo => 0,
                    TaskColumn::InProgress => 1,
                    TaskColumn::InReview => 2,
                    TaskColumn::Done => 3,
                };
                let tasks = self.tasks_for_column(self.selected_column);
                if self.selected_task_indices[column_index] < tasks.len().saturating_sub(1) {
                    self.selected_task_indices[column_index] += 1;
                }
            }
            View::Workspaces => {
                if self.selected_workspace_index < self.workspaces.len().saturating_sub(1) {
                    self.selected_workspace_index += 1;
                }
            }
            _ => {}
        }
    }

    /// Move selection left (columns in tasks view).
    pub fn move_left(&mut self) {
        if self.view == View::Tasks {
            self.selected_column = self.selected_column.prev();
        }
    }

    /// Move selection right (columns in tasks view).
    pub fn move_right(&mut self) {
        if self.view == View::Tasks {
            self.selected_column = self.selected_column.next();
        }
    }
}
