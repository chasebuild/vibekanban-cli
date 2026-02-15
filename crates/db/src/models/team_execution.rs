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
pub enum TeamExecutionStatus {
    #[default]
    Planning,
    Planned,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct TeamExecution {
    pub id: Uuid,
    pub epic_task_id: Uuid,
    pub epic_workspace_id: Option<Uuid>,
    pub status: TeamExecutionStatus,
    pub planner_output: Option<String>,
    pub planner_profile_id: Option<Uuid>,
    pub max_parallel_workers: i32,
    pub error_message: Option<String>,
    pub planned_at: Option<DateTime<Utc>>,
    pub execution_started_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct CreateTeamExecution {
    pub epic_task_id: Uuid,
    pub epic_workspace_id: Option<Uuid>,
    pub planner_profile_id: Option<Uuid>,
    pub max_parallel_workers: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct UpdateTeamExecution {
    pub status: Option<TeamExecutionStatus>,
    pub planner_output: Option<String>,
    pub error_message: Option<String>,
}

/// Plan output from the planner agent
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct TeamPlanOutput {
    pub complexity: String,
    pub requires_team: bool,
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

impl TeamExecution {
    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            TeamExecution,
            r#"SELECT 
                id AS "id!: Uuid",
                epic_task_id AS "epic_task_id!: Uuid",
                epic_workspace_id AS "epic_workspace_id: Uuid",
                status AS "status!: TeamExecutionStatus",
                planner_output,
                planner_profile_id AS "planner_profile_id: Uuid",
                max_parallel_workers AS "max_parallel_workers!: i32",
                error_message,
                planned_at AS "planned_at: DateTime<Utc>",
                execution_started_at AS "execution_started_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM team_executions
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
            TeamExecution,
            r#"SELECT 
                id AS "id!: Uuid",
                epic_task_id AS "epic_task_id!: Uuid",
                epic_workspace_id AS "epic_workspace_id: Uuid",
                status AS "status!: TeamExecutionStatus",
                planner_output,
                planner_profile_id AS "planner_profile_id: Uuid",
                max_parallel_workers AS "max_parallel_workers!: i32",
                error_message,
                planned_at AS "planned_at: DateTime<Utc>",
                execution_started_at AS "execution_started_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM team_executions
            WHERE epic_task_id = $1
            ORDER BY created_at DESC"#,
            epic_task_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_active(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            TeamExecution,
            r#"SELECT 
                id AS "id!: Uuid",
                epic_task_id AS "epic_task_id!: Uuid",
                epic_workspace_id AS "epic_workspace_id: Uuid",
                status AS "status!: TeamExecutionStatus",
                planner_output,
                planner_profile_id AS "planner_profile_id: Uuid",
                max_parallel_workers AS "max_parallel_workers!: i32",
                error_message,
                planned_at AS "planned_at: DateTime<Utc>",
                execution_started_at AS "execution_started_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM team_executions
            WHERE status IN ('planning', 'planned', 'executing')
            ORDER BY created_at DESC"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, data: &CreateTeamExecution) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        let max_parallel = data.max_parallel_workers.unwrap_or(3);

        sqlx::query_as!(
            TeamExecution,
            r#"INSERT INTO team_executions 
                (id, epic_task_id, epic_workspace_id, planner_profile_id, max_parallel_workers)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING 
                id AS "id!: Uuid",
                epic_task_id AS "epic_task_id!: Uuid",
                epic_workspace_id AS "epic_workspace_id: Uuid",
                status AS "status!: TeamExecutionStatus",
                planner_output,
                planner_profile_id AS "planner_profile_id: Uuid",
                max_parallel_workers AS "max_parallel_workers!: i32",
                error_message,
                planned_at AS "planned_at: DateTime<Utc>",
                execution_started_at AS "execution_started_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>""#,
            id,
            data.epic_task_id,
            data.epic_workspace_id,
            data.planner_profile_id,
            max_parallel
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update_status(
        pool: &SqlitePool,
        id: Uuid,
        status: TeamExecutionStatus,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        // Update the appropriate timestamp based on status transition
        match status {
            TeamExecutionStatus::Planned => {
                sqlx::query!(
                    "UPDATE team_executions SET status = $2, planned_at = $3, updated_at = $3 WHERE id = $1",
                    id, status, now
                )
                .execute(pool)
                .await?;
            }
            TeamExecutionStatus::Executing => {
                sqlx::query!(
                    "UPDATE team_executions SET status = $2, execution_started_at = $3, updated_at = $3 WHERE id = $1",
                    id, status, now
                )
                .execute(pool)
                .await?;
            }
            TeamExecutionStatus::Completed
            | TeamExecutionStatus::Failed
            | TeamExecutionStatus::Cancelled => {
                sqlx::query!(
                    "UPDATE team_executions SET status = $2, completed_at = $3, updated_at = $3 WHERE id = $1",
                    id, status, now
                )
                .execute(pool)
                .await?;
            }
            _ => {
                sqlx::query!(
                    "UPDATE team_executions SET status = $2, updated_at = $3 WHERE id = $1",
                    id, status, now
                )
                .execute(pool)
                .await?;
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
            "UPDATE team_executions SET planner_output = $2, updated_at = datetime('now', 'subsec') WHERE id = $1",
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
            "UPDATE team_executions SET error_message = $2, status = 'failed', completed_at = datetime('now', 'subsec'), updated_at = datetime('now', 'subsec') WHERE id = $1",
            id, error
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM team_executions WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
