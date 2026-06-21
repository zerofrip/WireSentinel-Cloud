-- Phase 14 additive: rebuild provisioning_jobs with Phase 14 columns (no federated_controllers FK)

CREATE TABLE IF NOT EXISTS provisioning_jobs_phase14 (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    job_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    controller_id TEXT,
    region_id TEXT,
    plan_tier TEXT,
    payload TEXT NOT NULL DEFAULT '{}',
    error_message TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT
);

INSERT OR IGNORE INTO provisioning_jobs_phase14 (
    id, tenant_id, job_type, status, controller_id, payload, created_at, updated_at
)
SELECT
    id,
    tenant_id,
    job_type,
    status,
    controller_id,
    payload,
    created_at,
    COALESCE(completed_at, created_at)
FROM provisioning_jobs;

DROP TABLE provisioning_jobs;
ALTER TABLE provisioning_jobs_phase14 RENAME TO provisioning_jobs;

CREATE INDEX IF NOT EXISTS idx_provisioning_jobs_tenant ON provisioning_jobs(tenant_id, status);
