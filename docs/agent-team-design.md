# Agent Team Feature Design

## Overview

The Agent Team feature enables parallel task execution through a coordinated multi-agent system. Users can define "epic tasks" that are automatically decomposed into atomic subtasks by a Team Manager agent, executed in parallel by multiple worker agents, and tracked as a single team execution.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              EPIC TASK                                       │
│                     (User-defined complex task)                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          TEAM MANAGER AGENT                                  │
│  • Analyzes epic task complexity                                            │
│  • Decomposes into atomic subtasks with dependencies                        │
│  • Assigns required skills to each subtask                                  │
│  • Decides parallelism strategy                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
            ┌───────────┐   ┌───────────┐   ┌───────────┐
            │  Subtask  │   │  Subtask  │   │  Subtask  │
            │     1     │   │     2     │   │     N     │
            │ (branch1) │   │ (branch2) │   │ (branchN) │
            └───────────┘   └───────────┘   └───────────┘
                    │               │               │
                    ▼               ▼               ▼
            ┌───────────┐   ┌───────────┐   ┌───────────┐
            │  Worker   │   │  Worker   │   │  Worker   │
            │  Agent 1  │   │  Agent 2  │   │  Agent N  │
            │ (skills)  │   │ (skills)  │   │ (skills)  │
            └───────────┘   └───────────┘   └───────────┘
```

## Core Concepts

### 1. Epic Task
An epic task is a complex, high-level task that requires decomposition. It extends the existing Task model with:
- Complexity estimation
- Decomposition strategy
- Team execution tracking

### 2. Agent Skills
Skills define what an agent can do. Examples:
- `frontend`: React, Vue, CSS, HTML expertise
- `backend`: API design, database, server logic
- `testing`: Unit tests, integration tests, E2E
- `documentation`: README, API docs, inline comments
- `refactoring`: Code optimization, cleanup
- `security`: Security audits, vulnerability fixes
- `devops`: CI/CD, deployment, infrastructure

### 3. Team Manager Agent
A specialized agent that:
1. Analyzes the epic task description
2. Estimates complexity (simple, moderate, complex)
3. Decomposes into atomic subtasks with:
   - Clear boundaries
   - Dependency ordering
   - Required skills
   - Estimated effort

### 4. Team Execution
Orchestrates parallel execution:
- Creates sub-branches from epic branch
- Assigns agents to subtasks based on skills
- Monitors progress and handles failures
- Manages resource allocation

## Data Models

### TeamExecution
```rust
pub struct TeamExecution {
    pub id: Uuid,
    pub epic_task_id: Uuid,
    pub status: TeamExecutionStatus,  // planning, executing, completed, failed, cancelled
    pub planner_output: Option<String>, // JSON of decomposition plan
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

### TeamTask
```rust
pub struct TeamTask {
    pub id: Uuid,
    pub team_execution_id: Uuid,
    pub task_id: Uuid,                  // Reference to actual Task
    pub workspace_id: Option<Uuid>,     // Workspace for this subtask
    pub sequence_order: i32,            // Execution order
    pub depends_on: Option<Vec<Uuid>>,  // Task dependencies
    pub required_skills: Vec<String>,   // Skills needed
    pub status: TeamTaskStatus,         // pending, running, completed, failed
    pub assigned_agent: Option<String>, // Which agent is working on this
}
```

### AgentSkill
```rust
pub struct AgentSkill {
    pub id: Uuid,
    pub name: String,           // e.g., "frontend", "backend"
    pub description: String,
    pub prompt_modifier: String, // Added to agent prompts
}
```

### AgentProfile  
```rust
pub struct AgentProfile {
    pub id: Uuid,
    pub name: String,
    pub executor: BaseCodingAgent,
    pub skills: Vec<Uuid>,      // References to AgentSkill
    pub is_planner: bool,       // Can this agent plan?
    pub max_concurrent_tasks: i32,
}
```

## API Endpoints

### Epic Task Management
- `POST /api/projects/{id}/epic-tasks` - Create epic task
- `GET /api/projects/{id}/epic-tasks` - List epic tasks
- `POST /api/teams` - Create team execution
- `POST /api/teams/{id}/plan` - Trigger team manager planning
- `POST /api/teams/{id}/execute` - Start team execution

### Team Management
- `GET /api/teams/{id}` - Get team execution status
- `GET /api/teams/{id}/tasks` - List subtasks
- `POST /api/teams/{id}/pause` - Pause execution
- `POST /api/teams/{id}/resume` - Resume execution
- `POST /api/teams/{id}/cancel` - Cancel execution

### Agent Skills
- `GET /api/agent-skills` - List available skills
- `POST /api/agent-skills` - Create skill
- `GET /api/agent-profiles` - List agent profiles
- `POST /api/agent-profiles` - Create agent profile

## Workflow

1. **Task Creation**: User creates an epic task with description
2. **Planning Phase**:
   - Team manager agent analyzes the task
   - Generates decomposition plan
   - Creates subtasks with dependencies
3. **Execution Phase**:
   - Sub-branches created from epic branch
   - Worker agents assigned based on skills
   - Parallel execution with dependency ordering
4. **Completion**:
   - All subtasks complete
   - Team execution marked complete

## Configuration

```toml
[team]
max_parallel_agents = 5
planner_model = "claude-3-opus"
default_worker_model = "claude-3-sonnet"
```

## Future Enhancements

1. **Learning System**: Track successful decompositions to improve planning
2. **Dynamic Scaling**: Auto-adjust team size based on task complexity
3. **Cross-Project Agents**: Share agent pools across projects
4. **Human-in-the-Loop**: Allow human intervention at any phase
5. **Cost Optimization**: Balance quality vs. API costs
