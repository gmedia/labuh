-- Add advanced domain fields
ALTER TABLE domains ADD COLUMN provider TEXT NOT NULL DEFAULT 'Custom';
ALTER TABLE domains ADD COLUMN type TEXT NOT NULL DEFAULT 'Caddy';
ALTER TABLE domains ADD COLUMN tunnel_id TEXT;
ALTER TABLE domains ADD COLUMN dns_record_id TEXT;
