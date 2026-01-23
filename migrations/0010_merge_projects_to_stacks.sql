-- Migration: Merge Projects into Stacks
-- Add webhook_token and deployment tracking to stacks

-- Add webhook_token column to stacks
ALTER TABLE stacks ADD COLUMN webhook_token TEXT;

-- Add stack_id column to domains (for migration)
ALTER TABLE domains ADD COLUMN stack_id TEXT;

-- Add stack_id column to deployment_logs (for migration)
ALTER TABLE deployment_logs ADD COLUMN stack_id TEXT;

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_domains_stack_id ON domains(stack_id);
CREATE INDEX IF NOT EXISTS idx_deployment_logs_stack_id ON deployment_logs(stack_id);
