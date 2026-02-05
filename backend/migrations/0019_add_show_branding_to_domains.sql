-- Add show_branding column to domains table
ALTER TABLE domains ADD COLUMN show_branding INTEGER NOT NULL DEFAULT 1;
