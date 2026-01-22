use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;
use uuid::Uuid;

use super::agent_skill::AgentSkill;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct AgentProfile {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub executor: String,
    pub variant: Option<String>,
    pub executor_config: Option<String>,
    pub is_planner: bool,
    pub is_reviewer: bool,
    pub is_worker: bool,
    pub max_concurrent_tasks: i32,
    pub priority: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct AgentProfileWithSkills {
    #[serde(flatten)]
    #[ts(flatten)]
    pub profile: AgentProfile,
    pub skills: Vec<AgentProfileSkill>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct AgentProfileSkill {
    pub agent_profile_id: Uuid,
    pub agent_skill_id: Uuid,
    pub proficiency: i32,
    // Flattened skill info
    pub skill_name: Option<String>,
    pub skill_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct CreateAgentProfile {
    pub name: String,
    pub description: Option<String>,
    pub executor: String,
    pub variant: Option<String>,
    pub executor_config: Option<String>,
    pub is_planner: Option<bool>,
    pub is_reviewer: Option<bool>,
    pub is_worker: Option<bool>,
    pub max_concurrent_tasks: Option<i32>,
    pub priority: Option<i32>,
    pub skills: Option<Vec<SkillAssignment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SkillAssignment {
    pub skill_id: Uuid,
    pub proficiency: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct UpdateAgentProfile {
    pub name: Option<String>,
    pub description: Option<String>,
    pub executor: Option<String>,
    pub variant: Option<String>,
    pub executor_config: Option<String>,
    pub is_planner: Option<bool>,
    pub is_reviewer: Option<bool>,
    pub is_worker: Option<bool>,
    pub max_concurrent_tasks: Option<i32>,
    pub priority: Option<i32>,
    pub active: Option<bool>,
}

impl AgentProfile {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            AgentProfile,
            r#"SELECT 
                id AS "id!: Uuid",
                name,
                description,
                executor,
                variant,
                executor_config,
                is_planner AS "is_planner!: bool",
                is_reviewer AS "is_reviewer!: bool",
                is_worker AS "is_worker!: bool",
                max_concurrent_tasks AS "max_concurrent_tasks!: i32",
                priority AS "priority!: i32",
                active AS "active!: bool",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM agent_profiles
            WHERE active = 1
            ORDER BY priority DESC, name"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            AgentProfile,
            r#"SELECT 
                id AS "id!: Uuid",
                name,
                description,
                executor,
                variant,
                executor_config,
                is_planner AS "is_planner!: bool",
                is_reviewer AS "is_reviewer!: bool",
                is_worker AS "is_worker!: bool",
                max_concurrent_tasks AS "max_concurrent_tasks!: i32",
                priority AS "priority!: i32",
                active AS "active!: bool",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM agent_profiles
            WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_planners(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            AgentProfile,
            r#"SELECT 
                id AS "id!: Uuid",
                name,
                description,
                executor,
                variant,
                executor_config,
                is_planner AS "is_planner!: bool",
                is_reviewer AS "is_reviewer!: bool",
                is_worker AS "is_worker!: bool",
                max_concurrent_tasks AS "max_concurrent_tasks!: i32",
                priority AS "priority!: i32",
                active AS "active!: bool",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM agent_profiles
            WHERE is_planner = 1 AND active = 1
            ORDER BY priority DESC, name"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_reviewers(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            AgentProfile,
            r#"SELECT 
                id AS "id!: Uuid",
                name,
                description,
                executor,
                variant,
                executor_config,
                is_planner AS "is_planner!: bool",
                is_reviewer AS "is_reviewer!: bool",
                is_worker AS "is_worker!: bool",
                max_concurrent_tasks AS "max_concurrent_tasks!: i32",
                priority AS "priority!: i32",
                active AS "active!: bool",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM agent_profiles
            WHERE is_reviewer = 1 AND active = 1
            ORDER BY priority DESC, name"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_workers(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            AgentProfile,
            r#"SELECT 
                id AS "id!: Uuid",
                name,
                description,
                executor,
                variant,
                executor_config,
                is_planner AS "is_planner!: bool",
                is_reviewer AS "is_reviewer!: bool",
                is_worker AS "is_worker!: bool",
                max_concurrent_tasks AS "max_concurrent_tasks!: i32",
                priority AS "priority!: i32",
                active AS "active!: bool",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM agent_profiles
            WHERE is_worker = 1 AND active = 1
            ORDER BY priority DESC, name"#
        )
        .fetch_all(pool)
        .await
    }

    /// Find worker profiles that have the required skills
    pub async fn find_workers_with_skills(
        pool: &SqlitePool,
        skill_ids: &[Uuid],
    ) -> Result<Vec<Self>, sqlx::Error> {
        if skill_ids.is_empty() {
            return Self::find_workers(pool).await;
        }

        // Build the query with skill matching
        let skill_ids_str: Vec<String> = skill_ids.iter().map(|id| format!("'{}'", id)).collect();
        let skill_list = skill_ids_str.join(",");

        let query = format!(
            r#"SELECT DISTINCT
                ap.id,
                ap.name,
                ap.description,
                ap.executor,
                ap.variant,
                ap.executor_config,
                ap.is_planner,
                ap.is_reviewer,
                ap.is_worker,
                ap.max_concurrent_tasks,
                ap.priority,
                ap.active,
                ap.created_at,
                ap.updated_at
            FROM agent_profiles ap
            INNER JOIN agent_profile_skills aps ON ap.id = aps.agent_profile_id
            WHERE ap.is_worker = 1 
              AND ap.active = 1
              AND aps.agent_skill_id IN ({})
            ORDER BY ap.priority DESC, ap.name"#,
            skill_list
        );

        sqlx::query_as::<_, AgentProfile>(&query)
            .fetch_all(pool)
            .await
    }

    pub async fn create(pool: &SqlitePool, data: &CreateAgentProfile) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        let is_planner = data.is_planner.unwrap_or(false);
        let is_reviewer = data.is_reviewer.unwrap_or(false);
        let is_worker = data.is_worker.unwrap_or(true);
        let max_concurrent = data.max_concurrent_tasks.unwrap_or(1);
        let priority = data.priority.unwrap_or(0);

        let profile = sqlx::query_as!(
            AgentProfile,
            r#"INSERT INTO agent_profiles 
                (id, name, description, executor, variant, executor_config, 
                 is_planner, is_reviewer, is_worker, max_concurrent_tasks, priority)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING 
                id AS "id!: Uuid",
                name,
                description,
                executor,
                variant,
                executor_config,
                is_planner AS "is_planner!: bool",
                is_reviewer AS "is_reviewer!: bool",
                is_worker AS "is_worker!: bool",
                max_concurrent_tasks AS "max_concurrent_tasks!: i32",
                priority AS "priority!: i32",
                active AS "active!: bool",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>""#,
            id,
            data.name,
            data.description,
            data.executor,
            data.variant,
            data.executor_config,
            is_planner,
            is_reviewer,
            is_worker,
            max_concurrent,
            priority
        )
        .fetch_one(pool)
        .await?;

        // Add skills if provided
        if let Some(skills) = &data.skills {
            for skill in skills {
                let proficiency = skill.proficiency.unwrap_or(3);
                sqlx::query!(
                    "INSERT INTO agent_profile_skills (agent_profile_id, agent_skill_id, proficiency)
                     VALUES ($1, $2, $3)",
                    id,
                    skill.skill_id,
                    proficiency
                )
                .execute(pool)
                .await?;
            }
        }

        Ok(profile)
    }

    pub async fn get_skills(pool: &SqlitePool, profile_id: Uuid) -> Result<Vec<AgentSkill>, sqlx::Error> {
        sqlx::query_as!(
            AgentSkill,
            r#"SELECT 
                s.id AS "id!: Uuid",
                s.name,
                s.description,
                s.prompt_modifier,
                s.category,
                s.icon,
                s.created_at AS "created_at!: DateTime<Utc>",
                s.updated_at AS "updated_at!: DateTime<Utc>"
            FROM agent_skills s
            INNER JOIN agent_profile_skills aps ON s.id = aps.agent_skill_id
            WHERE aps.agent_profile_id = $1
            ORDER BY aps.proficiency DESC, s.name"#,
            profile_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn add_skill(
        pool: &SqlitePool,
        profile_id: Uuid,
        skill_id: Uuid,
        proficiency: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT OR REPLACE INTO agent_profile_skills (agent_profile_id, agent_skill_id, proficiency)
             VALUES ($1, $2, $3)",
            profile_id,
            skill_id,
            proficiency
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn remove_skill(
        pool: &SqlitePool,
        profile_id: Uuid,
        skill_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM agent_profile_skills WHERE agent_profile_id = $1 AND agent_skill_id = $2",
            profile_id,
            skill_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn update(
        pool: &SqlitePool,
        id: Uuid,
        data: &UpdateAgentProfile,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            AgentProfile,
            r#"UPDATE agent_profiles SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                executor = COALESCE($4, executor),
                variant = COALESCE($5, variant),
                executor_config = COALESCE($6, executor_config),
                is_planner = COALESCE($7, is_planner),
                is_reviewer = COALESCE($8, is_reviewer),
                is_worker = COALESCE($9, is_worker),
                max_concurrent_tasks = COALESCE($10, max_concurrent_tasks),
                priority = COALESCE($11, priority),
                active = COALESCE($12, active),
                updated_at = datetime('now', 'subsec')
            WHERE id = $1
            RETURNING 
                id AS "id!: Uuid",
                name,
                description,
                executor,
                variant,
                executor_config,
                is_planner AS "is_planner!: bool",
                is_reviewer AS "is_reviewer!: bool",
                is_worker AS "is_worker!: bool",
                max_concurrent_tasks AS "max_concurrent_tasks!: i32",
                priority AS "priority!: i32",
                active AS "active!: bool",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>""#,
            id,
            data.name,
            data.description,
            data.executor,
            data.variant,
            data.executor_config,
            data.is_planner,
            data.is_reviewer,
            data.is_worker,
            data.max_concurrent_tasks,
            data.priority,
            data.active
        )
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM agent_profiles WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
