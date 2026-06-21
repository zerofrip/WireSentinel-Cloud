-- Phase 14-E/F: regions, recovery plans and runs

CREATE TABLE IF NOT EXISTS cloud_regions (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    provider TEXT NOT NULL DEFAULT 'wiresentinel',
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'maintenance', 'disabled')),
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS region_health (
    id TEXT PRIMARY KEY NOT NULL,
    region_id TEXT NOT NULL REFERENCES cloud_regions(id),
    healthy INTEGER NOT NULL DEFAULT 1,
    latency_ms REAL,
    message TEXT,
    checked_at TEXT NOT NULL
);

ALTER TABLE tenants ADD COLUMN region_id TEXT REFERENCES cloud_regions(id);

CREATE TABLE IF NOT EXISTS recovery_plans (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    plan_type TEXT NOT NULL DEFAULT 'failover',
    target_region_id TEXT REFERENCES cloud_regions(id),
    steps_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS recovery_runs (
    id TEXT PRIMARY KEY NOT NULL,
    plan_id TEXT NOT NULL REFERENCES recovery_plans(id),
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    started_at TEXT,
    completed_at TEXT,
    details TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

INSERT OR IGNORE INTO cloud_regions (id, name, display_name, provider, status, created_at) VALUES
    ('us-east', 'us-east', 'US East', 'wiresentinel', 'active', datetime('now')),
    ('us-west', 'us-west', 'US West', 'wiresentinel', 'active', datetime('now')),
    ('eu', 'eu', 'Europe', 'wiresentinel', 'active', datetime('now')),
    ('apac', 'apac', 'Asia Pacific', 'wiresentinel', 'active', datetime('now')),
    ('custom', 'custom', 'Custom Region', 'wiresentinel', 'active', datetime('now'));

CREATE INDEX IF NOT EXISTS idx_region_health_region ON region_health(region_id, checked_at);
CREATE INDEX IF NOT EXISTS idx_recovery_runs_tenant ON recovery_runs(tenant_id, status);
