//! Team Manager Service
//!
//! Orchestrates parallel execution of team tasks, managing agent assignment,
//! workspace creation, and execution coordination.

use db::models::{
    agent_profile::AgentProfile,
    team_execution::{TeamExecution, TeamExecutionStatus},
    team_task::{TeamProgress, TeamTask, TeamTaskStatus},
    task::{Task, TaskStatus},
    workspace::{CreateWorkspace, Workspace},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TeamError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Team execution not found: {0}")]
    ExecutionNotFound(Uuid),
    #[error("No available workers")]
    NoAvailableWorkers,
    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

/// Event types for team execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TeamEvent {
    TaskStarted {
        team_task_id: Uuid,
        agent_id: Uuid,
    },
    TaskCompleted {
        team_task_id: Uuid,
    },
    TaskFailed {
        team_task_id: Uuid,
        error: String,
    },
    ExecutionProgress {
        progress: TeamProgress,
    },
    ExecutionCompleted {
        team_execution_id: Uuid,
    },
    ExecutionFailed {
        team_execution_id: Uuid,
        error: String,
    },
}

/// Configuration for the team manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamManagerConfig {
    /// Maximum parallel tasks
    pub max_parallel_tasks: i32,
    /// Task timeout in seconds
    pub task_timeout_seconds: i64,
    /// Retry delay in seconds
    pub retry_delay_seconds: i64,
    /// Branch prefix for team tasks
    pub branch_prefix: String,
}

impl Default for TeamManagerConfig {
    fn default() -> Self {
        Self {
            max_parallel_tasks: 5,
            task_timeout_seconds: 3600, // 1 hour
            retry_delay_seconds: 30,
            branch_prefix: "team".to_string(),
        }
    }
}

/// Manages team execution orchestration
pub struct TeamManager {
    pool: SqlitePool,
    config: TeamManagerConfig,
    event_sender: Option<mpsc::Sender<TeamEvent>>,
}

