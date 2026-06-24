use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: String,
    pub user_id: String,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct CloudSecurityPolicy {
    pub jwt_secret: String,
    pub token_ttl_hours: i64,
    pub bcrypt_cost: u32,
    pub require_https: bool,
    pub require_tenant_header: bool,
    pub billing_webhook_ips_allowlist: Vec<String>,
}

impl Default for CloudSecurityPolicy {
    fn default() -> Self {
        Self {
            jwt_secret: std::env::var("WS_CLOUD_JWT_SECRET")
                .unwrap_or_else(|_| "dev-insecure-cloud-secret-change-me".into()),
            token_ttl_hours: 24,
            bcrypt_cost: 12,
            require_https: false,
            require_tenant_header: true,
            billing_webhook_ips_allowlist: vec![],
        }
    }
}

impl CloudSecurityPolicy {
    pub fn validate_tenant_access(&self, ctx: &TenantContext, requested_tenant: &str) -> bool {
        ctx.tenant_id == requested_tenant
    }

    pub fn validate_billing_webhook(&self, signature: &str) -> Result<(), String> {
        if std::env::var("STRIPE_MOCK").ok().as_deref() == Some("1") {
            return Ok(());
        }
        if signature.is_empty() {
            return Err("missing Stripe-Signature header".into());
        }
        if !signature.contains("t=") || !signature.contains("v1=") {
            return Err("malformed Stripe-Signature header".into());
        }
        Ok(())
    }

    pub fn require_https_for_billing(&self) -> bool {
        self.require_https || std::env::var("WS_BILLING_REQUIRE_HTTPS").ok().as_deref() == Some("1")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditWriteRequest {
    pub tenant_id: String,
    pub source: String,
    pub actor: Option<String>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
}

pub async fn write_audit_event(pool: &DbPool, req: AuditWriteRequest) -> Result<(), DbError> {
    let id = Uuid::new_v4().to_string();
    let created_at = now_iso();
    let details = req.details.to_string();
    sqlx::query(
        "INSERT INTO audit_events (id, tenant_id, source, actor, action, resource_type, resource_id, details, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&req.tenant_id)
    .bind(&req.source)
    .bind(&req.actor)
    .bind(&req.action)
    .bind(&req.resource_type)
    .bind(&req.resource_id)
    .bind(&details)
    .bind(&created_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Audit helper for Phase 15 ZTNA identity/policy/resource mutations.
pub async fn audit_ztna_mutation(pool: &DbPool, req: AuditWriteRequest) -> Result<(), DbError> {
    write_audit_event(pool, req).await
}

/// Audit helper for Phase 16 SSE policy mutations.
pub async fn audit_sse_mutation(pool: &DbPool, req: AuditWriteRequest) -> Result<(), DbError> {
    write_audit_event(pool, req).await
}

/// Audit helper for Phase 17 XDR hunt/policy mutations.
pub async fn audit_xdr_mutation(pool: &DbPool, req: AuditWriteRequest) -> Result<(), DbError> {
    write_audit_event(pool, req).await
}

/// Audit helper for Phase 18 CNAPP posture/policy mutations.
pub async fn audit_cnapp_mutation(pool: &DbPool, req: AuditWriteRequest) -> Result<(), DbError> {
    write_audit_event(pool, req).await
}

/// Audit helper for Phase 19 AI security investigation/policy mutations.
pub async fn audit_ai_mutation(pool: &DbPool, req: AuditWriteRequest) -> Result<(), DbError> {
    write_audit_event(pool, req).await
}

/// Audit helper for Phase 18.5 WireSock split template/policy mutations.
pub async fn audit_vpn_gateway_compat_mutation(
    pool: &DbPool,
    req: AuditWriteRequest,
) -> Result<(), DbError> {
    write_audit_event(pool, req).await
}
