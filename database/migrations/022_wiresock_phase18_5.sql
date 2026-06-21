-- Phase 18.5 WireSock cloud integration (18.5-L)

CREATE TABLE IF NOT EXISTS tenant_wiresock_split_templates (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    template_mode TEXT NOT NULL DEFAULT 'disabled',
    enabled INTEGER NOT NULL DEFAULT 1,
    app_rules_count INTEGER NOT NULL DEFAULT 0,
    domain_rules_count INTEGER NOT NULL DEFAULT 0,
    content_json TEXT NOT NULL DEFAULT '{}',
    synced_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_wiresock_tcp_termination (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    mode TEXT NOT NULL DEFAULT 'disabled',
    rule_name TEXT NOT NULL,
    process_name TEXT,
    profile_id TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    content_json TEXT NOT NULL DEFAULT '{}',
    synced_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_wiresock_handshake_proxy (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    name TEXT NOT NULL,
    proxy_type TEXT NOT NULL DEFAULT 'socks5',
    endpoint TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    content_json TEXT NOT NULL DEFAULT '{}',
    synced_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_wiresock_analytics_rollups (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    reporting_endpoints INTEGER NOT NULL DEFAULT 0,
    active_split_templates INTEGER NOT NULL DEFAULT 0,
    tcp_termination_rules INTEGER NOT NULL DEFAULT 0,
    handshake_proxy_active INTEGER NOT NULL DEFAULT 0,
    bypass_events INTEGER NOT NULL DEFAULT 0,
    fleet_health_score REAL NOT NULL DEFAULT 0,
    rollup_json TEXT NOT NULL DEFAULT '{}',
    rolled_up_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tenant_wiresock_split_templates_tenant ON tenant_wiresock_split_templates(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_wiresock_split_templates_synced ON tenant_wiresock_split_templates(synced_at);
CREATE INDEX IF NOT EXISTS idx_tenant_wiresock_tcp_termination_tenant ON tenant_wiresock_tcp_termination(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_wiresock_tcp_termination_synced ON tenant_wiresock_tcp_termination(synced_at);
CREATE INDEX IF NOT EXISTS idx_tenant_wiresock_handshake_proxy_tenant ON tenant_wiresock_handshake_proxy(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_wiresock_handshake_proxy_synced ON tenant_wiresock_handshake_proxy(synced_at);
CREATE INDEX IF NOT EXISTS idx_tenant_wiresock_analytics_rollups_tenant ON tenant_wiresock_analytics_rollups(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_wiresock_analytics_rollups_rolled ON tenant_wiresock_analytics_rollups(rolled_up_at);
