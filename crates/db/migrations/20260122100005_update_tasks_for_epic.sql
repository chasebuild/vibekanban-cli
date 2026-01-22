-- Update tasks table to support epic task features
ALTER TABLE tasks ADD COLUMN is_epic INTEGER NOT NULL DEFAULT 0;

-- Estimated complexity for the task (used for planning)
ALTER TABLE tasks ADD COLUMN complexity TEXT CHECK (complexity IN ('trivial', 'simple', 'moderate', 'complex', 'epic'));

-- JSON metadata for additional task properties
ALTER TABLE tasks ADD COLUMN metadata TEXT;

CREATE INDEX idx_tasks_is_epic ON tasks(is_epic);
CREATE INDEX idx_tasks_complexity ON tasks(complexity);
