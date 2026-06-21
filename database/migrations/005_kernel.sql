-- Cloud kernel fleet rollups from federated controllers (Phase 12-M)

CREATE TABLE IF NOT EXISTS cloud_kernel_rollups (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    controller_id TEXT,
    reporting_devices INTEGER NOT NULL DEFAULT 0,
    healthy_devices INTEGER NOT NULL DEFAULT 0,
    kernel_devices INTEGER NOT NULL DEFAULT 0,
    ndis_devices INTEGER NOT NULL DEFAULT 0,
    stub_devices INTEGER NOT NULL DEFAULT 0,
    total_active_routes INTEGER NOT NULL DEFAULT 0,
    classify_count INTEGER NOT NULL DEFAULT 0,
    packets_per_sec INTEGER NOT NULL DEFAULT 0,
    rollup_json TEXT NOT NULL DEFAULT '{}',
    rolled_up_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cloud_kernel_rollups_tenant ON cloud_kernel_rollups(tenant_id);
CREATE INDEX IF NOT EXISTS idx_cloud_kernel_rollups_rolled_up ON cloud_kernel_rollups(rolled_up_at);
