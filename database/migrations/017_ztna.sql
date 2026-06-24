-- ZTNA cloud integration: identity, trust, policy, resource, segment, connector, decision, rollup tables

CREATE TABLE IF NOT EXISTS identity_providers (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    provider_kind TEXT NOT NULL DEFAULT 'generic_oidc',
    config_json TEXT NOT NULL DEFAULT '{}',
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_identities (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    provider_id TEXT NOT NULL REFERENCES identity_providers(id),
    subject TEXT NOT NULL,
    email TEXT,
    display_name TEXT NOT NULL,
    authenticated_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(tenant_id, provider_id, subject)
);

CREATE TABLE IF NOT EXISTS group_identities (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    provider_id TEXT NOT NULL REFERENCES identity_providers(id),
    name TEXT NOT NULL,
    external_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(tenant_id, provider_id, name)
);

CREATE TABLE IF NOT EXISTS role_identities (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    provider_id TEXT NOT NULL REFERENCES identity_providers(id),
    name TEXT NOT NULL,
    permissions_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(tenant_id, provider_id, name)
);

CREATE TABLE IF NOT EXISTS device_trust_records (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    device_id TEXT NOT NULL,
    trust_level TEXT NOT NULL DEFAULT 'medium',
    trust_score INTEGER NOT NULL DEFAULT 50,
    posture_json TEXT NOT NULL DEFAULT '{}',
    last_evaluated_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(tenant_id, device_id)
);

CREATE TABLE IF NOT EXISTS conditional_access_policies (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    conditions_json TEXT NOT NULL DEFAULT '[]',
    action TEXT NOT NULL DEFAULT 'deny',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ztna_policies (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    min_trust_level TEXT NOT NULL DEFAULT 'medium',
    min_trust_score INTEGER NOT NULL DEFAULT 50,
    conditions_json TEXT NOT NULL DEFAULT '[]',
    default_action TEXT NOT NULL DEFAULT 'deny',
    content_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS published_resources (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    resource_type TEXT NOT NULL DEFAULT 'https',
    host TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 443,
    path_prefix TEXT,
    tags_json TEXT NOT NULL DEFAULT '[]',
    published INTEGER NOT NULL DEFAULT 0,
    access_policy_id TEXT,
    published_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS micro_segments (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    segment_type TEXT NOT NULL DEFAULT 'application',
    member_resource_ids_json TEXT NOT NULL DEFAULT '[]',
    isolation_level TEXT NOT NULL DEFAULT 'restricted',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ztna_connectors (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    connector_id TEXT NOT NULL,
    name TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    resource_ids_json TEXT NOT NULL DEFAULT '[]',
    healthy INTEGER NOT NULL DEFAULT 1,
    last_seen_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(tenant_id, connector_id)
);

CREATE TABLE IF NOT EXISTS trust_scores (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    device_id TEXT NOT NULL,
    score INTEGER NOT NULL DEFAULT 0,
    trust_level TEXT NOT NULL DEFAULT 'medium',
    captured_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ztna_access_decisions (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    subject_id TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    decision TEXT NOT NULL,
    trust_score INTEGER NOT NULL DEFAULT 0,
    reason TEXT NOT NULL DEFAULT '',
    recorded_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cloud_ztna_rollups (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    reporting_devices INTEGER NOT NULL DEFAULT 0,
    avg_trust_score REAL NOT NULL DEFAULT 0,
    allow_count INTEGER NOT NULL DEFAULT 0,
    deny_count INTEGER NOT NULL DEFAULT 0,
    challenge_count INTEGER NOT NULL DEFAULT 0,
    published_resources INTEGER NOT NULL DEFAULT 0,
    rollup_json TEXT NOT NULL DEFAULT '{}',
    rolled_up_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_identity_providers_tenant ON identity_providers(tenant_id);
CREATE INDEX IF NOT EXISTS idx_published_resources_tenant ON published_resources(tenant_id);
CREATE INDEX IF NOT EXISTS idx_ztna_policies_tenant ON ztna_policies(tenant_id);
CREATE INDEX IF NOT EXISTS idx_ztna_decisions_tenant ON ztna_access_decisions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_cloud_ztna_rollups_tenant ON cloud_ztna_rollups(tenant_id);
