-- Add webhook_token to projects table
ALTER TABLE projects ADD COLUMN webhook_token TEXT;
