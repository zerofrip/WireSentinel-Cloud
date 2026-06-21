-- WireSentinel Cloud teams (002)

CREATE TABLE IF NOT EXISTS teams (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    organization_id TEXT REFERENCES organizations(id),
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS team_memberships (
    id TEXT PRIMARY KEY NOT NULL,
    team_id TEXT NOT NULL REFERENCES teams(id),
    user_id TEXT NOT NULL REFERENCES users(id),
    role TEXT NOT NULL CHECK (role IN ('owner', 'administrator', 'operator', 'viewer')),
    created_at TEXT NOT NULL,
    UNIQUE (team_id, user_id)
);

CREATE TABLE IF NOT EXISTS team_devices (
    id TEXT PRIMARY KEY NOT NULL,
    team_id TEXT NOT NULL REFERENCES teams(id),
    device_id TEXT NOT NULL,
    assigned_at TEXT NOT NULL,
    UNIQUE (team_id, device_id)
);

CREATE TABLE IF NOT EXISTS team_policies (
    id TEXT PRIMARY KEY NOT NULL,
    team_id TEXT NOT NULL REFERENCES teams(id),
    policy_id TEXT NOT NULL,
    assigned_at TEXT NOT NULL,
    UNIQUE (team_id, policy_id)
);

CREATE INDEX IF NOT EXISTS idx_teams_tenant ON teams(tenant_id);
CREATE INDEX IF NOT EXISTS idx_team_memberships_team ON team_memberships(team_id);
CREATE INDEX IF NOT EXISTS idx_team_devices_team ON team_devices(team_id);
