-- Webhook logs table for tracking webhook invocations
CREATE TABLE IF NOT EXISTS webhook_logs (
    id TEXT PRIMARY KEY,
    stack_id TEXT NOT NULL,
    trigger_type TEXT NOT NULL,  -- 'manual', 'github', 'api'
    status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'success', 'failed'
    payload TEXT,
    response TEXT,
    triggered_at TEXT NOT NULL,
    completed_at TEXT,
    FOREIGN KEY (stack_id) REFERENCES stacks(id) ON DELETE CASCADE
);

CREATE INDEX idx_webhook_logs_stack_id ON webhook_logs(stack_id);
CREATE INDEX idx_webhook_logs_triggered_at ON webhook_logs(triggered_at);
