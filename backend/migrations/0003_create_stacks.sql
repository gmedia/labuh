-- Stacks table for Docker Compose support
CREATE TABLE IF NOT EXISTS stacks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    compose_content TEXT,
    status TEXT NOT NULL DEFAULT 'stopped',
    webhook_token TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_stacks_user_id ON stacks(user_id);
