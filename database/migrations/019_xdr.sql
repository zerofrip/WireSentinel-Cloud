-- XDR cloud integration

CREATE TABLE IF NOT EXISTS tenant_xdr_incidents (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    severity TEXT NOT NULL DEFAULT 'medium',
    detection_count INTEGER NOT NULL DEFAULT 0,
    content_json TEXT NOT NULL DEFAULT '{}',
    opened_at TEXT NOT NULL,
    resolved_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_xdr_detections (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    rule_name TEXT NOT NULL,
    rule_kind TEXT NOT NULL DEFAULT 'behavioral',
    severity TEXT NOT NULL DEFAULT 'medium',
    mitre_technique_id TEXT,
    device_id TEXT,
    matched_at TEXT NOT NULL,
    content_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_xdr_hunts (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    query_kind TEXT NOT NULL DEFAULT 'historical',
    status TEXT NOT NULL DEFAULT 'draft',
    enabled INTEGER NOT NULL DEFAULT 1,
    query_json TEXT NOT NULL DEFAULT '{}',
    results_count INTEGER NOT NULL DEFAULT 0,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_xdr_mitre_coverage (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    tactic TEXT NOT NULL,
    technique_id TEXT NOT NULL,
    technique_name TEXT NOT NULL,
    detection_count INTEGER NOT NULL DEFAULT 0,
    coverage_pct REAL NOT NULL DEFAULT 0,
    last_seen_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_xdr_analytics_rollups (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    reporting_devices INTEGER NOT NULL DEFAULT 0,
    total_incidents INTEGER NOT NULL DEFAULT 0,
    open_incidents INTEGER NOT NULL DEFAULT 0,
    critical_incidents INTEGER NOT NULL DEFAULT 0,
    total_detections INTEGER NOT NULL DEFAULT 0,
    active_hunts INTEGER NOT NULL DEFAULT 0,
    mitre_techniques_detected INTEGER NOT NULL DEFAULT 0,
    mitre_coverage_pct REAL NOT NULL DEFAULT 0,
    avg_incident_mttr_hours REAL NOT NULL DEFAULT 0,
    fleet_threat_score REAL NOT NULL DEFAULT 0,
    rollup_json TEXT NOT NULL DEFAULT '{}',
    rolled_up_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tenant_xdr_incidents_tenant ON tenant_xdr_incidents(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_xdr_incidents_status ON tenant_xdr_incidents(status);
CREATE INDEX IF NOT EXISTS idx_tenant_xdr_detections_tenant ON tenant_xdr_detections(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_xdr_detections_matched ON tenant_xdr_detections(matched_at);
CREATE INDEX IF NOT EXISTS idx_tenant_xdr_hunts_tenant ON tenant_xdr_hunts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_xdr_mitre_coverage_tenant ON tenant_xdr_mitre_coverage(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_xdr_analytics_rollups_tenant ON tenant_xdr_analytics_rollups(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_xdr_analytics_rollups_rolled ON tenant_xdr_analytics_rollups(rolled_up_at);
