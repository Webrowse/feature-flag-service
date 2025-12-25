-- migrations/20251225000000_create_environments.sql

-- Environments table (production, staging, development, etc.)
CREATE TABLE environments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,           -- e.g., "Production", "Staging", "Development"
    key TEXT NOT NULL,            -- e.g., "production", "staging", "development"
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(project_id, key)       -- Keys must be unique per project
);

-- Add environment_id to feature_flags
ALTER TABLE feature_flags ADD COLUMN environment_id UUID REFERENCES environments(id) ON DELETE CASCADE;

-- Drop the old unique constraint on (project_id, key)
ALTER TABLE feature_flags DROP CONSTRAINT feature_flags_project_id_key_key;

-- Add new unique constraint: flag keys must be unique per environment
ALTER TABLE feature_flags ADD CONSTRAINT feature_flags_environment_id_key_key UNIQUE(environment_id, key);

-- Create indexes for fast lookups
CREATE INDEX idx_environments_project ON environments(project_id);
CREATE INDEX idx_flags_environment ON feature_flags(environment_id);
CREATE INDEX idx_flags_environment_key ON feature_flags(environment_id, key);
