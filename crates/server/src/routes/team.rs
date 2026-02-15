//! Team Management API Routes
//!
//! Provides endpoints for managing agent teams, including:
//! - Creating and managing team executions
//! - Task planning and decomposition
//! - Agent skill and profile management

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use db::models::{
    agent_profile::{AgentProfile, CreateAgentProfile, UpdateAgentProfile},
    agent_skill::{AgentSkill, CreateAgentSkill, UpdateAgentSkill},
    team_execution::{TeamExecution, TeamPlanOutput},
    team_task::{TeamProgress, TeamTask},
    task::Task,
};
use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;
use ts_rs::TS;
use uuid::Uuid;

use crate::{DeploymentImpl, error::ApiError};

// ============== Request/Response Types ==============

#[derive(Debug, Deserialize, TS)]
pub struct CreateTeamExecutionRequest {
    pub epic_task_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub max_parallel_workers: Option<i32>,
}

#[derive(Debug, Serialize, TS)]
pub struct TeamExecutionResponse {
    pub execution: TeamExecution,
    pub tasks: Vec<TeamTask>,
    pub progress: TeamProgress,
}

#[derive(Debug, Serialize, TS)]
pub struct TeamPlanResponse {
    pub execution: TeamExecution,
    pub plan: TeamPlanOutput,
}

// ============== Routes ==============

pub fn router(deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        // Team Execution routes
        .route("/teams", post(create_team_execution))
        .route("/teams/{id}", get(get_team_execution))
        .route("/teams/{id}/plan", post(generate_plan))
        .route("/teams/{id}/execute", post(execute_plan))
        .route("/teams/{id}/progress", get(get_progress))
        .route("/teams/{id}/pause", post(pause_execution))
        .route("/teams/{id}/resume", post(resume_execution))
        .route("/teams/{id}/cancel", post(cancel_execution))
        // Team Tasks routes
        .route("/teams/{id}/tasks", get(get_team_tasks))
        .route("/teams/tasks/{task_id}/complete", post(complete_task))
        .route("/teams/tasks/{task_id}/fail", post(fail_task))
        // Agent Skills routes
        .route("/agent-skills", get(list_skills).post(create_skill))
        .route(
            "/agent-skills/{id}",
            get(get_skill).put(update_skill).delete(delete_skill),
        )
        // Agent Profiles routes
        .route("/agent-profiles", get(list_profiles).post(create_profile))
        .route(
            "/agent-profiles/{id}",
            get(get_profile).put(update_profile).delete(delete_profile),
        )
        .route("/agent-profiles/{id}/skills", get(get_profile_skills))
        .route(
            "/agent-profiles/{id}/skills/{skill_id}",
            post(add_profile_skill).delete(remove_profile_skill),
        )
        // Epic Tasks routes
        .route("/projects/{project_id}/epic-tasks", get(list_epic_tasks))
        .route("/tasks/{task_id}/set-epic", post(set_task_epic))
}

// ============== Team Execution Handlers ==============

async fn create_team_execution(
    State(deployment): State<DeploymentImpl>,
    Json(req): Json<CreateTeamExecutionRequest>,
) -> Result<Json<TeamExecution>, ApiError> {
    let pool = &deployment.db().pool;

    // Verify task exists and is epic
    let task = Task::find_by_id(pool, req.epic_task_id)
        .await?
        .ok_or_else(|| ApiError::Database(SqlxError::RowNotFound))?;

    if !task.is_epic {
        // Auto-set as epic
        Task::set_epic(pool, req.epic_task_id, true).await?;
    }

    let planner = services::services::team::PlannerService::new(pool.clone());
    let execution = planner
        .create_team_execution(req.epic_task_id, req.workspace_id, req.max_parallel_workers)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(execution))
}

async fn get_team_execution(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamExecutionResponse>, ApiError> {
    let pool = &deployment.db().pool;

    let execution = TeamExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::Database(SqlxError::RowNotFound))?;

    let tasks = TeamTask::find_by_team_execution(pool, id).await?;
    let progress = TeamTask::get_progress(pool, id).await?;

    Ok(Json(TeamExecutionResponse {
        execution,
        tasks,
        progress,
    }))
}

