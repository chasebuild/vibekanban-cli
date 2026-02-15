use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Type};
use strum_macros::{Display, EnumString};
use ts_rs::TS;
use uuid::Uuid;

#[derive(
    Debug, Clone, Type, Serialize, Deserialize, PartialEq, TS, EnumString, Display, Default,
)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum TeamTaskStatus {
    #[default]
    Pending,
    Blocked,
    Assigned,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct TeamTask {
    pub id: Uuid,
    pub team_execution_id: Uuid,
    pub task_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub sequence_order: i32,
    pub depends_on: Option<String>,
    pub required_skills: Option<String>,
    pub assigned_agent_profile_id: Option<Uuid>,
    pub status: TeamTaskStatus,
    pub branch_name: Option<String>,
    pub complexity: i32,
    pub duration_seconds: Option<i32>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct CreateTeamTask {
    pub team_execution_id: Uuid,
    pub task_id: Uuid,
    pub sequence_order: i32,
    pub depends_on: Option<Vec<Uuid>>,
    pub required_skills: Option<Vec<String>>,
    pub complexity: Option<i32>,
    pub max_retries: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct TeamTaskWithDetails {
    pub task_title: String,
    pub task_description: Option<String>,
    pub agent_name: Option<String>,
    #[serde(flatten)]
    pub team_task: TeamTask,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct TeamProgress {
    pub total: i32,
    pub completed: i32,
    pub running: i32,
    pub failed: i32,
    pub pending: i32,
    pub skipped: i32,
}

impl TeamTask {
    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            TeamTask,
            r#"SELECT 
                id AS "id!: Uuid",
                team_execution_id AS "team_execution_id!: Uuid",
                task_id AS "task_id!: Uuid",
                workspace_id AS "workspace_id: Uuid",
                sequence_order AS "sequence_order!: i32",
                depends_on,
                required_skills,
                assigned_agent_profile_id AS "assigned_agent_profile_id: Uuid",
                status AS "status!: TeamTaskStatus",
                branch_name,
                complexity AS "complexity!: i32",
                duration_seconds AS "duration_seconds: i32",
                error_message,
                retry_count AS "retry_count!: i32",
                max_retries AS "max_retries!: i32",
                started_at AS "started_at: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM team_tasks
            WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_team_execution(
        pool: &SqlitePool,
        team_execution_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            TeamTask,
            r#"SELECT 
                id AS "id!: Uuid",
                team_execution_id AS "team_execution_id!: Uuid",
                task_id AS "task_id!: Uuid",
                workspace_id AS "workspace_id: Uuid",
                sequence_order AS "sequence_order!: i32",
                depends_on,
                required_skills,
                assigned_agent_profile_id AS "assigned_agent_profile_id: Uuid",
                status AS "status!: TeamTaskStatus",
                branch_name,
                complexity AS "complexity!: i32",
                duration_seconds AS "duration_seconds: i32",
                error_message,
                retry_count AS "retry_count!: i32",
                max_retries AS "max_retries!: i32",
                started_at AS "started_at: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM team_tasks
            WHERE team_execution_id = $1
            ORDER BY sequence_order"#,
            team_execution_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_with_details(
        pool: &SqlitePool,
        team_execution_id: Uuid,
    ) -> Result<Vec<TeamTaskWithDetails>, sqlx::Error> {
        let all_tasks = Self::find_by_team_execution(pool, team_execution_id).await?;
        let mut result = Vec::with_capacity(all_tasks.len());

        for task in all_tasks {
            let task_details = sqlx::query!(
                r#"SELECT 
                    tasks.title AS "task_title!: String",
                    tasks.description AS "task_description: String",
                    agent_profiles.name AS "agent_name: String"
                FROM team_tasks
                JOIN tasks ON team_tasks.task_id = tasks.id
                LEFT JOIN agent_profiles ON agent_profiles.id = team_tasks.assigned_agent_profile_id
                WHERE team_tasks.id = $1"#,
                task.id
            )
            .fetch_optional(pool)
            .await?;

            let (task_title, task_description, agent_name) = match task_details {
                Some(details) => (
                    details.task_title,
                    details.task_description,
                    Some(details.agent_name),
                ),
                None => (String::new(), None, None),
            };

            result.push(TeamTaskWithDetails {
                task_title,
                task_description,
                agent_name,
                team_task: task,
            });
        }

        Ok(result)
    }

    pub async fn find_ready_tasks(
        pool: &SqlitePool,
        team_execution_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            TeamTask,
            r#"SELECT 
                id AS "id!: Uuid",
                team_execution_id AS "team_execution_id!: Uuid",
                task_id AS "task_id!: Uuid",
                workspace_id AS "workspace_id: Uuid",
                sequence_order AS "sequence_order!: i32",
                depends_on,
                required_skills,
                assigned_agent_profile_id AS "assigned_agent_profile_id: Uuid",
                status AS "status!: TeamTaskStatus",
                branch_name,
                complexity AS "complexity!: i32",
                duration_seconds AS "duration_seconds: i32",
                error_message,
                retry_count AS "retry_count!: i32",
                max_retries AS "max_retries!: i32",
                started_at AS "started_at: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM team_tasks
            WHERE team_execution_id = $1 AND status = 'pending'
            ORDER BY sequence_order"#,
            team_execution_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_running_tasks(
        pool: &SqlitePool,
        team_execution_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            TeamTask,
            r#"SELECT 
                id AS "id!: Uuid",
                team_execution_id AS "team_execution_id!: Uuid",
                task_id AS "task_id!: Uuid",
                workspace_id AS "workspace_id: Uuid",
                sequence_order AS "sequence_order!: i32",
                depends_on,
                required_skills,
                assigned_agent_profile_id AS "assigned_agent_profile_id: Uuid",
                status AS "status!: TeamTaskStatus",
                branch_name,
                complexity AS "complexity!: i32",
                duration_seconds AS "duration_seconds: i32",
                error_message,
                retry_count AS "retry_count!: i32",
                max_retries AS "max_retries!: i32",
                started_at AS "started_at: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM team_tasks
            WHERE team_execution_id = $1 AND status IN ('running', 'assigned')
            ORDER BY sequence_order"#,
            team_execution_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, data: &CreateTeamTask) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        let depends_on = data
            .depends_on
            .as_ref()
            .map(|d| serde_json::to_string(d).unwrap());
        let required_skills = data
            .required_skills
            .as_ref()
            .map(|d| serde_json::to_string(d).unwrap());
        let complexity = data.complexity.unwrap_or(1);
        let max_retries = data.max_retries.unwrap_or(2);

        sqlx::query_as!(
            TeamTask,
            r#"INSERT INTO team_tasks 
                (id, team_execution_id, task_id, sequence_order, depends_on, required_skills, complexity, max_retries)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id AS "id!: Uuid",
                team_execution_id AS "team_execution_id!: Uuid",
                task_id AS "task_id!: Uuid",
                workspace_id AS "workspace_id: Uuid",
                sequence_order AS "sequence_order!: i32",
                depends_on,
                required_skills,
                assigned_agent_profile_id AS "assigned_agent_profile_id: Uuid",
                status AS "status!: TeamTaskStatus",
                branch_name,
                complexity AS "complexity!: i32",
                duration_seconds AS "duration_seconds: i32",
                error_message,
                retry_count AS "retry_count!: i32",
                max_retries AS "max_retries!: i32",
                started_at AS "started_at: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>""#,
            id,
            data.team_execution_id,
            data.task_id,
            data.sequence_order,
            depends_on,
            required_skills,
            complexity,
            max_retries
        )
        .fetch_one(pool)
        .await
    }

    pub async fn assign_agent(
        pool: &SqlitePool,
        id: Uuid,
        agent_profile_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE team_tasks SET assigned_agent_profile_id = $2, status = 'assigned', updated_at = datetime('now', 'subsec') WHERE id = $1",
            id,
            agent_profile_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn set_workspace(
        pool: &SqlitePool,
        id: Uuid,
        workspace_id: Uuid,
        branch_name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE team_tasks SET workspace_id = $2, branch_name = $3, updated_at = datetime('now', 'subsec') WHERE id = $1",
            id,
            workspace_id,
            branch_name
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn start(pool: &SqlitePool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE team_tasks SET status = 'running', started_at = datetime('now', 'subsec'), updated_at = datetime('now', 'subsec') WHERE id = $1",
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn complete(pool: &SqlitePool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE team_tasks SET 
                status = 'completed',
                completed_at = datetime('now', 'subsec'),
                updated_at = datetime('now', 'subsec')
            WHERE id = $1"#,
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn fail(pool: &SqlitePool, id: Uuid, error: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE team_tasks SET status = 'failed', error_message = $2, completed_at = datetime('now', 'subsec'), updated_at = datetime('now', 'subsec') WHERE id = $1",
            id,
            error
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn retry(pool: &SqlitePool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"UPDATE team_tasks SET status = 'pending', retry_count = retry_count + 1, error_message = NULL, started_at = NULL, completed_at = NULL, updated_at = datetime('now', 'subsec') WHERE id = $1 AND retry_count < max_retries"#,
            id
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn skip(pool: &SqlitePool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE team_tasks SET status = 'skipped', updated_at = datetime('now', 'subsec') WHERE id = $1",
            id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Check if all tasks in a team execution are completed
    pub async fn all_completed(
        pool: &SqlitePool,
        team_execution_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"SELECT COUNT(*) AS "count!: i64" FROM team_tasks 
               WHERE team_execution_id = $1 AND status NOT IN ('completed', 'skipped')"#,
            team_execution_id
        )
        .fetch_one(pool)
        .await?;
        Ok(result.count == 0)
    }

    pub async fn get_progress(
        pool: &SqlitePool,
        team_execution_id: Uuid,
    ) -> Result<TeamProgress, sqlx::Error> {
        let result = sqlx::query!(
            r#"SELECT 
                COUNT(*) AS "total!: i64",
                SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) AS "completed!: i64",
                SUM(CASE WHEN status = 'running' THEN 1 ELSE 0 END) AS "running!: i64",
                SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) AS "failed!: i64",
                SUM(CASE WHEN status = 'pending' OR status = 'assigned' THEN 1 ELSE 0 END) AS "pending!: i64",
                SUM(CASE WHEN status = 'skipped' THEN 1 ELSE 0 END) AS "skipped!: i64"
            FROM team_tasks
            WHERE team_execution_id = $1"#,
            team_execution_id
        )
        .fetch_one(pool)
        .await?;

        Ok(TeamProgress {
            total: result.total as i32,
            completed: result.completed as i32,
            running: result.running as i32,
            failed: result.failed as i32,
            pending: result.pending as i32,
            skipped: result.skipped as i32,
        })
    }

    pub fn get_dependencies(&self) -> Vec<Uuid> {
        self.depends_on
            .as_ref()
            .and_then(|d| serde_json::from_str(d).ok())
            .unwrap_or_default()
    }

    pub fn get_required_skills(&self) -> Vec<String> {
        self.required_skills
            .as_ref()
            .and_then(|d| serde_json::from_str(d).ok())
            .unwrap_or_default()
    }
}
