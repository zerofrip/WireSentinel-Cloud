-- WireSentinel Cloud compliance anonymity check type (008)

PRAGMA foreign_keys=OFF;

CREATE TABLE compliance_reports_new (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    check_type TEXT NOT NULL CHECK (check_type IN ('device', 'policy', 'encryption', 'privacy', 'kernel', 'anonymity')),
    status TEXT NOT NULL CHECK (status IN ('passed', 'failed', 'warning')),
    summary TEXT NOT NULL,
    details TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

INSERT INTO compliance_reports_new SELECT * FROM compliance_reports;

DROP TABLE compliance_reports;

ALTER TABLE compliance_reports_new RENAME TO compliance_reports;

CREATE INDEX IF NOT EXISTS idx_compliance_reports_tenant ON compliance_reports(tenant_id, created_at);

PRAGMA foreign_keys=ON;
