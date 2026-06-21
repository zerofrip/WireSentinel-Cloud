# ZTNA Operations (Phase 15)

WireSentinel Cloud Phase 15 adds tenant-scoped Zero Trust Network Access (ZTNA) management alongside controller-side agent reporting.

## Components

| Layer | Crate / service | Responsibility |
|-------|-----------------|----------------|
| Cloud API | `cloud-ztna` | Fleet rollups, identity providers, policies, published resources |
| Controller | `controller::ZtnaManager` | Local policy/resource/trust state, agent heartbeats, connectors |
| Agent | `agents::AgentClient` | `push_ztna_heartbeat`, `register_connector` |
| Events | `cloud-events` | `ZtnaSecurityViolation`, `IdentitySecurityViolation` |

## Cloud API

All routes require JWT + `X-Tenant-Id`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/cloud/ztna` | Fleet overview (rollups) |
| GET | `/api/v1/cloud/ztna/analytics` | Deny ratio, trust averages |
| GET/POST | `/api/v1/identity/providers` | IdP catalog |
| GET/POST | `/api/v1/resources` | Published application catalog |
| PUT/DELETE | `/api/v1/resources/{id}` | Update or retire a publication |

ZTNA policy/resource mutations are written to `audit_events` via `audit_ztna_mutation`.

## Controller API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/ztna` | Dashboard summary |
| GET | `/api/v1/ztna/policies` | Local policy list |
| GET | `/api/v1/ztna/resources` | Published resources |
| GET | `/api/v1/ztna/trust` | Device trust records |
| GET | `/api/v1/ztna/analytics` | Heartbeat analytics |
| POST | `/api/v1/agents/{id}/ztna/heartbeat` | Agent status ingest |
| POST | `/api/v1/agents/{id}/ztna/connectors` | Connector registration |

## Database

Cloud migration `017_ztna_phase15.sql` adds identity, trust, policy, resource, segment, connector, decision, and rollup tables.

Controller migration `007_ztna.sql` adds local `ztna_policies`, `published_resources`, `device_trust`, `ztna_heartbeats`, and `connectors`.

## Operations checklist

1. Configure at least one identity provider per tenant.
2. Publish internal applications with an access policy reference.
3. Ensure agents push ZTNA heartbeats and register connectors.
4. Monitor `/cloud/ztna/analytics` for rising deny ratios.
5. Investigate `ZtnaSecurityViolation` / `IdentitySecurityViolation` cloud events.

## DTOs

Shared Phase 15 DTOs live in `WireSentinel/shared-types/src/phase15.rs`. Controller and Cloud integrations use compatible JSON shapes; WireSentinel-ZTNA crate DTO wiring is deferred until the standalone crate is populated.
