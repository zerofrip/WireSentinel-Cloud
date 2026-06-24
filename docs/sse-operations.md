# SSE Operations (Phase 16)

WireSentinel Cloud Phase 16 adds tenant-scoped Secure Service Edge (SSE) fleet monitoring alongside controller-side agent telemetry.

## Components

| Layer | Crate / service | Responsibility |
|-------|-----------------|----------------|
| Cloud API | `cloud-sse` | Fleet rollups, tenant SSE policies, analytics |
| Controller | `controller::SseManager` | Local policies, incidents, threats, risk, UEBA, telemetry |
| Agent | `agents::AgentClient` | `push_sse_telemetry`, `report_dlp_incident` |
| Events | `cloud-events` | `SseSecurityViolation` |

## Cloud API

All routes require JWT + `X-Tenant-Id`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/cloud/sse` | Fleet overview (rollups) |
| GET | `/api/v1/cloud/sse/analytics` | Block ratio, risk averages |
| GET/POST | `/api/v1/sse/policies` | Tenant SSE policy catalog |

SSE policy mutations are written to `audit_events` via `audit_sse_mutation`.

## Controller API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/sse/swg` | SWG summary and recent threats |
| GET | `/api/v1/sse/casb` | CASB incidents |
| GET | `/api/v1/sse/dlp` | DLP incidents |
| GET | `/api/v1/sse/risk` | Device risk scores |
| GET | `/api/v1/sse/ueba` | UEBA anomalies |
| POST | `/api/v1/agents/{id}/sse/telemetry` | Agent telemetry ingest |

## Database

Cloud migration `018_sse.sql` adds `sse_policies` and `cloud_sse_rollups`.

Controller migration `008_sse.sql` adds local SSE policy, incident, threat, risk, UEBA, and telemetry tables.

## Operations checklist

1. Define tenant SSE policies (SWG, CASB, DLP) in Cloud.
2. Ensure agents push SSE telemetry from endpoints protected by Phase 16-M core hooks.
3. Monitor `/cloud/sse/analytics` for rising block ratios and DLP/CASB incident counts.
4. Investigate `SseSecurityViolation` cloud events for high-severity blocks.

## DTOs

Shared Phase 16 DTOs are planned in `WireSentinel/shared-types/src/sse.rs`. Controller and Cloud integrations use compatible JSON shapes until the standalone `WireSentinel-SSE` crate is populated.
