-- Phase 14-G/H: backup objects metadata and aggregated logs

CREATE TABLE IF NOT EXISTS backup_objects (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    storage_provider TEXT NOT NULL DEFAULT 'local',
    object_key TEXT NOT NULL,
    content_type TEXT,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    checksum TEXT,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    UNIQUE (tenant_id, object_key)
);

CREATE TABLE IF NOT EXISTS aggregated_logs (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    source TEXT NOT NULL,
    level TEXT NOT NULL DEFAULT 'info',
    message TEXT NOT NULL,
    fields_json TEXT NOT NULL DEFAULT '{}',
    ingested_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_backup_objects_tenant ON backup_objects(tenant_id);
CREATE INDEX IF NOT EXISTS idx_aggregated_logs_tenant ON aggregated_logs(tenant_id, ingested_at);
