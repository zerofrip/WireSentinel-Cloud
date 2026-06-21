-- Phase 14-B/C: usage snapshots, aggregates, tenant quotas, grace periods

CREATE TABLE IF NOT EXISTS usage_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    metric TEXT NOT NULL,
    value REAL NOT NULL DEFAULT 0,
    window_start TEXT NOT NULL,
    window_end TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS usage_aggregates (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    metric TEXT NOT NULL,
    period TEXT NOT NULL,
    total REAL NOT NULL DEFAULT 0,
    peak REAL NOT NULL DEFAULT 0,
    sample_count INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL,
    UNIQUE (tenant_id, metric, period)
);

CREATE TABLE IF NOT EXISTS tenant_quotas (
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    resource TEXT NOT NULL,
    soft_limit REAL NOT NULL,
    hard_limit REAL NOT NULL,
    current_usage REAL NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (tenant_id, resource)
);

CREATE TABLE IF NOT EXISTS quota_grace_periods (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    resource TEXT NOT NULL,
    grace_until TEXT NOT NULL,
    reason TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_usage_snapshots_tenant ON usage_snapshots(tenant_id, metric);
CREATE INDEX IF NOT EXISTS idx_quota_grace_tenant ON quota_grace_periods(tenant_id, resource);
