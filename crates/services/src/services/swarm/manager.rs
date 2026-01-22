//! Swarm Manager Service
//!
//! Orchestrates parallel execution of swarm tasks, managing agent assignment,
//! workspace creation, and execution coordination.

use db::models::{
    agent_profile::AgentProfile,
    agent_skill::AgentSkill,
    swarm_execution::{SwarmExecution, SwarmExecutionStatus},
    swarm_task::{SwarmProgress, SwarmTask, SwarmTaskStatus},
    task::{Task, TaskStatus},
    workspace::{CreateWorkspace, Workspace},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SwarmError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Swarm execution not found: {0}")]
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

/// Event types for swarm execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwarmEvent {
    TaskStarted { swarm_task_id: Uuid, agent_id: Uuid },
    TaskCompleted { swarm_task_id: Uuid },
    TaskFailed { swarm_task_id: Uuid, error: String },
    ExecutionProgress { progress: SwarmProgress },
    ExecutionCompleted { swarm_execution_id: Uuid },
    ExecutionFailed { swarm_execution_id: Uuid, error: String },
    ConsensusRequired { swarm_execution_id: Uuid },
}

/// Configuration for the swarm manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmManagerConfig {
    /// Maximum parallel tasks
    pub max_parallel_tasks: i32,
    /// Task timeout in seconds
    pub task_timeout_seconds: i64,
    /// Retry delay in seconds
    pub retry_delay_seconds: i64,
    /// Branch prefix for swarm tasks
    pub branch_prefix: String,
}

impl Default for SwarmManagerConfig {
    fn default() -> Self {
        Self {
            max_parallel_tasks: 5,
            task_timeout_seconds: 3600, // 1 hour
            retry_delay_seconds: 30,
            branch_prefix: "swarm".to_string(),
        }
    }
}

/// Manages swarm execution orchestration
pub struct SwarmManager {
    pool: SqlitePool,
    config: SwarmManagerConfig,
    event_sender: Option<mpsc::Sender<SwarmEvent>>,
}

