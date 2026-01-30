-- Add container_name to stack_env_vars
-- This allows per-container environment variables
-- If container_name is empty string, it applies to all containers (global)

CREATE TABLE stack_env_vars_new (
    id TEXT PRIMARY KEY,
    stack_id TEXT NOT NULL,
    container_name TEXT NOT NULL DEFAULT '',
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    is_secret BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (stack_id) REFERENCES stacks(id) ON DELETE CASCADE,
    UNIQUE(stack_id, container_name, key)
);

-- Copy existing data (all were global)
INSERT INTO stack_env_vars_new (id, stack_id, container_name, key, value, is_secret, created_at, updated_at)
SELECT id, stack_id, '', key, value, is_secret, created_at, updated_at FROM stack_env_vars;

DROP TABLE stack_env_vars;
ALTER TABLE stack_env_vars_new RENAME TO stack_env_vars;

CREATE INDEX idx_stack_env_vars_stack_id ON stack_env_vars(stack_id);
CREATE INDEX idx_stack_env_vars_stack_container ON stack_env_vars(stack_id, container_name);
