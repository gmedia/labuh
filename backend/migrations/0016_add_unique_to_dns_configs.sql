-- Add unique constraint to dns_configs on (team_id, provider)
-- This is required for the ON CONFLICT clause to work correctly
CREATE UNIQUE INDEX IF NOT EXISTS idx_dns_configs_team_provider_unique ON dns_configs(team_id, provider);
