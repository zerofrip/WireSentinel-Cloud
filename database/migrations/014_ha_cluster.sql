-- Phase 14-K: HA cluster nodes and leader election leases

CREATE TABLE IF NOT EXISTS cluster_nodes (
    id TEXT PRIMARY KEY NOT NULL,
    node_name TEXT NOT NULL UNIQUE,
    address TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'follower' CHECK (role IN ('leader', 'follower', 'candidate')),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'degraded', 'failed', 'draining')),
    last_heartbeat_at TEXT,
    lease_expires_at TEXT,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS cluster_leases (
    lease_key TEXT PRIMARY KEY NOT NULL,
    holder_node_id TEXT NOT NULL REFERENCES cluster_nodes(id),
    expires_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cluster_nodes_status ON cluster_nodes(status, last_heartbeat_at);
CREATE INDEX IF NOT EXISTS idx_cluster_leases_expires ON cluster_leases(expires_at);
