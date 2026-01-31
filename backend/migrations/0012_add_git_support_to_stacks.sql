-- Add Git support to stacks
ALTER TABLE stacks ADD COLUMN git_url TEXT;
ALTER TABLE stacks ADD COLUMN git_branch TEXT;
ALTER TABLE stacks ADD COLUMN last_commit_hash TEXT;
