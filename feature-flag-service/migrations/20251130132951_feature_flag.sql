-- migrations/004_feature_flags.sql

-- Projects (different apps using your feature flag service)
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    sdk_key TEXT UNIQUE NOT NULL, -- For API authentication
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Feature flags belonging to projects
CREATE TABLE feature_flags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL, -- e.g., "new_checkout_flow"
    key TEXT NOT NULL, -- Unique key within project for SDK lookups
    description TEXT,
    enabled BOOLEAN DEFAULT FALSE, -- Global kill switch
    rollout_percentage INT DEFAULT 0 CHECK (rollout_percentage >= 0 AND rollout_percentage <= 100),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(project_id, key) -- Keys must be unique per project
);

-- Targeting rules (who gets the flag enabled)
CREATE TABLE flag_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flag_id UUID NOT NULL REFERENCES feature_flags(id) ON DELETE CASCADE,
    rule_type TEXT NOT NULL, -- 'user_id', 'user_email', 'attribute_match'
    rule_value TEXT NOT NULL, -- The actual value to match
    enabled BOOLEAN DEFAULT TRUE,
    priority INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Flag evaluation history (optional - for analytics)
CREATE TABLE flag_evaluations (
    id BIGSERIAL PRIMARY KEY,
    flag_id UUID NOT NULL REFERENCES feature_flags(id) ON DELETE CASCADE,
    user_identifier TEXT NOT NULL, -- Could be email, user_id, or custom identifier
    result BOOLEAN NOT NULL, -- Was flag enabled for this user?
    evaluated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for fast lookups
CREATE INDEX idx_projects_created_by ON projects(created_by);
CREATE INDEX idx_flags_project ON feature_flags(project_id);
CREATE INDEX idx_flags_project_key ON feature_flags(project_id, key);
CREATE INDEX idx_rules_flag ON flag_rules(flag_id);
CREATE INDEX idx_rules_flag_priority ON flag_rules(flag_id, priority DESC);
CREATE INDEX idx_evaluations_flag_time ON flag_evaluations(flag_id, evaluated_at DESC);
CREATE INDEX idx_project_sdk_key ON projects(sdk_key);

