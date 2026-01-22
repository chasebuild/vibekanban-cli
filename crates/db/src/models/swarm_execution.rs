use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Type};
use strum_macros::{Display, EnumString};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq, TS, EnumString, Display, Default)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SwarmExecutionStatus {
    #[default]
    Planning,
    Planned,
    Executing,
    Reviewing,
    Merging,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct SwarmExecution {
    pub id: Uuid,
    pub epic_task_id: Uuid,
    pub epic_workspace_id: Option<Uuid>,
    pub status: SwarmExecutionStatus,
    pub planner_output: Option<String>,
    pub planner_profile_id: Option<Uuid>,
    pub reviewer_count: i32,
    pub consensus_threshold: i32,
    pub consensus_approvals: i32,
    pub consensus_rejections: i32,
    pub max_parallel_workers: i32,
    pub error_message: Option<String>,
    pub planned_at: Option<DateTime<Utc>>,
    pub execution_started_at: Option<DateTime<Utc>>,
    pub review_started_at: Option<DateTime<Utc>>,
    pub merge_started_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct CreateSwarmExecution {
    pub epic_task_id: Uuid,
    pub epic_workspace_id: Option<Uuid>,
    pub planner_profile_id: Option<Uuid>,
    pub reviewer_count: Option<i32>,
    pub max_parallel_workers: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct UpdateSwarmExecution {
    pub status: Option<SwarmExecutionStatus>,
    pub planner_output: Option<String>,
    pub error_message: Option<String>,
    pub consensus_approvals: Option<i32>,
    pub consensus_rejections: Option<i32>,
}

/// Plan output from the planner agent
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SwarmPlanOutput {
    pub complexity: String,
    pub requires_swarm: bool,
    pub subtasks: Vec<PlannedSubtask>,
    pub estimated_total_duration: Option<i32>,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct PlannedSubtask {
    pub title: String,
    pub description: String,
    pub required_skills: Vec<String>,
    pub depends_on: Vec<i32>, // Indices of dependent tasks
    pub complexity: i32,      // 1-5
    pub estimated_duration: Option<i32>, // minutes
}

impl SwarmExecution {
    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            SwarmExecution,
            r#"SELECT 
                id AS "id!: Uuid",
                epic_task_id AS "epic_task_id!: Uuid",
                epic_workspace_id AS "epic_workspace_id: Uuid",
                status AS "status!: SwarmExecutionStatus",
                planner_output,
                planner_profile_id AS "planner_profile_id: Uuid",
                reviewer_count AS "reviewer_count!: i32",
                consensus_threshold AS "consensus_threshold!: i32",
                consensus_approvals AS "consensus_approvals!: i32",
                consensus_rejections AS "consensus_rejections!: i32",
                max_parallel_workers AS "max_parallel_workers!: i32",
                error_message,
                planned_at AS "planned_at: DateTime<Utc>",
                execution_started_at AS "execution_started_at: DateTime<Utc>",
                review_started_at AS "review_started_at: DateTime<Utc>",
                merge_started_at AS "merge_started_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM swarm_executions
            WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_epic_task(
        pool: &SqlitePool,
        epic_task_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            SwarmExecution,
            r#"SELECT 
                id AS "id!: Uuid",
                epic_task_id AS "epic_task_id!: Uuid",
                epic_workspace_id AS "epic_workspace_id: Uuid",
                status AS "status!: SwarmExecutionStatus",
                planner_output,
                planner_profile_id AS "planner_profile_id: Uuid",
                reviewer_count AS "reviewer_count!: i32",
                consensus_threshold AS "consensus_threshold!: i32",
                consensus_approvals AS "consensus_approvals!: i32",
                consensus_rejections AS "consensus_rejections!: i32",
                max_parallel_workers AS "max_parallel_workers!: i32",
                error_message,
                planned_at AS "planned_at: DateTime<Utc>",
                execution_started_at AS "execution_started_at: DateTime<Utc>",
                review_started_at AS "review_started_at: DateTime<Utc>",
                merge_started_at AS "merge_started_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM swarm_executions
            WHERE epic_task_id = $1
            ORDER BY created_at DESC"#,
            epic_task_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_active(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            SwarmExecution,
            r#"SELECT 
                id AS "id!: Uuid",
                epic_task_id AS "epic_task_id!: Uuid",
                epic_workspace_id AS "epic_workspace_id: Uuid",
                status AS "status!: SwarmExecutionStatus",
                planner_output,
                planner_profile_id AS "planner_profile_id: Uuid",
                reviewer_count AS "reviewer_count!: i32",
                consensus_threshold AS "consensus_threshold!: i32",
                consensus_approvals AS "consensus_approvals!: i32",
                consensus_rejections AS "consensus_rejections!: i32",
                max_parallel_workers AS "max_parallel_workers!: i32",
                error_message,
                planned_at AS "planned_at: DateTime<Utc>",
                execution_started_at AS "execution_started_at: DateTime<Utc>",
                review_started_at AS "review_started_at: DateTime<Utc>",
                merge_started_at AS "merge_started_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM swarm_executions
            WHERE status IN ('planning', 'planned', 'executing', 'reviewing', 'merging')
            ORDER BY created_at DESC"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, data: &CreateSwarmExecution) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        let reviewer_count = data.reviewer_count.unwrap_or(3);
        // pBFT: 2f+1 where f = floor((N-1)/3)
        let f = (reviewer_count - 1) / 3;
        let consensus_threshold = 2 * f + 1;
        let max_parallel = data.max_parallel_workers.unwrap_or(3);

        sqlx::query_as!(
            SwarmExecution,
            r#"INSERT INTO swarm_executions 
                (id, epic_task_id, epic_workspace_id, planner_profile_id, reviewer_count, consensus_threshold, max_parallel_workers)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING 
                id AS "id!: Uuid",
                epic_task_id AS "epic_task_id!: Uuid",
                epic_workspace_id AS "epic_workspace_id: Uuid",
                status AS "status!: SwarmExecutionStatus",
                planner_output,
                planner_profile_id AS "planner_profile_id: Uuid",
                reviewer_count AS "reviewer_count!: i32",
                consensus_threshold AS "consensus_threshold!: i32",
                consensus_approvals AS "consensus_approvals!: i32",
                consensus_rejections AS "consensus_rejections!: i32",
                max_parallel_workers AS "max_parallel_workers!: i32",
                error_message,
                planned_at AS "planned_at: DateTime<Utc>",
                execution_started_at AS "execution_started_at: DateTime<Utc>",
                review_started_at AS "review_started_at: DateTime<Utc>",
                merge_started_at AS "merge_started_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>""#,
            id,
            data.epic_task_id,
            data.epic_workspace_id,
            data.planner_profile_id,
            reviewer_count,
            consensus_threshold,
            max_parallel
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update_status(
        pool: &SqlitePool,
        id: Uuid,
        status: SwarmExecutionStatus,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        
        // Update the appropriate timestamp based on status transition
        match status {
            SwarmExecutionStatus::Planned => {
                sqlx::query!(
                    "UPDATE swarm_executions SET status = $2, planned_at = $3, updated_at = $3 WHERE id = $1",
                    id, status, now
                ).execute(pool).await?;
            }
            SwarmExecutionStatus::Executing => {
                sqlx::query!(
                    "UPDATE swarm_executions SET status = $2, execution_started_at = $3, updated_at = $3 WHERE id = $1",
                    id, status, now
                ).execute(pool).await?;
            }
            SwarmExecutionStatus::Reviewing => {
                sqlx::query!(
                    "UPDATE swarm_executions SET status = $2, review_started_at = $3, updated_at = $3 WHERE id = $1",
                    id, status, now
                ).execute(pool).await?;
            }
            SwarmExecutionStatus::Merging => {
                sqlx::query!(
                    "UPDATE swarm_executions SET status = $2, merge_started_at = $3, updated_at = $3 WHERE id = $1",
                    id, status, now
                ).execute(pool).await?;
            }
            SwarmExecutionStatus::Completed | SwarmExecutionStatus::Failed | SwarmExecutionStatus::Cancelled => {
                sqlx::query!(
                    "UPDATE swarm_executions SET status = $2, completed_at = $3, updated_at = $3 WHERE id = $1",
                    id, status, now
                ).execute(pool).await?;
            }
            _ => {
                sqlx::query!(
                    "UPDATE swarm_executions SET status = $2, updated_at = $3 WHERE id = $1",
                    id, status, now
                ).execute(pool).await?;
            }
        }
        
        Ok(())
    }

    pub async fn set_planner_output(
        pool: &SqlitePool,
        id: Uuid,
        output: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE swarm_executions SET planner_output = $2, updated_at = datetime('now', 'subsec') WHERE id = $1",
            id, output
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn set_error(
        pool: &SqlitePool,
        id: Uuid,
        error: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE swarm_executions SET error_message = $2, status = 'failed', completed_at = datetime('now', 'subsec'), updated_at = datetime('now', 'subsec') WHERE id = $1",
            id, error
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn increment_approval(pool: &SqlitePool, id: Uuid) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"UPDATE swarm_executions 
               SET consensus_approvals = consensus_approvals + 1, updated_at = datetime('now', 'subsec') 
               WHERE id = $1
               RETURNING consensus_approvals AS "count!: i32""#,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(result.count)
    }

    pub async fn increment_rejection(pool: &SqlitePool, id: Uuid) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"UPDATE swarm_executions 
               SET consensus_rejections = consensus_rejections + 1, updated_at = datetime('now', 'subsec') 
               WHERE id = $1
               RETURNING consensus_rejections AS "count!: i32""#,
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(result.count)
    }

    /// Check if consensus has been reached
    pub fn has_consensus(&self) -> bool {
        self.consensus_approvals >= self.consensus_threshold
    }

    /// Check if consensus has failed (too many rejections)
    pub fn consensus_failed(&self) -> bool {
        self.consensus_rejections > self.reviewer_count - self.consensus_threshold
    }

    pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM swarm_executions WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
