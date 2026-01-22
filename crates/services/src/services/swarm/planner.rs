//! Planner Agent Service
//!
//! Responsible for analyzing epic tasks and decomposing them into atomic subtasks.
//! The planner evaluates task complexity and determines whether to use single-agent
//! execution or swarm-based parallel execution.

use db::models::{
    agent_profile::AgentProfile,
    swarm_execution::{CreateSwarmExecution, PlannedSubtask, SwarmExecution, SwarmExecutionStatus, SwarmPlanOutput},
    swarm_task::{CreateSwarmTask, SwarmTask},
    task::{CreateTask, Task, TaskComplexity, TaskStatus},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PlannerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),
    #[error("Task is not an epic task")]
    NotEpicTask,
    #[error("No planner agent available")]
    NoPlannerAgent,
    #[error("Planning failed: {0}")]
    PlanningFailed(String),
    #[error("Invalid plan output: {0}")]
    InvalidPlanOutput(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Configuration for the planner service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerConfig {
    /// Minimum subtasks to trigger swarm execution
    pub swarm_threshold: i32,
    /// Maximum subtasks per epic
    pub max_subtasks: i32,
    /// Default reviewer count
    pub default_reviewer_count: i32,
    /// Maximum parallel workers
    pub max_parallel_workers: i32,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            swarm_threshold: 2,
            max_subtasks: 10,
            default_reviewer_count: 3,
            max_parallel_workers: 5,
        }
    }
}

/// Service for planning and decomposing epic tasks
pub struct PlannerService {
    pool: SqlitePool,
    config: PlannerConfig,
}

