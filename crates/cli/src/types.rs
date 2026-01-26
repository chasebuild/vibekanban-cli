//! Type definitions that mirror the server API types.
//!
//! These types are used for API communication with the Vibe Kanban server.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generic API response wrapper
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error_data: Option<serde_json::Value>,
    pub message: Option<String>,
}

/// Project model
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub default_agent_working_dir: Option<String>,
    pub remote_project_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create project request
#[derive(Debug, Serialize)]
pub struct CreateProject {
    pub name: String,
    pub repositories: Vec<CreateProjectRepo>,
}

/// Create project repository
#[derive(Debug, Serialize)]
pub struct CreateProjectRepo {
    pub display_name: String,
    pub git_repo_path: String,
}

/// Task status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Todo,
    Inprogress,
    Inreview,
    Done,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::Inprogress => "inprogress",
            TaskStatus::Inreview => "inreview",
            TaskStatus::Done => "done",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "To Do",
            TaskStatus::Inprogress => "In Progress",
            TaskStatus::Inreview => "In Review",
            TaskStatus::Done => "Done",
            TaskStatus::Cancelled => "Cancelled",
        }
    }
}

/// Task complexity enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskComplexity {
    Trivial,
    Simple,
    Moderate,
    Complex,
    Epic,
}

/// Task model
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Task {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub parent_workspace_id: Option<Uuid>,
    pub is_epic: bool,
    pub complexity: Option<TaskComplexity>,
    pub metadata: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Task with attempt status info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskWithAttemptStatus {
    pub has_in_progress_attempt: bool,
    pub last_attempt_failed: bool,
    pub executor: String,
    #[serde(flatten)]
    pub task: Task,
}

/// Create task request
#[derive(Debug, Serialize)]
pub struct CreateTask {
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub parent_workspace_id: Option<Uuid>,
    pub image_ids: Option<Vec<Uuid>>,
    pub is_epic: Option<bool>,
    pub complexity: Option<TaskComplexity>,
    pub metadata: Option<String>,
}

/// Update task request
#[derive(Debug, Serialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub parent_workspace_id: Option<Uuid>,
    pub image_ids: Option<Vec<Uuid>>,
    pub is_epic: Option<bool>,
    pub complexity: Option<TaskComplexity>,
    pub metadata: Option<String>,
}

