-- WireSentinel Cloud compliance (004)

CREATE TABLE IF NOT EXISTS compliance_reports (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    check_type TEXT NOT NULL CHECK (check_type IN ('device', 'policy', 'encryption', 'privacy')),
    status TEXT NOT NULL CHECK (status IN ('passed', 'failed', 'warning')),
    summary TEXT NOT NULL,
    details TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_compliance_reports_tenant ON compliance_reports(tenant_id, created_at);
