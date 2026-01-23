-- Domains table for custom domain routing
CREATE TABLE IF NOT EXISTS domains (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    domain TEXT NOT NULL UNIQUE,
    ssl_enabled INTEGER NOT NULL DEFAULT 1,
    verified INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_domains_project_id ON domains(project_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_domains_domain ON domains(domain);
