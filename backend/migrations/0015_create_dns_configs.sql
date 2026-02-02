-- DNS Configurations table
CREATE TABLE IF NOT EXISTS dns_configs (
    id TEXT PRIMARY KEY,
    team_id TEXT NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    config TEXT NOT NULL, -- JSON string containing provider specific config
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_dns_configs_team_id ON dns_configs(team_id);
