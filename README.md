# WireSentinel Cloud

Multi-tenant cloud control plane for WireSentinel — tenant management, team RBAC, federation with edge controllers, bidirectional sync, billing quotas, and compliance reporting.

## Workspace crates

| Crate | Purpose |
|-------|---------|
| `database` | SQLite/Postgres migrations and connection pool |
| `cloud-core` | Tenants, organizations, teams, metrics, security policy |
| `federation` | Federated controller registration and health |
| `sync` | Cloud sync engine with conflict resolution |
| `auth` | JWT auth, OIDC/SAML stubs, team roles |
| `billing` | Subscriptions, plans, quota enforcement |
| `compliance` | Compliance checks and reports |
| `api` | `cloud-api` HTTP server (Axum) |

## Quick start

```bash
# Run API (default sqlite://./data/cloud.db, port 8090)
cargo run -p cloud-api

# Run tests
cargo test --workspace

# Web UI
cd web-ui && npm install && npm run dev
```

Default admin credentials (seeded on first run): `admin` / `admin`

## Environment

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite://./data/cloud.db?mode=rwc` | SQLite or Postgres URL |
| `BIND_ADDR` | `127.0.0.1:8090` | API listen address |
| `WS_CLOUD_JWT_SECRET` | dev secret | JWT signing key |
| `WS_CLOUD_OIDC_ISSUER` | — | Optional OIDC issuer |

## API overview

- `GET/POST /api/v1/tenants` — tenant management
- `GET/POST /api/v1/organizations` — organizations (scoped by `X-Tenant-Id`)
- `GET/POST /api/v1/teams` — teams and members
- `GET/POST /api/v1/federation/controllers` — federated controllers
- `GET/POST /api/v1/cloud/sync` — sync push/pull
- `GET/POST /api/v1/compliance` — compliance reports
- `GET /api/v1/cloud/metrics` — JSON or Prometheus (`Accept: text/plain`)
- `GET /api/v1/subscriptions`, `GET /api/v1/plans`
- `POST /api/v1/auth/login`, `GET /api/v1/auth/me`
- `GET /health`

Protected routes require `Authorization: Bearer <token>` and `X-Tenant-Id`.

## License

Apache-2.0
