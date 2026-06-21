-- Cloud anonymity fleet rollups from federated controllers (Phase 13-M)

CREATE TABLE IF NOT EXISTS cloud_anonymity_rollups (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    reporting_devices INTEGER NOT NULL DEFAULT 0,
    healthy_devices INTEGER NOT NULL DEFAULT 0,
    connected_devices INTEGER NOT NULL DEFAULT 0,
    federation_peers_total INTEGER NOT NULL DEFAULT 0,
    avg_anonymity_score REAL NOT NULL DEFAULT 0,
    avg_entropy_bits REAL NOT NULL DEFAULT 0,
    avg_route_entropy REAL NOT NULL DEFAULT 0,
    total_active_routes INTEGER NOT NULL DEFAULT 0,
    rollup_json TEXT NOT NULL DEFAULT '{}',
    rolled_up_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cloud_anonymity_rollups_tenant ON cloud_anonymity_rollups(tenant_id);
CREATE INDEX IF NOT EXISTS idx_cloud_anonymity_rollups_rolled_up ON cloud_anonymity_rollups(rolled_up_at);
