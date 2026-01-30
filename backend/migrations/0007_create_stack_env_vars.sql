-- Stack environment variables table
-- These are inherited by all containers in the stack
CREATE TABLE IF NOT EXISTS stack_env_vars (
    id TEXT PRIMARY KEY,
    stack_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    is_secret BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (stack_id) REFERENCES stacks(id) ON DELETE CASCADE,
    UNIQUE(stack_id, key)
);

CREATE INDEX idx_stack_env_vars_stack_id ON stack_env_vars(stack_id);
