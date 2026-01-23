-- Agent Skills: Define capabilities that agents can have
CREATE TABLE agent_skills (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    -- Prompt modifier text that gets appended when this skill is active
    prompt_modifier TEXT,
    -- Category for grouping skills (e.g., 'development', 'testing', 'documentation')
    category TEXT NOT NULL DEFAULT 'general',
    -- Icon identifier for UI display
    icon TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

-- Pre-populate with common skills (using proper UUIDs)
INSERT INTO agent_skills (id, name, description, prompt_modifier, category) VALUES
    ('00000000-0000-0000-0000-000000000001', 'frontend', 'Frontend development: React, Vue, CSS, HTML, responsive design', 'You are an expert frontend developer. Focus on React best practices, component architecture, CSS styling, and responsive design.', 'development'),
    ('00000000-0000-0000-0000-000000000002', 'backend', 'Backend development: APIs, databases, server logic, microservices', 'You are an expert backend developer. Focus on API design, database optimization, server-side logic, and scalability.', 'development'),
    ('00000000-0000-0000-0000-000000000003', 'testing', 'Testing: Unit tests, integration tests, E2E tests, test coverage', 'You are an expert in software testing. Write comprehensive unit tests, integration tests, and ensure good test coverage.', 'quality'),
    ('00000000-0000-0000-0000-000000000004', 'documentation', 'Documentation: README files, API docs, inline comments, tutorials', 'You are an expert technical writer. Create clear, comprehensive documentation with examples and best practices.', 'documentation'),
    ('00000000-0000-0000-0000-000000000005', 'refactoring', 'Code refactoring: Optimization, cleanup, design patterns', 'You are an expert at code refactoring. Improve code quality, apply design patterns, and optimize performance without changing behavior.', 'development'),
    ('00000000-0000-0000-0000-000000000006', 'security', 'Security: Audits, vulnerability fixes, secure coding practices', 'You are a security expert. Identify vulnerabilities, apply secure coding practices, and implement proper authentication/authorization.', 'security'),
    ('00000000-0000-0000-0000-000000000007', 'devops', 'DevOps: CI/CD, deployment, infrastructure, containerization', 'You are a DevOps expert. Focus on CI/CD pipelines, containerization, infrastructure as code, and deployment automation.', 'infrastructure'),
    ('00000000-0000-0000-0000-000000000008', 'database', 'Database: Schema design, migrations, queries, optimization', 'You are a database expert. Design efficient schemas, write optimized queries, and handle migrations properly.', 'development'),
    ('00000000-0000-0000-0000-000000000009', 'architecture', 'Software architecture: System design, scalability, patterns', 'You are a software architect. Focus on system design, scalability patterns, and architectural decisions.', 'architecture');

CREATE INDEX idx_agent_skills_category ON agent_skills(category);
CREATE INDEX idx_agent_skills_name ON agent_skills(name);
