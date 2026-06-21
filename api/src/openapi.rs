//! OpenAPI route catalog (Phase 11-M / 14). Served at `/api/v1/openapi.json`.

use serde::Serialize;

#[derive(Serialize)]
pub struct OpenApiDocument {
    pub openapi: &'static str,
    pub info: OpenApiInfo,
    pub paths: serde_json::Value,
    pub components: serde_json::Value,
}

#[derive(Serialize)]
pub struct OpenApiInfo {
    pub title: &'static str,
    pub version: &'static str,
    pub description: &'static str,
}

pub fn document() -> OpenApiDocument {
    OpenApiDocument {
        openapi: "3.1.0",
        info: OpenApiInfo {
            title: "WireSentinel Cloud API",
            version: "0.1.0",
            description: "Multi-tenant cloud platform for WireSentinel federation, sync, compliance, billing, observability, and HA.",
        },
        paths: serde_json::json!({
            "/api/v1/auth/login": { "post": { "summary": "Login", "tags": ["auth"] } },
            "/api/v1/tenants": { "get": { "summary": "List tenants" }, "post": { "summary": "Create tenant" } },
            "/api/v1/organizations": { "get": { "summary": "List organizations" }, "post": { "summary": "Create organization" } },
            "/api/v1/teams": { "get": { "summary": "List teams" }, "post": { "summary": "Create team" } },
            "/api/v1/federation/controllers": { "get": { "summary": "List federated controllers" }, "post": { "summary": "Register controller" } },
            "/api/v1/cloud/sync": { "get": { "summary": "Pull sync entities" }, "post": { "summary": "Bidirectional sync" } },
            "/api/v1/compliance": { "get": { "summary": "List compliance reports" }, "post": { "summary": "Run compliance checks" } },
            "/api/v1/cloud/metrics": { "get": { "summary": "Tenant metrics (JSON or Prometheus)" } },
            "/api/v1/observability/metrics": { "get": { "summary": "Telemetry pipeline metrics (JSON or Prometheus)" } },
            "/api/v1/logs": { "get": { "summary": "List aggregated logs" } },
            "/api/v1/logs/search": { "get": { "summary": "Search aggregated logs" } },
            "/api/v1/cloud/logs": { "post": { "summary": "Ingest log entry" } },
            "/api/v1/subscriptions": { "get": { "summary": "List subscriptions" }, "post": { "summary": "Create subscription" } },
            "/api/v1/plans": { "get": { "summary": "List billing plans" } },
            "/api/v1/billing/plans": { "get": { "summary": "List cloud billing plans" }, "post": { "summary": "Seed billing plans" } },
            "/api/v1/billing/subscription": { "get": { "summary": "Get billing subscription" }, "post": { "summary": "Create billing subscription" } },
            "/api/v1/billing/invoices": { "get": { "summary": "List invoices" } },
            "/api/v1/billing/checkout": { "post": { "summary": "Create Stripe checkout session" } },
            "/api/v1/billing/webhook": { "post": { "summary": "Stripe webhook receiver" } },
            "/api/v1/quotas": { "get": { "summary": "List tenant quotas" }, "put": { "summary": "Update tenant quotas" } },
            "/api/v1/regions": { "get": { "summary": "List cloud regions" } },
            "/api/v1/regions/health": { "get": { "summary": "Probe region health" } },
            "/api/v1/recovery/run": { "post": { "summary": "Run disaster recovery plan" } },
            "/api/v1/recovery/runs": { "get": { "summary": "List recovery runs" } },
            "/api/v1/ha/nodes": { "get": { "summary": "List HA cluster nodes" }, "post": { "summary": "Register HA node" } },
            "/api/v1/ha/nodes/{id}/heartbeat": { "post": { "summary": "Node heartbeat and leader election" } },
            "/api/v1/cloud/anonymity": { "get": { "summary": "Anonymity fleet overview" } },
            "/api/v1/cloud/ztna": { "get": { "summary": "ZTNA fleet overview" } },
            "/api/v1/cloud/ztna/analytics": { "get": { "summary": "ZTNA analytics" } },
            "/api/v1/cloud/sse": { "get": { "summary": "SSE fleet overview" } },
            "/api/v1/cloud/sse/analytics": { "get": { "summary": "SSE analytics" } },
            "/api/v1/cloud/xdr": { "get": { "summary": "XDR fleet overview" } },
            "/api/v1/cloud/xdr/analytics": { "get": { "summary": "XDR analytics" } },
            "/api/v1/cloud/xdr/incidents": { "get": { "summary": "XDR incidents" } },
            "/api/v1/cloud/xdr/detections": { "get": { "summary": "XDR detections" } },
            "/api/v1/cloud/xdr/mitre-coverage": { "get": { "summary": "XDR MITRE coverage" } },
            "/api/v1/cloud/cnapp": { "get": { "summary": "CNAPP fleet overview" } },
            "/api/v1/cloud/cnapp/posture": { "get": { "summary": "CNAPP posture" } },
            "/api/v1/cloud/cnapp/compliance": { "get": { "summary": "CNAPP compliance" } },
            "/api/v1/cloud/cnapp/vulnerabilities": { "get": { "summary": "CNAPP vulnerabilities" } },
            "/api/v1/cloud/cnapp/analytics": { "get": { "summary": "CNAPP analytics" } },
            "/api/v1/cloud/ai": { "get": { "summary": "AI security fleet overview" } },
            "/api/v1/cloud/ai/risk": { "get": { "summary": "AI risk assessments" } },
            "/api/v1/cloud/ai/reports": { "get": { "summary": "AI security reports" } },
            "/api/v1/cloud/ai/investigations": { "get": { "summary": "AI investigations" } },
            "/api/v1/cloud/ai/analytics": { "get": { "summary": "AI security analytics" } },
            "/api/v1/sse/policies": { "get": { "summary": "List SSE policies" }, "post": { "summary": "Create SSE policy" } },
            "/api/v1/identity/providers": { "get": { "summary": "List identity providers" }, "post": { "summary": "Create identity provider" } },
            "/api/v1/resources": { "get": { "summary": "List published resources" }, "post": { "summary": "Publish resource" } },
            "/api/v1/resources/{id}": { "put": { "summary": "Update published resource" }, "delete": { "summary": "Delete published resource" } },
            "/api/v1/ha/leader": { "get": { "summary": "Current cluster leader" } }
        }),
        components: serde_json::json!({
            "securitySchemes": {
                "bearerAuth": { "type": "http", "scheme": "bearer", "bearerFormat": "JWT" },
                "tenantHeader": { "type": "apiKey", "in": "header", "name": "X-Tenant-Id" }
            },
            "security": [{ "bearerAuth": [], "tenantHeader": [] }]
        }),
    }
}
