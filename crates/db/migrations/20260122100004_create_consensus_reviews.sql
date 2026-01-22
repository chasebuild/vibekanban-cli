-- Consensus Reviews: pBFT-style consensus voting for swarm execution
CREATE TABLE consensus_reviews (
    id TEXT PRIMARY KEY NOT NULL,
    swarm_execution_id TEXT NOT NULL REFERENCES swarm_executions(id) ON DELETE CASCADE,
    -- Reviewer agent profile
    reviewer_profile_id TEXT NOT NULL REFERENCES agent_profiles(id),
    -- Session for the review process
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    -- Vote decision
    vote TEXT NOT NULL CHECK (vote IN ('approve', 'reject', 'abstain', 'pending')),
    -- Review comments/feedback
    comments TEXT,
    -- JSON structured feedback
    structured_feedback TEXT,
    -- Hash of the diff being reviewed (for consistency verification)
    review_diff_hash TEXT,
    -- Confidence score (0-100)
    confidence INTEGER,
    -- Categories of issues found (JSON array)
    issues_found TEXT,
    -- Suggested fixes (JSON array)
    suggested_fixes TEXT,
    -- Round number for multi-round consensus
    round INTEGER NOT NULL DEFAULT 1,
    -- Timestamps
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

CREATE INDEX idx_consensus_reviews_swarm ON consensus_reviews(swarm_execution_id);
CREATE INDEX idx_consensus_reviews_vote ON consensus_reviews(swarm_execution_id, vote);
CREATE INDEX idx_consensus_reviews_round ON consensus_reviews(swarm_execution_id, round);
