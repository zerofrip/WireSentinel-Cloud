# WireSentinel Cloud — SaaS Operations

Operational guide for running WireSentinel Cloud in production (Phase 14).

## Services

| Component | Crate | Responsibility |
|-----------|-------|----------------|
| API | `cloud-api` | HTTP routes, auth, tenant isolation |
| Billing | `cloud-billing` | Stripe checkout, invoices, webhooks |
| Metering | `cloud-metering` | Usage snapshots and aggregates |
| Quotas | `cloud-quotas` | Soft/hard limits per tenant resource |
| Regions | `cloud-regions` | Multi-region health probes |
| Recovery | `cloud-recovery` | DR plans and run tracking |
| Observability | `cloud-observability` | TelemetryPipeline (Prometheus/JSON/OTLP) |
| Logging | `cloud-logging` | `aggregated_logs` ingestion and search |
| HA | `cloud-ha` | Cluster nodes, DB lease leader election |

## Environment

```bash
DATABASE_URL=sqlite://./data/cloud.db?mode=rwc
WS_CLOUD_JWT_SECRET=<random-256-bit>
STRIPE_MOCK=1                    # dev/CI only
STRIPE_SECRET_KEY=sk_live_...    # production
STRIPE_WEBHOOK_SECRET=whsec_...
OTEL_ENABLED=1                   # enable OTLP exporter stub/live
OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
WS_BACKUP_DIR=./data/backups
```

## Health checks

- `GET /health` — liveness (public)
- `GET /api/v1/cloud/metrics` — tenant-scoped counters
- `GET /api/v1/observability/metrics` — telemetry pipeline snapshot
- `GET /api/v1/regions/health` — region probe results
- `GET /api/v1/ha/leader` — current API leader node

## HA operations

1. Register nodes: `POST /api/v1/ha/nodes`
2. Heartbeat + lease: `POST /api/v1/ha/nodes/{id}/heartbeat`
3. Monitor `NodeFailed` / `FailoverTriggered` events from heartbeat response
4. Failed nodes marked when heartbeat older than 30s

## Logging

- Ingest: `POST /api/v1/cloud/logs`
- List: `GET /api/v1/logs?limit=100`
- Search: `GET /api/v1/logs/search?q=error&level=warn`

## Quota enforcement

Hosted controller provisioning calls `enforce_controller_quota` before creating resources. Grace periods stored in `quota_grace_periods`.

## Security audit (14-O)

Billing mutations write to `audit_events`. Invalid Stripe webhook signatures emit `BillingSecurityViolation` and return HTTP 401.

## Runbook snippets

**Stripe webhook failures:** verify `STRIPE_WEBHOOK_SECRET`, check `billing_events` for `billing.security_violation`.

**Region degradation:** run region health probe; ingest manual health via `POST /api/v1/cloud/health`.

**Leader split-brain:** inspect `cluster_leases` table; only one holder per `cloud-api-leader` key.
