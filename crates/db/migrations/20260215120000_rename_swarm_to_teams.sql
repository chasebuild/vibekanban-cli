PRAGMA foreign_keys=OFF;

-- Create team_executions table
CREATE TABLE team_executions (
    id TEXT PRIMARY KEY NOT NULL,
    -- The epic task being executed
    epic_task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    -- Parent workspace for the epic task
    epic_workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL,
    -- Execution status
    status TEXT NOT NULL DEFAULT 'planning' CHECK (status IN (
        'planning',      -- Team manager is decomposing the task
        'planned',       -- Decomposition complete, ready for execution
        'executing',     -- Worker agents are executing subtasks
        'completed',     -- Successfully completed
        'failed',        -- Execution failed
        'cancelled'      -- Cancelled by user
    )),
    -- JSON output from team manager with decomposition plan
    planner_output TEXT,
    -- Team manager agent profile used
    planner_profile_id TEXT REFERENCES agent_profiles(id),
    -- Maximum parallel workers
    max_parallel_workers INTEGER NOT NULL DEFAULT 3,
    -- Error message if failed
    error_message TEXT,
    -- Timestamps
    planned_at TEXT,
    execution_started_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    completed_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

-- Migrate swarm execution data
INSERT INTO team_executions (
    id,
    epic_task_id,
    epic_workspace_id,
    status,
    planner_output,
    planner_profile_id,
    max_parallel_workers,
    error_message,
    planned_at,
    execution_started_at,
    created_at,
    completed_at,
    updated_at
)
SELECT
    id,
    epic_task_id,
    epic_workspace_id,
    CASE
        WHEN status IN ('reviewing', 'merging') THEN 'executing'
        ELSE status
    END,
    planner_output,
    planner_profile_id,
    max_parallel_workers,
    error_message,
    planned_at,
    execution_started_at,
    created_at,
    completed_at,
    updated_at
FROM swarm_executions;

-- Create team_tasks table
CREATE TABLE team_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    team_execution_id TEXT NOT NULL REFERENCES team_executions(id) ON DELETE CASCADE,
    -- Reference to the actual task entity
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    -- Workspace for this subtask
    workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL,
    -- Execution order (for dependency management)
    sequence_order INTEGER NOT NULL DEFAULT 0,
    -- JSON array of team_task IDs this task depends on
    depends_on TEXT,
    -- JSON array of required skill IDs
    required_skills TEXT,
    -- Assigned agent profile
    assigned_agent_profile_id TEXT REFERENCES agent_profiles(id),
    -- Status of this subtask
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending',       -- Waiting to be executed
        'blocked',       -- Blocked by dependencies
        'assigned',      -- Assigned to an agent
        'running',       -- Currently executing
        'completed',     -- Successfully completed
        'failed',        -- Failed to execute
        'skipped'        -- Skipped (e.g., due to parent failure)
    )),
    -- Git branch for this subtask
    branch_name TEXT,
    -- Estimated complexity (1-5)
    complexity INTEGER NOT NULL DEFAULT 3,
    -- Actual duration in seconds
    duration_seconds INTEGER,
    -- Error message if failed
    error_message TEXT,
    -- Number of retry attempts
    retry_count INTEGER NOT NULL DEFAULT 0,
    -- Max retries allowed
    max_retries INTEGER NOT NULL DEFAULT 2,
    -- Timestamps
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

-- Migrate swarm task data
INSERT INTO team_tasks (
    id,
    team_execution_id,
    task_id,
    workspace_id,
    sequence_order,
    depends_on,
    required_skills,
    assigned_agent_profile_id,
    status,
    branch_name,
    complexity,
    duration_seconds,
    error_message,
    retry_count,
    max_retries,
    started_at,
    completed_at,
    created_at,
    updated_at
)
SELECT
    id,
    swarm_execution_id,
    task_id,
    workspace_id,
    sequence_order,
    depends_on,
    required_skills,
    assigned_agent_profile_id,
    status,
    branch_name,
    complexity,
    duration_seconds,
    error_message,
    retry_count,
    max_retries,
    started_at,
    completed_at,
    created_at,
    updated_at
FROM swarm_tasks;

DROP TABLE IF EXISTS consensus_reviews;
DROP TABLE swarm_tasks;
DROP TABLE swarm_executions;

CREATE INDEX idx_team_executions_epic_task ON team_executions(epic_task_id);
CREATE INDEX idx_team_executions_status ON team_executions(status);
CREATE INDEX idx_team_executions_created_at ON team_executions(created_at);

CREATE INDEX idx_team_tasks_team_execution ON team_tasks(team_execution_id);
CREATE INDEX idx_team_tasks_task ON team_tasks(task_id);
CREATE INDEX idx_team_tasks_status ON team_tasks(status);
CREATE INDEX idx_team_tasks_sequence ON team_tasks(team_execution_id, sequence_order);

PRAGMA foreign_keys=ON;
