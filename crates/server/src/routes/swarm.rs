//! Swarm Management API Routes
//!
//! Provides endpoints for managing agent swarm executions, including:
//! - Creating and managing swarm executions
//! - Task planning and decomposition
//! - Consensus review management
//! - Agent skill and profile management

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use db::models::{
    agent_profile::{AgentProfile, CreateAgentProfile, UpdateAgentProfile},
    agent_skill::{AgentSkill, CreateAgentSkill, UpdateAgentSkill},
    consensus_review::{ConsensusReview, ConsensusSummary, SubmitReviewVote},
    swarm_execution::{SwarmExecution, SwarmPlanOutput},
    swarm_task::{SwarmProgress, SwarmTask},
    task::Task,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::{DeploymentImpl, error::AppError};

// ============== Request/Response Types ==============

#[derive(Debug, Deserialize, TS)]
pub struct CreateSwarmExecutionRequest {
    pub epic_task_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub reviewer_count: Option<i32>,
    pub max_parallel_workers: Option<i32>,
}

#[derive(Debug, Serialize, TS)]
pub struct SwarmExecutionResponse {
    pub execution: SwarmExecution,
    pub tasks: Vec<SwarmTask>,
    pub progress: SwarmProgress,
}

#[derive(Debug, Serialize, TS)]
pub struct SwarmPlanResponse {
    pub execution: SwarmExecution,
    pub plan: SwarmPlanOutput,
}

#[derive(Debug, Deserialize, TS)]
pub struct ExecutePlanRequest {
    pub swarm_execution_id: Uuid,
}

#[derive(Debug, Serialize, TS)]
pub struct ConsensusStatusResponse {
    pub execution: SwarmExecution,
    pub reviews: Vec<ConsensusReview>,
    pub summary: ConsensusSummary,
}

#[derive(Debug, Deserialize, TS)]
pub struct SubmitVoteRequest {
    pub review_id: Uuid,
    pub vote: SubmitReviewVote,
}

// ============== Routes ==============

pub fn router(deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        // Swarm Execution routes
        .route("/swarms", post(create_swarm_execution))
        .route("/swarms/{id}", get(get_swarm_execution))
        .route("/swarms/{id}/plan", post(generate_plan))
        .route("/swarms/{id}/execute", post(execute_plan))
        .route("/swarms/{id}/progress", get(get_progress))
        .route("/swarms/{id}/pause", post(pause_execution))
        .route("/swarms/{id}/resume", post(resume_execution))
        .route("/swarms/{id}/cancel", post(cancel_execution))
        // Swarm Tasks routes
        .route("/swarms/{id}/tasks", get(get_swarm_tasks))
        .route("/swarms/tasks/{task_id}/complete", post(complete_task))
        .route("/swarms/tasks/{task_id}/fail", post(fail_task))
        // Consensus routes
        .route("/swarms/{id}/consensus", get(get_consensus_status))
        .route("/swarms/{id}/consensus/start", post(start_consensus))
        .route("/swarms/{id}/consensus/vote", post(submit_vote))
        .route("/swarms/{id}/consensus/finalize", post(finalize_consensus))
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

// ============== Swarm Execution Handlers ==============

async fn create_swarm_execution(
    State(deployment): State<DeploymentImpl>,
    Json(req): Json<CreateSwarmExecutionRequest>,
) -> Result<Json<SwarmExecution>, AppError> {
    let pool = deployment.pool();
    
    // Verify task exists and is epic
    let task = Task::find_by_id(pool, req.epic_task_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".into()))?;

    if !task.is_epic {
        // Auto-set as epic
        Task::set_epic(pool, req.epic_task_id, true).await?;
    }

    let planner = services::services::swarm::PlannerService::new(pool.clone());
    let execution = planner
        .create_swarm_execution(req.epic_task_id, req.workspace_id)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    Ok(Json(execution))
}

async fn get_swarm_execution(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<SwarmExecutionResponse>, AppError> {
    let pool = deployment.pool();

    let execution = SwarmExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Swarm execution not found".into()))?;

    let tasks = SwarmTask::find_by_swarm_execution(pool, id).await?;
    let progress = SwarmTask::get_progress(pool, id).await?;

    Ok(Json(SwarmExecutionResponse {
        execution,
        tasks,
        progress,
    }))
}

async fn generate_plan(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<SwarmPlanResponse>, AppError> {
    let pool = deployment.pool();
    let planner = services::services::swarm::PlannerService::new(pool.clone());

    let plan = planner
        .generate_plan(id)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let execution = SwarmExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Swarm execution not found".into()))?;

    Ok(Json(SwarmPlanResponse { execution, plan }))
}

async fn execute_plan(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<SwarmTask>>, AppError> {
    let pool = deployment.pool();

    let execution = SwarmExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Swarm execution not found".into()))?;

    let plan: SwarmPlanOutput = execution
        .planner_output
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("No plan generated yet".into()))
        .and_then(|p| {
            serde_json::from_str(p).map_err(|e| AppError::BadRequest(e.to_string()))
        })?;

    let planner = services::services::swarm::PlannerService::new(pool.clone());
    let tasks = planner
        .execute_plan(id, &plan)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    Ok(Json(tasks))
}

async fn get_progress(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<SwarmProgress>, AppError> {
    let pool = deployment.pool();
    let progress = SwarmTask::get_progress(pool, id).await?;
    Ok(Json(progress))
}

async fn pause_execution(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<SwarmExecution>, AppError> {
    let pool = deployment.pool();
    let manager = services::services::swarm::SwarmManager::new(pool.clone());

    manager
        .pause_execution(id)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let execution = SwarmExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Swarm execution not found".into()))?;

    Ok(Json(execution))
}

async fn resume_execution(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<SwarmExecution>, AppError> {
    let pool = deployment.pool();
    let manager = services::services::swarm::SwarmManager::new(pool.clone());

    manager
        .resume_execution(id)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let execution = SwarmExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Swarm execution not found".into()))?;

    Ok(Json(execution))
}

async fn cancel_execution(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<SwarmExecution>, AppError> {
    let pool = deployment.pool();
    let manager = services::services::swarm::SwarmManager::new(pool.clone());

    manager
        .cancel_execution(id)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let execution = SwarmExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Swarm execution not found".into()))?;

    Ok(Json(execution))
}

// ============== Swarm Tasks Handlers ==============

async fn get_swarm_tasks(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<SwarmTask>>, AppError> {
    let pool = deployment.pool();
    let tasks = SwarmTask::find_by_swarm_execution(pool, id).await?;
    Ok(Json(tasks))
}

async fn complete_task(
    State(deployment): State<DeploymentImpl>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<SwarmTask>, AppError> {
    let pool = deployment.pool();
    let manager = services::services::swarm::SwarmManager::new(pool.clone());

    manager
        .complete_task(task_id)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let task = SwarmTask::find_by_id(pool, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Swarm task not found".into()))?;

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
) -> Result<Json<SwarmTask>, AppError> {
    let pool = deployment.pool();
    let manager = services::services::swarm::SwarmManager::new(pool.clone());

    manager
        .fail_task(task_id, &req.error)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let task = SwarmTask::find_by_id(pool, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Swarm task not found".into()))?;

    Ok(Json(task))
}

// ============== Consensus Handlers ==============

async fn get_consensus_status(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConsensusStatusResponse>, AppError> {
    let pool = deployment.pool();
    let consensus = services::services::swarm::ConsensusService::new(pool.clone());

    let execution = SwarmExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Swarm execution not found".into()))?;

    let reviews = consensus
        .get_reviews(id)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let summary = consensus
        .get_summary(id)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    Ok(Json(ConsensusStatusResponse {
        execution,
        reviews,
        summary,
    }))
}

async fn start_consensus(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ConsensusReview>>, AppError> {
    let pool = deployment.pool();
    let consensus = services::services::swarm::ConsensusService::new(pool.clone());

    let reviews = consensus
        .start_review(id)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    Ok(Json(reviews))
}

async fn submit_vote(
    State(deployment): State<DeploymentImpl>,
    Path(_id): Path<Uuid>,
    Json(req): Json<SubmitVoteRequest>,
) -> Result<Json<ConsensusReview>, AppError> {
    let pool = deployment.pool();
    let consensus = services::services::swarm::ConsensusService::new(pool.clone());

    let review = consensus
        .submit_vote(req.review_id, &req.vote)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    Ok(Json(review))
}

async fn finalize_consensus(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<SwarmExecution>, AppError> {
    let pool = deployment.pool();
    let consensus = services::services::swarm::ConsensusService::new(pool.clone());

    consensus
        .finalize_consensus(id)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let execution = SwarmExecution::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Swarm execution not found".into()))?;

    Ok(Json(execution))
}

// ============== Agent Skills Handlers ==============

async fn list_skills(
    State(deployment): State<DeploymentImpl>,
) -> Result<Json<Vec<AgentSkill>>, AppError> {
    let pool = deployment.pool();
    let skills = AgentSkill::find_all(pool).await?;
    Ok(Json(skills))
}

async fn get_skill(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentSkill>, AppError> {
    let pool = deployment.pool();
    let skill = AgentSkill::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Skill not found".into()))?;
    Ok(Json(skill))
}

async fn create_skill(
    State(deployment): State<DeploymentImpl>,
    Json(req): Json<CreateAgentSkill>,
) -> Result<Json<AgentSkill>, AppError> {
    let pool = deployment.pool();
    let skill = AgentSkill::create(pool, &req).await?;
    Ok(Json(skill))
}

async fn update_skill(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAgentSkill>,
) -> Result<Json<AgentSkill>, AppError> {
    let pool = deployment.pool();
    let skill = AgentSkill::update(pool, id, &req).await?;
    Ok(Json(skill))
}

async fn delete_skill(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, AppError> {
    let pool = deployment.pool();
    AgentSkill::delete(pool, id).await?;
    Ok(Json(true))
}

// ============== Agent Profiles Handlers ==============

async fn list_profiles(
    State(deployment): State<DeploymentImpl>,
) -> Result<Json<Vec<AgentProfile>>, AppError> {
    let pool = deployment.pool();
    let profiles = AgentProfile::find_all(pool).await?;
    Ok(Json(profiles))
}

async fn get_profile(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentProfile>, AppError> {
    let pool = deployment.pool();
    let profile = AgentProfile::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Profile not found".into()))?;
    Ok(Json(profile))
}

async fn create_profile(
    State(deployment): State<DeploymentImpl>,
    Json(req): Json<CreateAgentProfile>,
) -> Result<Json<AgentProfile>, AppError> {
    let pool = deployment.pool();
    let profile = AgentProfile::create(pool, &req).await?;
    Ok(Json(profile))
}

async fn update_profile(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAgentProfile>,
) -> Result<Json<AgentProfile>, AppError> {
    let pool = deployment.pool();
    let profile = AgentProfile::update(pool, id, &req).await?;
    Ok(Json(profile))
}

async fn delete_profile(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, AppError> {
    let pool = deployment.pool();
    AgentProfile::delete(pool, id).await?;
    Ok(Json(true))
}

async fn get_profile_skills(
    State(deployment): State<DeploymentImpl>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AgentSkill>>, AppError> {
    let pool = deployment.pool();
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
) -> Result<Json<bool>, AppError> {
    let pool = deployment.pool();
    let proficiency = req.proficiency.unwrap_or(3);
    AgentProfile::add_skill(pool, id, skill_id, proficiency).await?;
    Ok(Json(true))
}

async fn remove_profile_skill(
    State(deployment): State<DeploymentImpl>,
    Path((id, skill_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<bool>, AppError> {
    let pool = deployment.pool();
    AgentProfile::remove_skill(pool, id, skill_id).await?;
    Ok(Json(true))
}

// ============== Epic Tasks Handlers ==============

async fn list_epic_tasks(
    State(deployment): State<DeploymentImpl>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<Task>>, AppError> {
    let pool = deployment.pool();
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
) -> Result<Json<Task>, AppError> {
    let pool = deployment.pool();
    Task::set_epic(pool, task_id, req.is_epic).await?;
    
    let task = Task::find_by_id(pool, task_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".into()))?;

    Ok(Json(task))
}
