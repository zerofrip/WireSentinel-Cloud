use cloud_core::{audit_ztna_mutation, AuditWriteRequest};
use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProviderRecord {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub provider_kind: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIdentityProviderRequest {
    pub name: String,
    pub provider_kind: String,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertUserIdentityRequest {
    pub provider_id: String,
    pub subject: String,
    pub email: Option<String>,
    pub display_name: String,
}

pub struct TenantIdentityService {
    pool: DbPool,
}

impl TenantIdentityService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn list_providers(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<IdentityProviderRecord>, DbError> {
        let rows: Vec<(String, String, String, String, String, i64, String, String)> =
            sqlx::query_as(
                "SELECT id, tenant_id, name, provider_kind, config_json, enabled, created_at, updated_at
                 FROM identity_providers WHERE tenant_id = ? ORDER BY updated_at DESC",
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, tenant_id, name, provider_kind, config_json, enabled, created_at, updated_at)| {
                    IdentityProviderRecord {
                        id,
                        tenant_id,
                        name,
                        provider_kind,
                        config: serde_json::from_str(&config_json).unwrap_or(serde_json::json!({})),
                        enabled: enabled != 0,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn create_provider(
        &self,
        tenant_id: &str,
        req: CreateIdentityProviderRequest,
        actor: Option<&str>,
    ) -> Result<IdentityProviderRecord, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let config_json = serde_json::to_string(&req.config.unwrap_or(serde_json::json!({})))
            .unwrap_or_else(|_| "{}".into());
        let enabled = req.enabled.unwrap_or(true);

        sqlx::query(
            "INSERT INTO identity_providers (
                id, tenant_id, name, provider_kind, config_json, enabled, created_at, updated_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&req.provider_kind)
        .bind(&config_json)
        .bind(enabled)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        audit_ztna_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-ztna".into(),
                actor: actor.map(str::to_string),
                action: "identity.provider.create".into(),
                resource_type: Some("identity_provider".into()),
                resource_id: Some(id.clone()),
                details: serde_json::json!({ "name": req.name, "provider_kind": req.provider_kind }),
            },
        )
        .await?;

        Ok(IdentityProviderRecord {
            id,
            tenant_id: tenant_id.to_string(),
            name: req.name,
            provider_kind: req.provider_kind,
            config: serde_json::from_str(&config_json).unwrap_or(serde_json::json!({})),
            enabled,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn upsert_user_identity(
        &self,
        tenant_id: &str,
        req: UpsertUserIdentityRequest,
    ) -> Result<(), DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        sqlx::query(
            "INSERT INTO user_identities (
                id, tenant_id, provider_id, subject, email, display_name, authenticated_at, created_at, updated_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(tenant_id, provider_id, subject) DO UPDATE SET
               email = excluded.email,
               display_name = excluded.display_name,
               authenticated_at = excluded.authenticated_at,
               updated_at = excluded.updated_at",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(&req.provider_id)
        .bind(&req.subject)
        .bind(&req.email)
        .bind(&req.display_name)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