async fn generate_plan(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamPlanResponse>, ApiError> {
    let pool = &deployment.db().pool;
    let planner = services::services::team::PlannerService::new(pool.clone());

    let plan = planner
        .generate_plan(id)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let execution = TeamExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::Database(SqlxError::RowNotFound))?;

    Ok(Json(TeamPlanResponse { execution, plan }))
}

async fn execute_plan(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<TeamTask>>, ApiError> {
    let pool = &deployment.db().pool;

    let execution = TeamExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::Database(SqlxError::RowNotFound))?;

    let plan: TeamPlanOutput = execution
        .planner_output
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("No plan generated yet".into()))
        .and_then(|p| serde_json::from_str(p).map_err(|e| ApiError::BadRequest(e.to_string())))?;

    let planner = services::services::team::PlannerService::new(pool.clone());
    let tasks = planner
        .execute_plan(id, &plan)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    Ok(Json(tasks))
}

async fn get_progress(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamProgress>, ApiError> {
    let pool = &deployment.db().pool;
    let progress = TeamTask::get_progress(pool, id).await?;
    Ok(Json(progress))
}

async fn pause_execution(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamExecution>, ApiError> {
    let pool = &deployment.db().pool;
    let manager = services::services::team::TeamManager::new(pool.clone());

    manager
        .pause_execution(id)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let execution = TeamExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::Database(SqlxError::RowNotFound))?;

    Ok(Json(execution))
}

async fn resume_execution(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamExecution>, ApiError> {
    let pool = &deployment.db().pool;
    let manager = services::services::team::TeamManager::new(pool.clone());

    manager
        .resume_execution(id)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let execution = TeamExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::Database(SqlxError::RowNotFound))?;

    Ok(Json(execution))
}

async fn cancel_execution(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamExecution>, ApiError> {
    let pool = &deployment.db().pool;
    let manager = services::services::team::TeamManager::new(pool.clone());

    manager
        .cancel_execution(id)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let execution = TeamExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::Database(SqlxError::RowNotFound))?;

    Ok(Json(execution))
}

// ============== Team Tasks Handlers ==============

async fn get_team_tasks(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<TeamTask>>, ApiError> {
    let pool = &deployment.db().pool;
    let tasks = TeamTask::find_by_team_execution(pool, id).await?;
    Ok(Json(tasks))
}

async fn complete_task(
    State(deployment): State<DeploymentImpl>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TeamTask>, ApiError> {
    let pool = &deployment.db().pool;
    let manager = services::services::team::TeamManager::new(pool.clone());

    manager
        .complete_task(task_id)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let task = TeamTask::find_by_id(pool, task_id)
        .await?
        .ok_or_else(|| ApiError::Database(SqlxError::RowNotFound))?;

    Ok(Json(task))
}

#[derive(Debug, Deserialize)]
pub struct FailTaskRequest {
    pub error: String,
}

async fn fail_task(
    State(deployment): State<DeploymentImpl>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<FailTaskRequest>,
) -> Result<Json<TeamTask>, ApiError> {
    let pool = &deployment.db().pool;
    let manager = services::services::team::TeamManager::new(pool.clone());

    manager
        .fail_task(task_id, &req.error)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let task = TeamTask::find_by_id(pool, task_id)
        .await?
        .ok_or_else(|| ApiError::Database(SqlxError::RowNotFound))?;

    Ok(Json(task))
}

// ============== Agent Skills Handlers ==============

async fn list_skills(
    State(deployment): State<DeploymentImpl>,
) -> Result<Json<Vec<AgentSkill>>, ApiError> {
    let pool = &deployment.db().pool;
    let skills = AgentSkill::find_all(pool).await?;
    Ok(Json(skills))
}

async fn get_skill(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentSkill>, ApiError> {
    let pool = &deployment.db().pool;
    let skill = AgentSkill::find_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::Database(SqlxError::RowNotFound))?;
    Ok(Json(skill))
}

async fn create_skill(
    State(deployment): State<DeploymentImpl>,
    Json(req): Json<CreateAgentSkill>,
) -> Result<Json<AgentSkill>, ApiError> {
    let pool = &deployment.db().pool;
    let skill = AgentSkill::create(pool, &req).await?;
    Ok(Json(skill))
}

async fn update_skill(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAgentSkill>,
) -> Result<Json<AgentSkill>, ApiError> {
    let pool = &deployment.db().pool;
    let skill = AgentSkill::update(pool, id, &req).await?;
    Ok(Json(skill))
}

async fn delete_skill(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    let pool = &deployment.db().pool;
    AgentSkill::delete(pool, id).await?;
    Ok(Json(true))
}

// ============== Agent Profiles Handlers ==============

async fn list_profiles(
    State(deployment): State<DeploymentImpl>,
) -> Result<Json<Vec<AgentProfile>>, ApiError> {
    let pool = &deployment.db().pool;
    let profiles = AgentProfile::find_all(pool).await?;
    Ok(Json(profiles))
}

async fn get_profile(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentProfile>, ApiError> {
    let pool = &deployment.db().pool;
    let profile = AgentProfile::find_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::Database(SqlxError::RowNotFound))?;
    Ok(Json(profile))
}

async fn create_profile(
    State(deployment): State<DeploymentImpl>,
    Json(req): Json<CreateAgentProfile>,
) -> Result<Json<AgentProfile>, ApiError> {
    let pool = &deployment.db().pool;
    let profile = AgentProfile::create(pool, &req).await?;
    Ok(Json(profile))
}

async fn update_profile(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAgentProfile>,
) -> Result<Json<AgentProfile>, ApiError> {
    let pool = &deployment.db().pool;
    let profile = AgentProfile::update(pool, id, &req).await?;
    Ok(Json(profile))
}

async fn delete_profile(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, ApiError> {
    let pool = &deployment.db().pool;
    AgentProfile::delete(pool, id).await?;
    Ok(Json(true))
}

async fn get_profile_skills(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentSkill>>, ApiError> {
    let pool = &deployment.db().pool;
    let skills = AgentProfile::get_skills(pool, id).await?;
    Ok(Json(skills))
}

#[derive(Debug, Deserialize)]
pub struct AddSkillRequest {
    pub proficiency: Option<i32>,
}

async fn add_profile_skill(
    State(deployment): State<DeploymentImpl>,
    Path((id, skill_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<AddSkillRequest>,
) -> Result<Json<bool>, ApiError> {
    let pool = &deployment.db().pool;
    let proficiency = req.proficiency.unwrap_or(3);
    AgentProfile::add_skill(pool, id, skill_id, proficiency).await?;
    Ok(Json(true))
}

async fn remove_profile_skill(
    State(deployment): State<DeploymentImpl>,
    Path((id, skill_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<bool>, ApiError> {
    let pool = &deployment.db().pool;
    AgentProfile::remove_skill(pool, id, skill_id).await?;
    Ok(Json(true))
}

// ============== Epic Tasks Handlers ==============

async fn list_epic_tasks(
    State(deployment): State<DeploymentImpl>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Task>>, ApiError> {
    let pool = &deployment.db().pool;
    let tasks = Task::find_epic_tasks(pool, project_id).await?;
    Ok(Json(tasks))
}

#[derive(Debug, Deserialize)]
pub struct SetEpicRequest {
    pub is_epic: bool,
}

async fn set_task_epic(
    State(deployment): State<DeploymentImpl>,
    Path(task_id): Path<Uuid>,
    Json(req): Json<SetEpicRequest>,
) -> Result<Json<Task>, ApiError> {
    let pool = &deployment.db().pool;
    Task::set_epic(pool, task_id, req.is_epic).await?;

    let task = Task::find_by_id(pool, task_id)
        .await?
        .ok_or_else(|| ApiError::Database(SqlxError::RowNotFound))?;

    Ok(Json(task))
}
