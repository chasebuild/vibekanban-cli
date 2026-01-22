-- Agent Profiles: Named agent configurations with specific skills
CREATE TABLE agent_profiles (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    -- Base executor type (CLAUDE_CODE, CODEX, etc.)
    executor TEXT NOT NULL,
    -- Executor variant (e.g., 'PLAN', 'ROUTER')
    variant TEXT,
    -- JSON configuration for the executor
    executor_config TEXT,
    -- Can this agent decompose tasks?
    is_planner INTEGER NOT NULL DEFAULT 0,
    -- Can this agent review code?
    is_reviewer INTEGER NOT NULL DEFAULT 0,
    -- Can this agent execute tasks?
    is_worker INTEGER NOT NULL DEFAULT 1,
    -- Maximum concurrent tasks this agent can handle
    max_concurrent_tasks INTEGER NOT NULL DEFAULT 1,
    -- Priority for assignment (higher = preferred)
    priority INTEGER NOT NULL DEFAULT 0,
    -- Is this profile active?
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

-- Junction table for agent profile skills
CREATE TABLE agent_profile_skills (
    agent_profile_id TEXT NOT NULL REFERENCES agent_profiles(id) ON DELETE CASCADE,
    agent_skill_id TEXT NOT NULL REFERENCES agent_skills(id) ON DELETE CASCADE,
    -- Proficiency level (1-5, higher = more proficient)
    proficiency INTEGER NOT NULL DEFAULT 3,
    PRIMARY KEY (agent_profile_id, agent_skill_id)
);

-- Create default planner and reviewer profiles
INSERT INTO agent_profiles (id, name, description, executor, is_planner, is_reviewer, is_worker, max_concurrent_tasks) VALUES
    ('profile-planner', 'Planner Agent', 'Specialized in task decomposition and planning', 'CLAUDE_CODE', 1, 0, 0, 1),
    ('profile-reviewer', 'Reviewer Agent', 'Specialized in code review and consensus', 'CLAUDE_CODE', 0, 1, 0, 3),
    ('profile-worker-general', 'General Worker', 'General-purpose worker agent', 'CLAUDE_CODE', 0, 0, 1, 2);

-- Link skills to the general worker
INSERT INTO agent_profile_skills (agent_profile_id, agent_skill_id, proficiency) VALUES
    ('profile-worker-general', 'skill-frontend', 3),
    ('profile-worker-general', 'skill-backend', 3),
    ('profile-worker-general', 'skill-testing', 2),
    ('profile-worker-general', 'skill-documentation', 2);

CREATE INDEX idx_agent_profiles_executor ON agent_profiles(executor);
CREATE INDEX idx_agent_profiles_is_planner ON agent_profiles(is_planner);
CREATE INDEX idx_agent_profiles_is_reviewer ON agent_profiles(is_reviewer);
CREATE INDEX idx_agent_profiles_active ON agent_profiles(active);
