use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct AgentSkill {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub prompt_modifier: Option<String>,
    pub category: String,
    pub icon: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct CreateAgentSkill {
    pub name: String,
    pub description: String,
    pub prompt_modifier: Option<String>,
    pub category: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct UpdateAgentSkill {
    pub name: Option<String>,
    pub description: Option<String>,
    pub prompt_modifier: Option<String>,
    pub category: Option<String>,
    pub icon: Option<String>,
}

impl AgentSkill {
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            AgentSkill,
            r#"SELECT 
                id AS "id!: Uuid",
                name,
                description,
                prompt_modifier,
                category,
                icon,
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM agent_skills
            ORDER BY category, name"#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            AgentSkill,
            r#"SELECT 
                id AS "id!: Uuid",
                name,
                description,
                prompt_modifier,
                category,
                icon,
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM agent_skills
            WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_name(pool: &SqlitePool, name: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            AgentSkill,
            r#"SELECT 
                id AS "id!: Uuid",
                name,
                description,
                prompt_modifier,
                category,
                icon,
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM agent_skills
            WHERE name = $1"#,
            name
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_category(
        pool: &SqlitePool,
        category: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            AgentSkill,
            r#"SELECT 
                id AS "id!: Uuid",
                name,
                description,
                prompt_modifier,
                category,
                icon,
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM agent_skills
            WHERE category = $1
            ORDER BY name"#,
            category
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, data: &CreateAgentSkill) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        let category = data.category.as_deref().unwrap_or("general");

        sqlx::query_as!(
            AgentSkill,
            r#"INSERT INTO agent_skills (id, name, description, prompt_modifier, category, icon)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING 
                id AS "id!: Uuid",
                name,
                description,
                prompt_modifier,
                category,
                icon,
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>""#,
            id,
            data.name,
            data.description,
            data.prompt_modifier,
            category,
            data.icon
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &SqlitePool,
        id: Uuid,
        data: &UpdateAgentSkill,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            AgentSkill,
            r#"UPDATE agent_skills SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                prompt_modifier = COALESCE($4, prompt_modifier),
                category = COALESCE($5, category),
                icon = COALESCE($6, icon),
                updated_at = datetime('now', 'subsec')
            WHERE id = $1
            RETURNING 
                id AS "id!: Uuid",
                name,
                description,
                prompt_modifier,
                category,
                icon,
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>""#,
            id,
            data.name,
            data.description,
            data.prompt_modifier,
            data.category,
            data.icon
        )
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM agent_skills WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}
