-- Deployment logs for webhook history (stack deployments only)
CREATE TABLE IF NOT EXISTS deployment_logs (
    id TEXT PRIMARY KEY NOT NULL,
    stack_id TEXT NOT NULL,
    trigger_type TEXT NOT NULL, -- 'webhook', 'manual', 'api'
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'success', 'failed'
    logs TEXT,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    FOREIGN KEY (stack_id) REFERENCES stacks(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_deployment_logs_stack_id ON deployment_logs(stack_id);
CREATE INDEX IF NOT EXISTS idx_deployment_logs_started_at ON deployment_logs(started_at DESC);