impl SwarmManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            config: SwarmManagerConfig::default(),
            event_sender: None,
        }
    }

    pub fn with_config(pool: SqlitePool, config: SwarmManagerConfig) -> Self {
        Self {
            pool,
            config,
            event_sender: None,
        }
    }

    pub fn with_event_channel(mut self, sender: mpsc::Sender<SwarmEvent>) -> Self {
        self.event_sender = Some(sender);
        self
    }

    async fn emit_event(&self, event: SwarmEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event).await;
        }
    }

    /// Get the current status of a swarm execution
    pub async fn get_status(
        &self,
        swarm_execution_id: Uuid,
    ) -> Result<SwarmExecutionStatus, SwarmError> {
        let execution = SwarmExecution::find_by_id(&self.pool, swarm_execution_id)
            .await?
            .ok_or(SwarmError::ExecutionNotFound(swarm_execution_id))?;

        Ok(execution.status)
    }

    /// Get progress of a swarm execution
    pub async fn get_progress(
        &self,
        swarm_execution_id: Uuid,
    ) -> Result<SwarmProgress, SwarmError> {
        let progress = SwarmTask::get_progress(&self.pool, swarm_execution_id).await?;
        Ok(progress)
    }

    /// Start executing ready tasks in a swarm
    pub async fn execute_ready_tasks(
        &self,
        swarm_execution_id: Uuid,
    ) -> Result<Vec<Uuid>, SwarmError> {
        let execution = SwarmExecution::find_by_id(&self.pool, swarm_execution_id)
            .await?
            .ok_or(SwarmError::ExecutionNotFound(swarm_execution_id))?;

        if execution.status != SwarmExecutionStatus::Executing {
            return Err(SwarmError::InvalidStateTransition(
                "Execution is not in executing state".into(),
            ));
        }

        // Get currently running tasks
        let running_tasks = SwarmTask::find_running_tasks(&self.pool, swarm_execution_id).await?;
        let available_slots =
            (execution.max_parallel_workers as usize).saturating_sub(running_tasks.len());

        if available_slots == 0 {
            return Ok(vec![]);
        }

        // Get ready tasks
        let ready_tasks = SwarmTask::find_ready_tasks(&self.pool, swarm_execution_id).await?;
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
        if SwarmTask::all_completed(&self.pool, swarm_execution_id).await? {
            // Transition to review phase
            SwarmExecution::update_status(
                &self.pool,
                swarm_execution_id,
                SwarmExecutionStatus::Reviewing,
            )
            .await?;
            
            self.emit_event(SwarmEvent::ConsensusRequired { swarm_execution_id })
                .await;
        }

        Ok(started_task_ids)
    }

    /// Start execution of a single task
    async fn start_task(&self, swarm_task: &SwarmTask) -> Result<(), SwarmError> {
        // Find an available agent with required skills
        let agent = self.find_best_agent(swarm_task).await?;

        // Assign the agent
        SwarmTask::assign_agent(&self.pool, swarm_task.id, agent.id).await?;

        // Create workspace and branch for this task
        let execution = SwarmExecution::find_by_id(&self.pool, swarm_task.swarm_execution_id)
            .await?
            .ok_or(SwarmError::ExecutionNotFound(swarm_task.swarm_execution_id))?;

        let branch_name = format!(
            "{}/task-{}",
            self.config.branch_prefix,
            &swarm_task.id.to_string()[..8]
        );

        // Get the task for this swarm task
        let task = Task::find_by_id(&self.pool, swarm_task.task_id)
            .await?
            .ok_or(SwarmError::TaskNotFound(swarm_task.task_id))?;

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
        .map_err(|e| SwarmError::Database(e.into()))?;

        // Update swarm task with workspace info
        SwarmTask::set_workspace(&self.pool, swarm_task.id, workspace.id, &branch_name).await?;
        SwarmTask::start(&self.pool, swarm_task.id).await?;

        // Update task status
        Task::update_status(&self.pool, task.id, TaskStatus::InProgress).await?;

        self.emit_event(SwarmEvent::TaskStarted {
            swarm_task_id: swarm_task.id,
            agent_id: agent.id,
        })
        .await;

        Ok(())
    }

    /// Find the best available agent for a task based on required skills
    async fn find_best_agent(&self, swarm_task: &SwarmTask) -> Result<AgentProfile, SwarmError> {
        let required_skills = swarm_task.get_required_skills();

        // Get all workers
        let workers = AgentProfile::find_workers(&self.pool).await?;

        if workers.is_empty() {
            return Err(SwarmError::NoAvailableWorkers);
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

        best_worker.ok_or(SwarmError::NoAvailableWorkers)
    }

    /// Mark a task as completed
    pub async fn complete_task(&self, swarm_task_id: Uuid) -> Result<(), SwarmError> {
        let swarm_task = SwarmTask::find_by_id(&self.pool, swarm_task_id)
            .await?
            .ok_or(SwarmError::TaskNotFound(swarm_task_id))?;

        SwarmTask::complete(&self.pool, swarm_task_id).await?;

        // Update the associated task
        Task::update_status(&self.pool, swarm_task.task_id, TaskStatus::Done).await?;

        self.emit_event(SwarmEvent::TaskCompleted { swarm_task_id })
            .await;

        // Emit progress update
        let progress = SwarmTask::get_progress(&self.pool, swarm_task.swarm_execution_id).await?;
        self.emit_event(SwarmEvent::ExecutionProgress { progress })
            .await;

        // Try to execute more tasks
        self.execute_ready_tasks(swarm_task.swarm_execution_id)
            .await?;

        Ok(())
    }

    /// Mark a task as failed
    pub async fn fail_task(&self, swarm_task_id: Uuid, error: &str) -> Result<bool, SwarmError> {
        let swarm_task = SwarmTask::find_by_id(&self.pool, swarm_task_id)
            .await?
            .ok_or(SwarmError::TaskNotFound(swarm_task_id))?;

        // Try to retry
        if SwarmTask::retry(&self.pool, swarm_task_id).await? {
            tracing::info!("Task {} scheduled for retry", swarm_task_id);
            return Ok(true);
        }

        // Mark as failed
        SwarmTask::fail(&self.pool, swarm_task_id, error).await?;
        Task::update_status(&self.pool, swarm_task.task_id, TaskStatus::Cancelled).await?;

        self.emit_event(SwarmEvent::TaskFailed {
            swarm_task_id,
            error: error.to_string(),
        })
        .await;

        // Skip dependent tasks
        self.skip_dependent_tasks(swarm_task.swarm_execution_id, swarm_task_id)
            .await?;

        // Check if execution should fail
        let progress = SwarmTask::get_progress(&self.pool, swarm_task.swarm_execution_id).await?;
        
        // If more than half failed, fail the execution
        if progress.failed > progress.total / 2 {
            SwarmExecution::set_error(
                &self.pool,
                swarm_task.swarm_execution_id,
                "Too many tasks failed",
            )
            .await?;

            self.emit_event(SwarmEvent::ExecutionFailed {
                swarm_execution_id: swarm_task.swarm_execution_id,
                error: "Too many tasks failed".to_string(),
            })
            .await;
        }

        Ok(false)
    }

    /// Skip tasks that depend on a failed task
    async fn skip_dependent_tasks(
        &self,
        swarm_execution_id: Uuid,
        failed_task_id: Uuid,
    ) -> Result<(), SwarmError> {
        let all_tasks = SwarmTask::find_by_swarm_execution(&self.pool, swarm_execution_id).await?;

        for task in all_tasks {
            let deps = task.get_dependencies();
            if deps.contains(&failed_task_id) && task.status == SwarmTaskStatus::Pending {
                SwarmTask::skip(&self.pool, task.id).await?;
                // Recursively skip tasks that depend on this one
                Box::pin(self.skip_dependent_tasks(swarm_execution_id, task.id)).await?;
            }
        }

        Ok(())
    }

    /// Pause a swarm execution
    pub async fn pause_execution(&self, swarm_execution_id: Uuid) -> Result<(), SwarmError> {
        let execution = SwarmExecution::find_by_id(&self.pool, swarm_execution_id)
            .await?
            .ok_or(SwarmError::ExecutionNotFound(swarm_execution_id))?;

        if execution.status != SwarmExecutionStatus::Executing {
            return Err(SwarmError::InvalidStateTransition(
                "Can only pause executing swarms".into(),
            ));
        }

        // Note: In a full implementation, this would also signal running agents to pause
        SwarmExecution::update_status(&self.pool, swarm_execution_id, SwarmExecutionStatus::Planned)
            .await?;

        Ok(())
    }

    /// Resume a paused swarm execution
    pub async fn resume_execution(&self, swarm_execution_id: Uuid) -> Result<(), SwarmError> {
        let execution = SwarmExecution::find_by_id(&self.pool, swarm_execution_id)
            .await?
            .ok_or(SwarmError::ExecutionNotFound(swarm_execution_id))?;

        if execution.status != SwarmExecutionStatus::Planned {
            return Err(SwarmError::InvalidStateTransition(
                "Can only resume planned/paused swarms".into(),
            ));
        }

        SwarmExecution::update_status(
            &self.pool,
            swarm_execution_id,
            SwarmExecutionStatus::Executing,
        )
        .await?;

        // Start executing ready tasks
        self.execute_ready_tasks(swarm_execution_id).await?;

        Ok(())
    }

    /// Cancel a swarm execution
    pub async fn cancel_execution(&self, swarm_execution_id: Uuid) -> Result<(), SwarmError> {
        // Mark all pending tasks as skipped
        let tasks = SwarmTask::find_by_swarm_execution(&self.pool, swarm_execution_id).await?;
        
        for task in tasks {
            if task.status == SwarmTaskStatus::Pending || task.status == SwarmTaskStatus::Blocked {
                SwarmTask::skip(&self.pool, task.id).await?;
            }
        }

        SwarmExecution::update_status(
            &self.pool,
            swarm_execution_id,
            SwarmExecutionStatus::Cancelled,
        )
        .await?;

        Ok(())
    }
}