impl TeamManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            config: TeamManagerConfig::default(),
            event_sender: None,
        }
    }

    pub fn with_config(pool: SqlitePool, config: TeamManagerConfig) -> Self {
        Self {
            pool,
            config,
            event_sender: None,
        }
    }

    pub fn with_event_channel(mut self, sender: mpsc::Sender<TeamEvent>) -> Self {
        self.event_sender = Some(sender);
        self
    }

    async fn emit_event(&self, event: TeamEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event).await;
        }
    }

    /// Get the current status of a team execution
    pub async fn get_status(
        &self,
        team_execution_id: Uuid,
    ) -> Result<TeamExecutionStatus, TeamError> {
        let execution = TeamExecution::find_by_id(&self.pool, team_execution_id)
            .await?
            .ok_or(TeamError::ExecutionNotFound(team_execution_id))?;

        Ok(execution.status)
    }

    /// Get progress of a team execution
    pub async fn get_progress(
        &self,
        team_execution_id: Uuid,
    ) -> Result<TeamProgress, TeamError> {
        let progress = TeamTask::get_progress(&self.pool, team_execution_id).await?;
        Ok(progress)
    }

    /// Start executing ready tasks in a team
    pub async fn execute_ready_tasks(
        &self,
        team_execution_id: Uuid,
    ) -> Result<Vec<Uuid>, TeamError> {
        let execution = TeamExecution::find_by_id(&self.pool, team_execution_id)
            .await?
            .ok_or(TeamError::ExecutionNotFound(team_execution_id))?;

        if execution.status != TeamExecutionStatus::Executing {
            return Err(TeamError::InvalidStateTransition(
                "Execution is not in executing state".into(),
            ));
        }

        // Get currently running tasks
        let running_tasks = TeamTask::find_running_tasks(&self.pool, team_execution_id).await?;
        let available_slots =
            (execution.max_parallel_workers as usize).saturating_sub(running_tasks.len());

        if available_slots == 0 {
            return Ok(vec![]);
        }

        // Get ready tasks
        let ready_tasks = TeamTask::find_ready_tasks(&self.pool, team_execution_id).await?;
        let tasks_to_start: Vec<_> = ready_tasks.into_iter().take(available_slots).collect();

        let mut started_task_ids = Vec::new();

        for task in tasks_to_start {
            match self.start_task(&task).await {
                Ok(_) => {
                    started_task_ids.push(task.id);
                }
                Err(e) => {
                    tracing::error!("Failed to start task {}: {:?}", task.id, e);
                }
            }
        }

        // Check if all tasks are completed
        if TeamTask::all_completed(&self.pool, team_execution_id).await? {
            TeamExecution::update_status(
                &self.pool,
                team_execution_id,
                TeamExecutionStatus::Completed,
            )
            .await?;

            self.emit_event(TeamEvent::ExecutionCompleted { team_execution_id })
                .await;
        }

        Ok(started_task_ids)
    }

    /// Start execution of a single task
    async fn start_task(&self, team_task: &TeamTask) -> Result<(), TeamError> {
        // Find an available agent with required skills
        let agent = self.find_best_agent(team_task).await?;

        // Assign the agent
        TeamTask::assign_agent(&self.pool, team_task.id, agent.id).await?;

        // Create workspace and branch for this task
        let _execution = TeamExecution::find_by_id(&self.pool, team_task.team_execution_id)
            .await?
            .ok_or(TeamError::ExecutionNotFound(team_task.team_execution_id))?;

        let branch_name = format!(
            "{}/task-{}",
            self.config.branch_prefix,
            &team_task.id.to_string()[..8]
        );

        // Get the task for this team task
        let task = Task::find_by_id(&self.pool, team_task.task_id)
            .await?
            .ok_or(TeamError::TaskNotFound(team_task.task_id))?;

        // Create workspace
        let workspace = Workspace::create(
            &self.pool,
            &CreateWorkspace {
                branch: branch_name.clone(),
                agent_working_dir: None,
            },
            Uuid::new_v4(),
            task.id,
        )
        .await
        .map_err(|e| TeamError::ExecutionFailed(e.to_string()))?;

        // Update team task with workspace info
        TeamTask::set_workspace(&self.pool, team_task.id, workspace.id, &branch_name).await?;
        TeamTask::start(&self.pool, team_task.id).await?;

        // Update task status
        Task::update_status(&self.pool, task.id, TaskStatus::InProgress).await?;

        self.emit_event(TeamEvent::TaskStarted {
            team_task_id: team_task.id,
            agent_id: agent.id,
        })
        .await;

        Ok(())
    }

    /// Find the best available agent for a task based on required skills
    async fn find_best_agent(&self, team_task: &TeamTask) -> Result<AgentProfile, TeamError> {
        let required_skills = team_task.get_required_skills();

        // Get all workers
        let workers = AgentProfile::find_workers(&self.pool).await?;

        if workers.is_empty() {
            return Err(TeamError::NoAvailableWorkers);
        }

        // If no specific skills required, return any worker
        if required_skills.is_empty() {
            return Ok(workers.into_iter().next().unwrap());
        }

        // Score workers based on skill match
        let mut best_worker = None;
        let mut best_score = 0;

        for worker in workers {
            let worker_skills = AgentProfile::get_skills(&self.pool, worker.id).await?;
            let worker_skill_names: Vec<_> = worker_skills.iter().map(|s| s.name.clone()).collect();

            let score: i32 = required_skills
                .iter()
                .filter(|skill| worker_skill_names.contains(skill))
                .count() as i32;

            if score > best_score {
                best_score = score;
                best_worker = Some(worker);
            }
        }

        best_worker.ok_or(TeamError::NoAvailableWorkers)
    }

    /// Mark a task as completed
    pub async fn complete_task(&self, team_task_id: Uuid) -> Result<(), TeamError> {
        let team_task = TeamTask::find_by_id(&self.pool, team_task_id)
            .await?
            .ok_or(TeamError::TaskNotFound(team_task_id))?;

        TeamTask::complete(&self.pool, team_task_id).await?;

        // Update the associated task
        Task::update_status(&self.pool, team_task.task_id, TaskStatus::Done).await?;

        self.emit_event(TeamEvent::TaskCompleted { team_task_id })
            .await;

        // Emit progress update
        let progress = TeamTask::get_progress(&self.pool, team_task.team_execution_id).await?;
        self.emit_event(TeamEvent::ExecutionProgress { progress })
            .await;

        // Try to execute more tasks
        self.execute_ready_tasks(team_task.team_execution_id).await?;

        Ok(())
    }

    /// Mark a task as failed
    pub async fn fail_task(&self, team_task_id: Uuid, error: &str) -> Result<bool, TeamError> {
        let team_task = TeamTask::find_by_id(&self.pool, team_task_id)
            .await?
            .ok_or(TeamError::TaskNotFound(team_task_id))?;

        // Try to retry
        if TeamTask::retry(&self.pool, team_task_id).await? {
            tracing::info!("Task {} scheduled for retry", team_task_id);
            return Ok(true);
        }

        // Mark as failed
        TeamTask::fail(&self.pool, team_task_id, error).await?;
        Task::update_status(&self.pool, team_task.task_id, TaskStatus::Cancelled).await?;

        self.emit_event(TeamEvent::TaskFailed {
            team_task_id,
            error: error.to_string(),
        })
        .await;

        // Skip dependent tasks
        self.skip_dependent_tasks(team_task.team_execution_id, team_task_id)
            .await?;

        // Check if execution should fail
        let progress = TeamTask::get_progress(&self.pool, team_task.team_execution_id).await?;

        // If more than half failed, fail the execution
        if progress.failed > progress.total / 2 {
            TeamExecution::set_error(
                &self.pool,
                team_task.team_execution_id,
                "Too many tasks failed",
            )
            .await?;

            self.emit_event(TeamEvent::ExecutionFailed {
                team_execution_id: team_task.team_execution_id,
                error: "Too many tasks failed".to_string(),
            })
            .await;
        }

        Ok(false)
    }

    /// Skip tasks that depend on a failed task
    async fn skip_dependent_tasks(
        &self,
        team_execution_id: Uuid,
        failed_task_id: Uuid,
    ) -> Result<(), TeamError> {
        let all_tasks = TeamTask::find_by_team_execution(&self.pool, team_execution_id).await?;

        for task in all_tasks {
            let deps = task.get_dependencies();
            if deps.contains(&failed_task_id) && task.status == TeamTaskStatus::Pending {
                TeamTask::skip(&self.pool, task.id).await?;
                // Recursively skip tasks that depend on this one
                Box::pin(self.skip_dependent_tasks(team_execution_id, task.id)).await?;
            }
        }

        Ok(())
    }

    /// Pause a team execution
    pub async fn pause_execution(&self, team_execution_id: Uuid) -> Result<(), TeamError> {
        let execution = TeamExecution::find_by_id(&self.pool, team_execution_id)
            .await?
            .ok_or(TeamError::ExecutionNotFound(team_execution_id))?;

        if execution.status != TeamExecutionStatus::Executing {
            return Err(TeamError::InvalidStateTransition(
                "Can only pause executing teams".into(),
            ));
        }

        // Note: In a full implementation, this would also signal running agents to pause
        TeamExecution::update_status(
            &self.pool,
            team_execution_id,
            TeamExecutionStatus::Planned,
        )
        .await?;

        Ok(())
    }

    /// Resume a paused team execution
    pub async fn resume_execution(&self, team_execution_id: Uuid) -> Result<(), TeamError> {
        let execution = TeamExecution::find_by_id(&self.pool, team_execution_id)
            .await?
            .ok_or(TeamError::ExecutionNotFound(team_execution_id))?;

        if execution.status != TeamExecutionStatus::Planned {
            return Err(TeamError::InvalidStateTransition(
                "Can only resume planned/paused teams".into(),
            ));
        }

        TeamExecution::update_status(
            &self.pool,
            team_execution_id,
            TeamExecutionStatus::Executing,
        )
        .await?;

        // Start executing ready tasks
        self.execute_ready_tasks(team_execution_id).await?;

        Ok(())
    }

    /// Cancel a team execution
    pub async fn cancel_execution(&self, team_execution_id: Uuid) -> Result<(), TeamError> {
        // Mark all pending tasks as skipped
        let tasks = TeamTask::find_by_team_execution(&self.pool, team_execution_id).await?;

        for task in tasks {
            if task.status == TeamTaskStatus::Pending || task.status == TeamTaskStatus::Blocked {
                TeamTask::skip(&self.pool, task.id).await?;
            }
        }

        TeamExecution::update_status(
            &self.pool,
            team_execution_id,
            TeamExecutionStatus::Cancelled,
        )
        .await?;

        Ok(())
    }
}
