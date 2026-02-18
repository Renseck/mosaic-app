-- IF (NOT EXISTS (SELECT * FROM sys.schemas WHERE name = 'portal'))
-- BEGIN
--     EXEC ('CREATE SCHEMA [portal] AUTHORIZATION [dbo]')
-- END

CREATE SCHEMA IF NOT EXISTS portal;

-- Users & Auth
CREATE TABLE IF NOT EXISTS portal.users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      VARCHAR(64) UNIQUE NOT NULL,
    email         VARCHAR(255),
    password_hash TEXT NOT NULL,
    role          VARCHAR(16) NOT NULL DEFAULT 'viewer',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portal.sessions (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES portal.users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON portal.sessions(token_hash);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON portal.sessions(user_id);

-- Dashboards & Panels
CREATE TABLE IF NOT EXISTS portal.dashboards (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id   UUID REFERENCES portal.users(id),
    title      VARCHAR(255) NOT NULL,
    slug       VARCHAR(255) UNIQUE NOT NULL,
    icon       VARCHAR(64),
    sort_order INT NOT NULL DEFAULT 0,
    is_shared  BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS  portal.panels (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dashboard_id UUID NOT NULL REFERENCES portal.dashboards(id) ON DELETE CASCADE,
    title        VARCHAR(255),
    panel_type   VARCHAR(32) NOT NULL,
    source_url   TEXT,
    config       JSONB NOT NULL DEFAULT '{}',
    grid_x       INT NOT NULL DEFAULT 0,
    grid_y       INT NOT NULL DEFAULT 0,
    grid_w       INT NOT NULL DEFAULT 6,
    grid_h       INT NOT NULL DEFAULT 4,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_panels_dashboard_id ON portal.panels(dashboard_id);

-- Dataset Templates
CREATE TABLE IF NOT EXISTS portal.dataset_templates (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                  VARCHAR(255) NOT NULL,
    description           TEXT,
    nocodb_table_id       VARCHAR(255),
    nocodb_form_id        VARCHAR(255),
    grafana_dashboard_uid VARCHAR(255),
    fields                JSONB NOT NULL,
    created_by            UUID REFERENCES portal.users(id),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);
