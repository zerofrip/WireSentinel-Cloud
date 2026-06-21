use cloud_core::{audit_sse_mutation, AuditWriteRequest};
use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsePolicyRecord {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub policy_kind: String,
    pub enabled: bool,
    pub rules: Vec<serde_json::Value>,
    pub default_action: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSsePolicyRequest {
    pub name: String,
    pub policy_kind: Option<String>,
    pub enabled: Option<bool>,
    pub rules: Option<Vec<serde_json::Value>>,
    pub default_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSsePolicyRequest {
    pub name: Option<String>,
    pub policy_kind: Option<String>,
    pub enabled: Option<bool>,
    pub rules: Option<Vec<serde_json::Value>>,
    pub default_action: Option<String>,
}

pub struct TenantSsePolicyService {
    pool: DbPool,
}

impl TenantSsePolicyService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, tenant_id: &str) -> Result<Vec<SsePolicyRecord>, DbError> {
        let rows: Vec<(String, String, String, String, i64, String, String, String, String)> =
            sqlx::query_as(
                "SELECT id, tenant_id, name, policy_kind, enabled, rules_json, default_action, created_at, updated_at
                 FROM sse_policies WHERE tenant_id = ? ORDER BY updated_at DESC",
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, tenant_id, name, policy_kind, enabled, rules_json, default_action, created_at, updated_at)| {
                    SsePolicyRecord {
                        id,
                        tenant_id,
                        name,
                        policy_kind,
                        enabled: enabled != 0,
                        rules: serde_json::from_str(&rules_json).unwrap_or_default(),
                        default_action,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn create(
        &self,
        tenant_id: &str,
        req: CreateSsePolicyRequest,
        actor: Option<&str>,
    ) -> Result<SsePolicyRecord, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let policy_kind = req.policy_kind.unwrap_or_else(|| "swg".into());
        let enabled = req.enabled.unwrap_or(true);
        let rules_json =
            serde_json::to_string(&req.rules.unwrap_or_default()).unwrap_or_else(|_| "[]".into());
        let default_action = req.default_action.unwrap_or_else(|| "block".into());

        sqlx::query(
            "INSERT INTO sse_policies (
                id, tenant_id, name, policy_kind, enabled, rules_json, default_action, content_json, created_at, updated_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, '{}', ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&policy_kind)
        .bind(enabled)
        .bind(&rules_json)
        .bind(&default_action)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        audit_sse_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-sse".into(),
                actor: actor.map(str::to_string),
                action: "sse.policy.create".into(),
                resource_type: Some("sse_policy".into()),
                resource_id: Some(id.clone()),
                details: serde_json::json!({ "name": req.name, "policy_kind": policy_kind }),
            },
        )
        .await?;

        Ok(SsePolicyRecord {
            id,
            tenant_id: tenant_id.to_string(),
            name: req.name,
            policy_kind,
            enabled,
            rules: serde_json::from_str(&rules_json).unwrap_or_default(),
            default_action,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update(
        &self,
        tenant_id: &str,
        policy_id: &str,
        req: UpdateSsePolicyRequest,
        actor: Option<&str>,
    ) -> Result<SsePolicyRecord, DbError> {
        let existing = self
            .list(tenant_id)
            .await?
            .into_iter()
            .find(|p| p.id == policy_id)
            .ok_or_else(|| DbError::NotFound(format!("sse policy {policy_id}")))?;

        let now = now_iso();
        let name = req.name.unwrap_or(existing.name);
        let policy_kind = req.policy_kind.unwrap_or(existing.policy_kind);
        let enabled = req.enabled.unwrap_or(existing.enabled);
        let rules = req.rules.unwrap_or(existing.rules);
        let default_action = req.default_action.unwrap_or(existing.default_action);
        let rules_json = serde_json::to_string(&rules).unwrap_or_else(|_| "[]".into());

        sqlx::query(
            "UPDATE sse_policies SET name = ?, policy_kind = ?, enabled = ?, rules_json = ?,
                    default_action = ?, updated_at = ?
             WHERE id = ? AND tenant_id = ?",
        )
        .bind(&name)
        .bind(&policy_kind)
        .bind(enabled)
        .bind(&rules_json)
        .bind(&default_action)
        .bind(&now)
        .bind(policy_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        audit_sse_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-sse".into(),
                actor: actor.map(str::to_string),
                action: "sse.policy.update".into(),
                resource_type: Some("sse_policy".into()),
                resource_id: Some(policy_id.to_string()),
                details: serde_json::json!({ "name": name }),
            },
        )
        .await?;

        Ok(SsePolicyRecord {
            id: policy_id.to_string(),
            tenant_id: tenant_id.to_string(),
            name,
            policy_kind,
            enabled,
            rules,
            default_action,
            created_at: existing.created_at,
            updated_at: now,
        })
    }
}
