-- Phase 18 CNAPP cloud integration (18-L)

CREATE TABLE IF NOT EXISTS tenant_cnapp_posture (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    cloud_provider TEXT NOT NULL DEFAULT 'aws',
    account_id TEXT,
    resource_kind TEXT NOT NULL DEFAULT 'account',
    posture_score REAL NOT NULL DEFAULT 0,
    risk_level TEXT NOT NULL DEFAULT 'medium',
    findings_count INTEGER NOT NULL DEFAULT 0,
    content_json TEXT NOT NULL DEFAULT '{}',
    assessed_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_cnapp_compliance (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    framework TEXT NOT NULL,
    control_id TEXT NOT NULL,
    control_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'unknown',
    compliance_pct REAL NOT NULL DEFAULT 0,
    last_checked_at TEXT,
    content_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_cnapp_vulnerabilities (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    cve_id TEXT,
    title TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'medium',
    resource_id TEXT,
    cloud_provider TEXT,
    status TEXT NOT NULL DEFAULT 'open',
    discovered_at TEXT NOT NULL,
    content_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_cnapp_attack_paths (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    name TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'high',
    path_length INTEGER NOT NULL DEFAULT 0,
    entry_point TEXT,
    target_asset TEXT,
    status TEXT NOT NULL DEFAULT 'open',
    content_json TEXT NOT NULL DEFAULT '{}',
    discovered_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_cnapp_analytics_rollups (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    reporting_accounts INTEGER NOT NULL DEFAULT 0,
    posture_score REAL NOT NULL DEFAULT 0,
    compliance_pct REAL NOT NULL DEFAULT 0,
    open_vulnerabilities INTEGER NOT NULL DEFAULT 0,
    critical_vulnerabilities INTEGER NOT NULL DEFAULT 0,
    attack_paths_detected INTEGER NOT NULL DEFAULT 0,
    multi_cloud_providers INTEGER NOT NULL DEFAULT 0,
    fleet_risk_score REAL NOT NULL DEFAULT 0,
    rollup_json TEXT NOT NULL DEFAULT '{}',
    rolled_up_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tenant_cnapp_posture_tenant ON tenant_cnapp_posture(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_cnapp_posture_assessed ON tenant_cnapp_posture(assessed_at);
CREATE INDEX IF NOT EXISTS idx_tenant_cnapp_compliance_tenant ON tenant_cnapp_compliance(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_cnapp_compliance_framework ON tenant_cnapp_compliance(framework);
CREATE INDEX IF NOT EXISTS idx_tenant_cnapp_vulnerabilities_tenant ON tenant_cnapp_vulnerabilities(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_cnapp_vulnerabilities_discovered ON tenant_cnapp_vulnerabilities(discovered_at);
CREATE INDEX IF NOT EXISTS idx_tenant_cnapp_attack_paths_tenant ON tenant_cnapp_attack_paths(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_cnapp_attack_paths_discovered ON tenant_cnapp_attack_paths(discovered_at);
CREATE INDEX IF NOT EXISTS idx_tenant_cnapp_analytics_rollups_tenant ON tenant_cnapp_analytics_rollups(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_cnapp_analytics_rollups_rolled ON tenant_cnapp_analytics_rollups(rolled_up_at);
