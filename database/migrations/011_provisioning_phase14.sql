-- Phase 14-D: hosted controllers and provisioning jobs

CREATE TABLE IF NOT EXISTS provisioning_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    job_type TEXT NOT NULL CHECK (job_type IN ('provision', 'upgrade', 'backup', 'restore')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    controller_id TEXT,
    region_id TEXT,
    plan_tier TEXT,
    payload TEXT NOT NULL DEFAULT '{}',
    error_message TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS hosted_controllers (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    region_id TEXT NOT NULL,
    plan_tier TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'provisioning' CHECK (status IN ('provisioning', 'active', 'upgrading', 'failed', 'terminated')),
    endpoint_url TEXT,
    version TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS hosted_controller_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    controller_id TEXT NOT NULL REFERENCES hosted_controllers(id),
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    snapshot_type TEXT NOT NULL DEFAULT 'manual',
    storage_key TEXT NOT NULL,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_provisioning_jobs_tenant ON provisioning_jobs(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_hosted_controllers_tenant ON hosted_controllers(tenant_id);