/// Repository model
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repo {
    pub id: Uuid,
    pub path: String,
    pub name: String,
    pub display_name: String,
    pub setup_script: Option<String>,
    pub cleanup_script: Option<String>,
    pub copy_files: Option<String>,
    pub parallel_setup_script: bool,
    pub dev_server_script: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Workspace (task attempt) model
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Workspace {
    pub id: Uuid,
    pub task_id: Uuid,
    pub container_ref: Option<String>,
    pub branch: String,
    pub agent_working_dir: Option<String>,
    pub setup_completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub pinned: bool,
    pub name: Option<String>,
}

/// Session model
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Session {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub executor: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Execution process status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionProcessStatus {
    Running,
    Completed,
    Failed,
    Killed,
}

/// Execution process model
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionProcess {
    pub id: Uuid,
    pub session_id: Uuid,
    pub run_reason: String,
    pub status: ExecutionProcessStatus,
    pub exit_code: Option<i64>,
    pub dropped: bool,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Base coding agent types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BaseCodingAgent {
    ClaudeCode,
    Amp,
    Gemini,
    Codex,
    Opencode,
    CursorAgent,
    QwenCode,
    Copilot,
    Droid,
}

impl BaseCodingAgent {
    pub fn as_str(&self) -> &'static str {
        match self {
            BaseCodingAgent::ClaudeCode => "CLAUDE_CODE",
            BaseCodingAgent::Amp => "AMP",
            BaseCodingAgent::Gemini => "GEMINI",
            BaseCodingAgent::Codex => "CODEX",
            BaseCodingAgent::Opencode => "OPENCODE",
            BaseCodingAgent::CursorAgent => "CURSOR_AGENT",
            BaseCodingAgent::QwenCode => "QWEN_CODE",
            BaseCodingAgent::Copilot => "COPILOT",
            BaseCodingAgent::Droid => "DROID",
        }
    }
}

/// Executor profile ID
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutorProfileId {
    pub executor: BaseCodingAgent,
    pub variant: Option<String>,
}

/// Create task attempt body
#[derive(Debug, Serialize)]
pub struct CreateTaskAttemptBody {
    pub task_id: Uuid,
    pub executor_profile_id: ExecutorProfileId,
    pub repos: Vec<WorkspaceRepoInput>,
}

/// Workspace repository input
#[derive(Debug, Serialize)]
pub struct WorkspaceRepoInput {
    pub repo_id: Uuid,
    pub target_branch: String,
}

/// Create and start task request
#[derive(Debug, Serialize)]
pub struct CreateAndStartTaskRequest {
    pub task: CreateTask,
    pub executor_profile_id: ExecutorProfileId,
    pub repos: Vec<WorkspaceRepoInput>,
}

/// Follow-up request
#[derive(Debug, Serialize)]
pub struct CreateFollowUpAttempt {
    pub prompt: String,
    pub executor_profile_id: ExecutorProfileId,
    pub retry_process_id: Option<Uuid>,
    pub force_when_dirty: Option<bool>,
    pub perform_git_reset: Option<bool>,
}

/// Merge task attempt request
#[derive(Debug, Serialize)]
pub struct MergeTaskAttemptRequest {
    pub repo_id: Uuid,
}

/// Push task attempt request
#[derive(Debug, Serialize)]
pub struct PushTaskAttemptRequest {
    pub repo_id: Uuid,
}

/// Rebase task attempt request
#[derive(Debug, Serialize)]
pub struct RebaseTaskAttemptRequest {
    pub repo_id: Uuid,
    pub old_base_branch: Option<String>,
    pub new_base_branch: Option<String>,
}

/// Git branch info
#[derive(Debug, Clone, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub last_commit_date: DateTime<Utc>,
}

/// Branch status
#[derive(Debug, Clone, Deserialize)]
pub struct BranchStatus {
    pub commits_behind: Option<usize>,
    pub commits_ahead: Option<usize>,
    pub has_uncommitted_changes: Option<bool>,
    pub head_oid: Option<String>,
    pub uncommitted_count: Option<usize>,
    pub untracked_count: Option<usize>,
    pub target_branch_name: String,
    pub remote_commits_behind: Option<usize>,
    pub remote_commits_ahead: Option<usize>,
    pub is_rebase_in_progress: bool,
    pub conflict_op: Option<String>,
    pub conflicted_files: Vec<String>,
}

/// Repository branch status
#[derive(Debug, Clone, Deserialize)]
pub struct RepoBranchStatus {
    pub repo_id: Uuid,
    pub repo_name: String,
    #[serde(flatten)]
    pub status: BranchStatus,
}

/// Diff change kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiffChangeKind {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    PermissionChange,
}

/// Diff entry
#[derive(Debug, Clone, Deserialize)]
pub struct Diff {
    pub change: DiffChangeKind,
    #[serde(rename = "oldPath")]
    pub old_path: Option<String>,
    #[serde(rename = "newPath")]
    pub new_path: Option<String>,
    #[serde(rename = "oldContent")]
    pub old_content: Option<String>,
    #[serde(rename = "newContent")]
    pub new_content: Option<String>,
    #[serde(rename = "contentOmitted")]
    pub content_omitted: bool,
    pub additions: Option<i32>,
    pub deletions: Option<i32>,
    #[serde(rename = "repoId")]
    pub repo_id: Option<Uuid>,
}

/// Diff stats
#[derive(Debug, Clone, Deserialize)]
pub struct DiffStats {
    pub files_changed: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
}

/// Repo with target branch
#[derive(Debug, Clone, Deserialize)]
pub struct RepoWithTargetBranch {
    pub target_branch: String,
    #[serde(flatten)]
    pub repo: Repo,
}

/// Workspace summary
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceSummary {
    pub workspace_id: Uuid,
    pub latest_session_id: Option<Uuid>,
    pub has_pending_approval: bool,
    pub files_changed: Option<i32>,
    pub lines_added: Option<i32>,
    pub lines_removed: Option<i32>,
    pub latest_process_completed_at: Option<String>,
    pub latest_process_status: Option<ExecutionProcessStatus>,
    pub has_running_dev_server: bool,
    pub has_unseen_turns: bool,
    pub pr_status: Option<String>,
}
