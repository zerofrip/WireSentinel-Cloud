-- SSE cloud integration: policies and fleet rollups

CREATE TABLE IF NOT EXISTS sse_policies (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    policy_kind TEXT NOT NULL DEFAULT 'swg',
    enabled INTEGER NOT NULL DEFAULT 1,
    rules_json TEXT NOT NULL DEFAULT '[]',
    default_action TEXT NOT NULL DEFAULT 'block',
    content_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cloud_sse_rollups (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    reporting_devices INTEGER NOT NULL DEFAULT 0,
    swg_requests INTEGER NOT NULL DEFAULT 0,
    swg_blocked INTEGER NOT NULL DEFAULT 0,
    threat_count INTEGER NOT NULL DEFAULT 0,
    casb_incidents INTEGER NOT NULL DEFAULT 0,
    dlp_incidents INTEGER NOT NULL DEFAULT 0,
    avg_risk_score REAL NOT NULL DEFAULT 0,
    ueba_alerts INTEGER NOT NULL DEFAULT 0,
    rollup_json TEXT NOT NULL DEFAULT '{}',
    rolled_up_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sse_policies_tenant ON sse_policies(tenant_id);
CREATE INDEX IF NOT EXISTS idx_cloud_sse_rollups_tenant ON cloud_sse_rollups(tenant_id);
CREATE INDEX IF NOT EXISTS idx_cloud_sse_rollups_rolled ON cloud_sse_rollups(rolled_up_at);
