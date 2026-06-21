-- Phase 14-A: relax subscription plan check; seed billing_plans including enterprise_plus

-- SQLite cannot alter CHECK constraints; recreate subscriptions without plan CHECK.
CREATE TABLE subscriptions_new (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id),
    plan TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'cancelled', 'past_due')),
    seats INTEGER NOT NULL DEFAULT 1,
    expires_at TEXT,
    stripe_customer_id TEXT,
    stripe_subscription_id TEXT,
    created_at TEXT NOT NULL
);

INSERT INTO subscriptions_new (id, tenant_id, plan, status, seats, expires_at, stripe_customer_id, stripe_subscription_id, created_at)
SELECT id, tenant_id, plan, status, seats, expires_at, stripe_customer_id, stripe_subscription_id, created_at
FROM subscriptions;

DROP TABLE subscriptions;
ALTER TABLE subscriptions_new RENAME TO subscriptions;

CREATE INDEX IF NOT EXISTS idx_subscriptions_tenant ON subscriptions(tenant_id);

INSERT OR IGNORE INTO billing_plans (id, name, tier, price_cents, currency, limits_json, active, created_at) VALUES
    ('free', 'Free', 'free', 0, 'usd', '{"max_users":5,"max_teams":2,"max_controllers":1}', 1, datetime('now')),
    ('team', 'Team', 'team', 2900, 'usd', '{"max_users":50,"max_teams":20,"max_controllers":5}', 1, datetime('now')),
    ('enterprise', 'Enterprise', 'enterprise', 9900, 'usd', '{"max_users":10000,"max_teams":1000,"max_controllers":100}', 1, datetime('now')),
    ('enterprise_plus', 'Enterprise Plus', 'enterprise_plus', 19900, 'usd', '{"max_users":50000,"max_teams":5000,"max_controllers":500}', 1, datetime('now'));