impl PlannerService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            config: PlannerConfig::default(),
        }
    }

    pub fn with_config(pool: SqlitePool, config: PlannerConfig) -> Self {
        Self { pool, config }
    }

    /// Analyze a task and determine its complexity
    pub async fn analyze_complexity(&self, task: &Task) -> TaskComplexity {
        let description_len = task.description.as_ref().map(|d| d.len()).unwrap_or(0);
        let title_complexity = self.estimate_title_complexity(&task.title);
        
        // Simple heuristics for complexity estimation
        // In production, this would use the planner agent
        match (description_len, title_complexity) {
            (0..=50, 1) => TaskComplexity::Trivial,
            (0..=200, 1..=2) => TaskComplexity::Simple,
            (0..=500, 1..=3) => TaskComplexity::Moderate,
            (0..=1000, _) => TaskComplexity::Complex,
            _ => TaskComplexity::Epic,
        }
    }

    fn estimate_title_complexity(&self, title: &str) -> i32 {
        let keywords = [
            "refactor", "implement", "build", "create", "design",
            "integrate", "migrate", "optimize", "architecture",
        ];
        
        let complexity_keywords = [
            "system", "framework", "platform", "engine", "complete",
            "full", "entire", "comprehensive", "end-to-end",
        ];

        let mut score = 1;
        let lower_title = title.to_lowercase();
        
        for kw in &keywords {
            if lower_title.contains(kw) {
                score += 1;
            }
        }
        
        for kw in &complexity_keywords {
            if lower_title.contains(kw) {
                score += 2;
            }
        }

        score.min(5)
    }

    /// Create a swarm execution for an epic task
    pub async fn create_swarm_execution(
        &self,
        epic_task_id: Uuid,
        workspace_id: Option<Uuid>,
    ) -> Result<SwarmExecution, PlannerError> {
        // Verify task exists and is epic
        let task = Task::find_by_id(&self.pool, epic_task_id)
            .await?
            .ok_or(PlannerError::TaskNotFound(epic_task_id))?;

        if !task.is_epic {
            return Err(PlannerError::NotEpicTask);
        }

        // Find a planner agent
        let planners = AgentProfile::find_planners(&self.pool).await?;
        let planner = planners.first().ok_or(PlannerError::NoPlannerAgent)?;

        // Create swarm execution
        let execution = SwarmExecution::create(
            &self.pool,
            &CreateSwarmExecution {
                epic_task_id,
                epic_workspace_id: workspace_id,
                planner_profile_id: Some(planner.id),
                reviewer_count: Some(self.config.default_reviewer_count),
                max_parallel_workers: Some(self.config.max_parallel_workers),
            },
        )
        .await?;

        Ok(execution)
    }

    /// Generate a decomposition plan for an epic task
    /// In production, this would invoke the planner agent
    pub async fn generate_plan(
        &self,
        swarm_execution_id: Uuid,
    ) -> Result<SwarmPlanOutput, PlannerError> {
        let execution = SwarmExecution::find_by_id(&self.pool, swarm_execution_id)
            .await?
            .ok_or(PlannerError::PlanningFailed("Execution not found".into()))?;

        let task = Task::find_by_id(&self.pool, execution.epic_task_id)
            .await?
            .ok_or(PlannerError::TaskNotFound(execution.epic_task_id))?;

        // Generate plan based on task analysis
        // This is a simplified version - in production, invoke the planner agent
        let plan = self.decompose_task(&task).await?;

        // Save plan output
        let plan_json = serde_json::to_string(&plan)?;
        SwarmExecution::set_planner_output(&self.pool, swarm_execution_id, &plan_json).await?;
        SwarmExecution::update_status(&self.pool, swarm_execution_id, SwarmExecutionStatus::Planned).await?;

        Ok(plan)
    }

    /// Decompose a task into subtasks
    async fn decompose_task(&self, task: &Task) -> Result<SwarmPlanOutput, PlannerError> {
        let complexity = self.analyze_complexity(task);
        
        // Simple rule-based decomposition for demonstration
        // In production, this would be done by the planner agent
        let subtasks = self.generate_subtasks(task, &complexity);
        let requires_swarm = subtasks.len() >= self.config.swarm_threshold as usize;

        Ok(SwarmPlanOutput {
            complexity: format!("{:?}", complexity),
            requires_swarm,
            subtasks,
            estimated_total_duration: None,
            reasoning: format!(
                "Task '{}' analyzed as {:?} complexity. {} subtasks identified.",
                task.title,
                complexity,
                subtasks.len()
            ),
        })
    }

    fn generate_subtasks(&self, task: &Task, complexity: &TaskComplexity) -> Vec<PlannedSubtask> {
        // This is a simplified implementation
        // In production, the planner agent would analyze the task and generate proper subtasks
        
        let base_subtasks = match complexity {
            TaskComplexity::Trivial | TaskComplexity::Simple => {
                vec![PlannedSubtask {
                    title: task.title.clone(),
                    description: task.description.clone().unwrap_or_default(),
                    required_skills: vec!["backend".to_string()],
                    depends_on: vec![],
                    complexity: 1,
                    estimated_duration: Some(30),
                }]
            }
            TaskComplexity::Moderate => {
                vec![
                    PlannedSubtask {
                        title: format!("Analyze requirements: {}", task.title),
                        description: "Analyze and document the requirements".to_string(),
                        required_skills: vec!["architecture".to_string()],
                        depends_on: vec![],
                        complexity: 2,
                        estimated_duration: Some(30),
                    },
                    PlannedSubtask {
                        title: format!("Implement: {}", task.title),
                        description: task.description.clone().unwrap_or_default(),
                        required_skills: vec!["backend".to_string(), "frontend".to_string()],
                        depends_on: vec![0],
                        complexity: 3,
                        estimated_duration: Some(60),
                    },
                    PlannedSubtask {
                        title: format!("Test: {}", task.title),
                        description: "Write tests and verify implementation".to_string(),
                        required_skills: vec!["testing".to_string()],
                        depends_on: vec![1],
                        complexity: 2,
                        estimated_duration: Some(30),
                    },
                ]
            }
            TaskComplexity::Complex | TaskComplexity::Epic => {
                vec![
                    PlannedSubtask {
                        title: format!("Architecture design: {}", task.title),
                        description: "Design the overall architecture and components".to_string(),
                        required_skills: vec!["architecture".to_string()],
                        depends_on: vec![],
                        complexity: 3,
                        estimated_duration: Some(45),
                    },
                    PlannedSubtask {
                        title: "Backend implementation".to_string(),
                        description: "Implement backend services and APIs".to_string(),
                        required_skills: vec!["backend".to_string(), "database".to_string()],
                        depends_on: vec![0],
                        complexity: 4,
                        estimated_duration: Some(90),
                    },
                    PlannedSubtask {
                        title: "Frontend implementation".to_string(),
                        description: "Implement frontend components and UI".to_string(),
                        required_skills: vec!["frontend".to_string()],
                        depends_on: vec![0],
                        complexity: 4,
                        estimated_duration: Some(90),
                    },
                    PlannedSubtask {
                        title: "Integration".to_string(),
                        description: "Integrate frontend and backend components".to_string(),
                        required_skills: vec!["backend".to_string(), "frontend".to_string()],
                        depends_on: vec![1, 2],
                        complexity: 3,
                        estimated_duration: Some(45),
                    },
                    PlannedSubtask {
                        title: "Testing and QA".to_string(),
                        description: "Comprehensive testing and quality assurance".to_string(),
                        required_skills: vec!["testing".to_string()],
                        depends_on: vec![3],
                        complexity: 3,
                        estimated_duration: Some(60),
                    },
                    PlannedSubtask {
                        title: "Documentation".to_string(),
                        description: "Write documentation and update README".to_string(),
                        required_skills: vec!["documentation".to_string()],
                        depends_on: vec![3],
                        complexity: 2,
                        estimated_duration: Some(30),
                    },
                ]
            }
        };

        base_subtasks
    }

    /// Create actual tasks and swarm tasks from a plan
    pub async fn execute_plan(
        &self,
        swarm_execution_id: Uuid,
        plan: &SwarmPlanOutput,
    ) -> Result<Vec<SwarmTask>, PlannerError> {
        let execution = SwarmExecution::find_by_id(&self.pool, swarm_execution_id)
            .await?
            .ok_or(PlannerError::PlanningFailed("Execution not found".into()))?;

        let epic_task = Task::find_by_id(&self.pool, execution.epic_task_id)
            .await?
            .ok_or(PlannerError::TaskNotFound(execution.epic_task_id))?;

        let mut swarm_tasks = Vec::new();
        let mut task_id_map: std::collections::HashMap<usize, Uuid> = std::collections::HashMap::new();

        for (idx, planned) in plan.subtasks.iter().enumerate() {
            // Create the actual task
            let task = Task::create(
                &self.pool,
                &CreateTask {
                    project_id: epic_task.project_id,
                    title: planned.title.clone(),
                    description: Some(planned.description.clone()),
                    status: Some(TaskStatus::Todo),
                    parent_workspace_id: execution.epic_workspace_id,
                    image_ids: None,
                    is_epic: Some(false),
                    complexity: Some(match planned.complexity {
                        1 => TaskComplexity::Trivial,
                        2 => TaskComplexity::Simple,
                        3 => TaskComplexity::Moderate,
                        4 => TaskComplexity::Complex,
                        _ => TaskComplexity::Epic,
                    }),
                    metadata: None,
                },
                Uuid::new_v4(),
            )
            .await?;

            task_id_map.insert(idx, task.id);

            // Map dependency indices to UUIDs
            let depends_on: Vec<Uuid> = planned
                .depends_on
                .iter()
                .filter_map(|&dep_idx| task_id_map.get(&(dep_idx as usize)).copied())
                .collect();

            // Create the swarm task
            let swarm_task = SwarmTask::create(
                &self.pool,
                &CreateSwarmTask {
                    swarm_execution_id,
                    task_id: task.id,
                    sequence_order: idx as i32,
                    depends_on: if depends_on.is_empty() {
                        None
                    } else {
                        Some(depends_on)
                    },
                    required_skills: Some(planned.required_skills.clone()),
                    complexity: Some(planned.complexity),
                    max_retries: Some(2),
                },
            )
            .await?;

            swarm_tasks.push(swarm_task);
        }

        // Update execution status
        SwarmExecution::update_status(&self.pool, swarm_execution_id, SwarmExecutionStatus::Executing)
            .await?;

        Ok(swarm_tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_complexity() {
        let service = PlannerService {
            pool: unsafe { std::mem::zeroed() }, // Not used in this test
            config: PlannerConfig::default(),
        };

        assert!(service.estimate_title_complexity("Fix bug") <= 2);
        assert!(service.estimate_title_complexity("Implement new feature") >= 2);
        assert!(service.estimate_title_complexity("Build complete authentication system") >= 4);
    }
}
