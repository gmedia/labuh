-- Domains table for custom domain routing
CREATE TABLE IF NOT EXISTS domains (
    id TEXT PRIMARY KEY,
    stack_id TEXT NOT NULL REFERENCES stacks(id) ON DELETE CASCADE,
    container_name TEXT NOT NULL,
    container_port INTEGER NOT NULL DEFAULT 80,
    domain TEXT NOT NULL UNIQUE,
    ssl_enabled INTEGER NOT NULL DEFAULT 1,
    verified INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_domains_stack_id ON domains(stack_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_domains_domain ON domains(domain);
