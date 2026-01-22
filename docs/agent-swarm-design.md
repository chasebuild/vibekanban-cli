# Agent Swarm Feature Design

## Overview

The Agent Swarm feature enables parallel task execution through a coordinated multi-agent system. Users can define "epic tasks" that are automatically decomposed into atomic subtasks by a Planner Agent, executed in parallel by multiple worker agents, and merged back using a pBFT-inspired consensus review mechanism.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              EPIC TASK                                       │
│                     (User-defined complex task)                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PLANNER AGENT                                      │
│  • Analyzes epic task complexity                                            │
│  • Determines workforce size (single agent vs swarm)                        │
│  • Decomposes into atomic subtasks with dependencies                        │
│  • Assigns required skills to each subtask                                  │
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
                    │               │               │
                    └───────────────┼───────────────┘
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       CONSENSUS REVIEW (pBFT)                                │
│  • Collect reviews from multiple reviewer agents                            │
│  • Require 2f+1 approvals (f = max faulty reviewers)                       │
│  • Detect conflicts and flag for human intervention                         │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         MERGE COORDINATOR                                    │
│  • Merge all subtask branches to epic branch                                │
│  • Handle merge conflicts                                                    │
│  • Create final consolidated commit                                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Core Concepts

### 1. Epic Task
An epic task is a complex, high-level task that requires decomposition. It extends the existing Task model with:
- Complexity estimation
- Decomposition strategy
- Swarm execution tracking

### 2. Agent Skills
Skills define what an agent can do. Examples:
- `frontend`: React, Vue, CSS, HTML expertise
- `backend`: API design, database, server logic
- `testing`: Unit tests, integration tests, E2E
- `documentation`: README, API docs, inline comments
- `refactoring`: Code optimization, cleanup
- `security`: Security audits, vulnerability fixes
- `devops`: CI/CD, deployment, infrastructure

### 3. Planner Agent
A specialized agent that:
1. Analyzes the epic task description
2. Estimates complexity (simple, moderate, complex)
3. Decides execution strategy:
   - Simple → Single agent execution
   - Moderate/Complex → Swarm execution
4. Decomposes into atomic subtasks with:
   - Clear boundaries
   - Dependency ordering
   - Required skills
   - Estimated effort

### 4. Swarm Execution
Orchestrates parallel execution:
- Creates sub-branches from epic branch
- Assigns agents to subtasks based on skills
- Monitors progress and handles failures
- Manages resource allocation

### 5. pBFT Consensus Review
Inspired by practical Byzantine Fault Tolerance:
- N reviewer agents review the combined changes
- Requires 2f+1 approvals where f = floor((N-1)/3)
- Each reviewer provides:
  - Approval/Rejection vote
  - Review comments
  - Suggested fixes
- Conflicts trigger human intervention

## Data Models

### SwarmExecution
```rust
pub struct SwarmExecution {
    pub id: Uuid,
    pub epic_task_id: Uuid,
    pub status: SwarmExecutionStatus,  // planning, executing, reviewing, merging, completed, failed
    pub planner_output: Option<String>, // JSON of decomposition plan
    pub consensus_threshold: i32,       // Number of approvals needed
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

### SwarmTask
```rust
pub struct SwarmTask {
    pub id: Uuid,
    pub swarm_execution_id: Uuid,
    pub task_id: Uuid,                  // Reference to actual Task
    pub workspace_id: Option<Uuid>,     // Workspace for this subtask
    pub sequence_order: i32,            // Execution order
    pub depends_on: Option<Vec<Uuid>>,  // Task dependencies
    pub required_skills: Vec<String>,   // Skills needed
    pub status: SwarmTaskStatus,        // pending, running, completed, failed
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
    pub is_reviewer: bool,      // Can this agent review?
    pub max_concurrent_tasks: i32,
}
```

### ConsensusReview
```rust
pub struct ConsensusReview {
    pub id: Uuid,
    pub swarm_execution_id: Uuid,
    pub reviewer_agent_id: Uuid,
    pub vote: ConsensusVote,    // approve, reject, abstain
    pub comments: Option<String>,
    pub review_diff_hash: String,
    pub created_at: DateTime<Utc>,
}
```

## API Endpoints

### Epic Task Management
- `POST /api/projects/{id}/epic-tasks` - Create epic task
- `GET /api/projects/{id}/epic-tasks` - List epic tasks
- `POST /api/epic-tasks/{id}/plan` - Trigger planner agent
- `POST /api/epic-tasks/{id}/execute` - Start swarm execution

### Swarm Management
- `GET /api/swarms/{id}` - Get swarm execution status
- `GET /api/swarms/{id}/tasks` - List subtasks
- `POST /api/swarms/{id}/pause` - Pause execution
- `POST /api/swarms/{id}/resume` - Resume execution
- `POST /api/swarms/{id}/cancel` - Cancel execution

### Agent Skills
- `GET /api/agent-skills` - List available skills
- `POST /api/agent-skills` - Create skill
- `GET /api/agent-profiles` - List agent profiles
- `POST /api/agent-profiles` - Create agent profile

### Consensus Review
- `GET /api/swarms/{id}/reviews` - Get consensus reviews
- `POST /api/swarms/{id}/reviews` - Submit review
- `POST /api/swarms/{id}/merge` - Trigger final merge

## Workflow

1. **Task Creation**: User creates an epic task with description
2. **Planning Phase**:
   - Planner agent analyzes the task
   - Generates decomposition plan
   - Creates subtasks with dependencies
3. **Execution Phase**:
   - Sub-branches created from epic branch
   - Worker agents assigned based on skills
   - Parallel execution with dependency ordering
4. **Review Phase**:
   - All subtasks complete
   - Reviewer agents analyze combined changes
   - pBFT consensus voting
5. **Merge Phase**:
   - Consensus reached
   - Sequential merge of subtask branches
   - Conflict resolution
   - Final commit to epic branch

## Configuration

```toml
[swarm]
max_parallel_agents = 5
consensus_reviewers = 3
min_consensus_threshold = 2
planner_model = "claude-3-opus"
default_worker_model = "claude-3-sonnet"
```

## Future Enhancements

1. **Learning System**: Track successful decompositions to improve planning
2. **Dynamic Scaling**: Auto-adjust swarm size based on task complexity
3. **Cross-Project Agents**: Share agent pools across projects
4. **Human-in-the-Loop**: Allow human intervention at any phase
5. **Cost Optimization**: Balance quality vs. API costs
