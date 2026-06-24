-- AI Security cloud integration

CREATE TABLE IF NOT EXISTS tenant_ai_investigations (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    severity TEXT NOT NULL DEFAULT 'medium',
    category TEXT NOT NULL DEFAULT 'general',
    model_name TEXT,
    agent_id TEXT,
    finding_count INTEGER NOT NULL DEFAULT 0,
    content_json TEXT NOT NULL DEFAULT '{}',
    opened_at TEXT NOT NULL,
    resolved_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_ai_risk (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    risk_category TEXT NOT NULL DEFAULT 'model',
    risk_score REAL NOT NULL DEFAULT 0,
    severity TEXT NOT NULL DEFAULT 'medium',
    model_name TEXT,
    resource_id TEXT,
    status TEXT NOT NULL DEFAULT 'open',
    content_json TEXT NOT NULL DEFAULT '{}',
    assessed_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_ai_reports (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    report_type TEXT NOT NULL DEFAULT 'security',
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    compliance_pct REAL NOT NULL DEFAULT 0,
    period_start TEXT,
    period_end TEXT,
    content_json TEXT NOT NULL DEFAULT '{}',
    generated_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_ai_correlations (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    correlation_key TEXT NOT NULL,
    event_count INTEGER NOT NULL DEFAULT 0,
    severity TEXT NOT NULL DEFAULT 'medium',
    status TEXT NOT NULL DEFAULT 'open',
    source_kinds_json TEXT NOT NULL DEFAULT '[]',
    content_json TEXT NOT NULL DEFAULT '{}',
    correlated_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_ai_analytics_rollups (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    reporting_agents INTEGER NOT NULL DEFAULT 0,
    open_investigations INTEGER NOT NULL DEFAULT 0,
    critical_risks INTEGER NOT NULL DEFAULT 0,
    total_correlations INTEGER NOT NULL DEFAULT 0,
    compliance_pct REAL NOT NULL DEFAULT 0,
    avg_risk_score REAL NOT NULL DEFAULT 0,
    prompt_injection_events INTEGER NOT NULL DEFAULT 0,
    data_exfiltration_events INTEGER NOT NULL DEFAULT 0,
    fleet_ai_risk_score REAL NOT NULL DEFAULT 0,
    rollup_json TEXT NOT NULL DEFAULT '{}',
    rolled_up_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tenant_ai_investigations_tenant ON tenant_ai_investigations(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_ai_investigations_opened ON tenant_ai_investigations(opened_at);
CREATE INDEX IF NOT EXISTS idx_tenant_ai_risk_tenant ON tenant_ai_risk(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_ai_risk_assessed ON tenant_ai_risk(assessed_at);
CREATE INDEX IF NOT EXISTS idx_tenant_ai_reports_tenant ON tenant_ai_reports(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_ai_reports_generated ON tenant_ai_reports(generated_at);
CREATE INDEX IF NOT EXISTS idx_tenant_ai_correlations_tenant ON tenant_ai_correlations(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_ai_correlations_correlated ON tenant_ai_correlations(correlated_at);
CREATE INDEX IF NOT EXISTS idx_tenant_ai_analytics_rollups_tenant ON tenant_ai_analytics_rollups(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_ai_analytics_rollups_rolled ON tenant_ai_analytics_rollups(rolled_up_at);
