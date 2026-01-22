-- Swarm Executions: Track swarm-based task execution
CREATE TABLE swarm_executions (
    id TEXT PRIMARY KEY NOT NULL,
    -- The epic task being executed
    epic_task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    -- Parent workspace for the epic task
    epic_workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL,
    -- Execution status
    status TEXT NOT NULL DEFAULT 'planning' CHECK (status IN (
        'planning',      -- Planner agent is decomposing the task
        'planned',       -- Decomposition complete, ready for execution
        'executing',     -- Worker agents are executing subtasks
        'reviewing',     -- Consensus review in progress
        'merging',       -- Merging subtask branches
        'completed',     -- Successfully completed
        'failed',        -- Execution failed
        'cancelled'      -- Cancelled by user
    )),
    -- JSON output from planner agent with decomposition plan
    planner_output TEXT,
    -- Planner agent profile used
    planner_profile_id TEXT REFERENCES agent_profiles(id),
    -- Number of reviewer agents
    reviewer_count INTEGER NOT NULL DEFAULT 3,
    -- Consensus threshold (2f+1 where f = floor((N-1)/3))
    consensus_threshold INTEGER NOT NULL DEFAULT 2,
    -- Current consensus votes (approve count)
    consensus_approvals INTEGER NOT NULL DEFAULT 0,
    -- Current consensus votes (reject count)  
    consensus_rejections INTEGER NOT NULL DEFAULT 0,
    -- Maximum parallel workers
    max_parallel_workers INTEGER NOT NULL DEFAULT 3,
    -- Error message if failed
    error_message TEXT,
    -- Timestamps
    planned_at TEXT,
    execution_started_at TEXT,
    review_started_at TEXT,
    merge_started_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    completed_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

CREATE INDEX idx_swarm_executions_epic_task ON swarm_executions(epic_task_id);
CREATE INDEX idx_swarm_executions_status ON swarm_executions(status);
CREATE INDEX idx_swarm_executions_created_at ON swarm_executions(created_at);
