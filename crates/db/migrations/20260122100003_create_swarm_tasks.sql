-- Swarm Tasks: Individual subtasks within a swarm execution
CREATE TABLE swarm_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    swarm_execution_id TEXT NOT NULL REFERENCES swarm_executions(id) ON DELETE CASCADE,
    -- Reference to the actual task entity
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    -- Workspace for this subtask
    workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL,
    -- Execution order (for dependency management)
    sequence_order INTEGER NOT NULL DEFAULT 0,
    -- JSON array of swarm_task IDs this task depends on
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

CREATE INDEX idx_swarm_tasks_swarm_execution ON swarm_tasks(swarm_execution_id);
CREATE INDEX idx_swarm_tasks_task ON swarm_tasks(task_id);
CREATE INDEX idx_swarm_tasks_status ON swarm_tasks(status);
CREATE INDEX idx_swarm_tasks_sequence ON swarm_tasks(swarm_execution_id, sequence_order);
