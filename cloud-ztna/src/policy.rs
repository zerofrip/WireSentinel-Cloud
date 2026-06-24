use cloud_core::{audit_ztna_mutation, AuditWriteRequest};
use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZtnaPolicyRecord {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub enabled: bool,
    pub min_trust_level: String,
    pub min_trust_score: u8,
    pub conditions: Vec<serde_json::Value>,
    pub default_action: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateZtnaPolicyRequest {
    pub name: String,
    pub enabled: Option<bool>,
    pub min_trust_level: Option<String>,
    pub min_trust_score: Option<u8>,
    pub conditions: Option<Vec<serde_json::Value>>,
    pub default_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateZtnaPolicyRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub min_trust_level: Option<String>,
    pub min_trust_score: Option<u8>,
    pub conditions: Option<Vec<serde_json::Value>>,
    pub default_action: Option<String>,
}

pub struct CloudZtnaPolicyService {
    pool: DbPool,
}

impl CloudZtnaPolicyService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, tenant_id: &str) -> Result<Vec<ZtnaPolicyRecord>, DbError> {
        let rows: Vec<(
            String,
            String,
            String,
            i64,
            String,
            i64,
            String,
            String,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, name, enabled, min_trust_level, min_trust_score, conditions_json,
                    default_action, content_json, created_at, updated_at
             FROM ztna_policies WHERE tenant_id = ? ORDER BY updated_at DESC",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    tenant_id,
                    name,
                    enabled,
                    min_trust_level,
                    min_trust_score,
                    conditions_json,
                    default_action,
                    _content_json,
                    created_at,
                    updated_at,
                )| {
                    ZtnaPolicyRecord {
                        id,
                        tenant_id,
                        name,
                        enabled: enabled != 0,
                        min_trust_level,
                        min_trust_score: min_trust_score.clamp(0, 100) as u8,
                        conditions: serde_json::from_str(&conditions_json).unwrap_or_default(),
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
        req: CreateZtnaPolicyRequest,
        actor: Option<&str>,
    ) -> Result<ZtnaPolicyRecord, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let enabled = req.enabled.unwrap_or(true);
        let min_trust_level = req.min_trust_level.unwrap_or_else(|| "medium".into());
        let min_trust_score = req.min_trust_score.unwrap_or(50);
        let conditions_json = serde_json::to_string(&req.conditions.unwrap_or_default())
            .unwrap_or_else(|_| "[]".into());
        let default_action = req.default_action.unwrap_or_else(|| "deny".into());

        sqlx::query(
            "INSERT INTO ztna_policies (
                id, tenant_id, name, enabled, min_trust_level, min_trust_score, conditions_json,
                default_action, content_json, created_at, updated_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, '{}', ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(&req.name)
        .bind(enabled)
        .bind(&min_trust_level)
        .bind(i64::from(min_trust_score))
        .bind(&conditions_json)
        .bind(&default_action)
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
                action: "ztna.policy.create".into(),
                resource_type: Some("ztna_policy".into()),
                resource_id: Some(id.clone()),
                details: serde_json::json!({ "name": req.name }),
            },
        )
        .await?;

        Ok(ZtnaPolicyRecord {
            id,
            tenant_id: tenant_id.to_string(),
            name: req.name,
            enabled,
            min_trust_level,
            min_trust_score,
            conditions: serde_json::from_str(&conditions_json).unwrap_or_default(),
            default_action,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update(
        &self,
        tenant_id: &str,
        policy_id: &str,
        req: UpdateZtnaPolicyRequest,
        actor: Option<&str>,
    ) -> Result<ZtnaPolicyRecord, DbError> {
        let existing = self
            .list(tenant_id)
            .await?
            .into_iter()
            .find(|p| p.id == policy_id)
            .ok_or_else(|| DbError::NotFound(format!("ztna policy {policy_id}")))?;

        let now = now_iso();
        let name = req.name.unwrap_or(existing.name);
        let enabled = req.enabled.unwrap_or(existing.enabled);
        let min_trust_level = req.min_trust_level.unwrap_or(existing.min_trust_level);
        let min_trust_score = req.min_trust_score.unwrap_or(existing.min_trust_score);
        let conditions = req.conditions.unwrap_or(existing.conditions);
        let default_action = req.default_action.unwrap_or(existing.default_action);
        let conditions_json = serde_json::to_string(&conditions).unwrap_or_else(|_| "[]".into());

        sqlx::query(
            "UPDATE ztna_policies SET name = ?, enabled = ?, min_trust_level = ?, min_trust_score = ?,
                    conditions_json = ?, default_action = ?, updated_at = ?
             WHERE id = ? AND tenant_id = ?",
        )
        .bind(&name)
        .bind(enabled)
        .bind(&min_trust_level)
        .bind(i64::from(min_trust_score))
        .bind(&conditions_json)
        .bind(&default_action)
        .bind(&now)
        .bind(policy_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        audit_ztna_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-ztna".into(),
                actor: actor.map(str::to_string),
                action: "ztna.policy.update".into(),
                resource_type: Some("ztna_policy".into()),
                resource_id: Some(policy_id.to_string()),
                details: serde_json::json!({ "name": name }),
            },
        )
        .await?;

        Ok(ZtnaPolicyRecord {
            id: policy_id.to_string(),
            tenant_id: tenant_id.to_string(),
            name,
            enabled,
            min_trust_level,
            min_trust_score,
            conditions,
            default_action,
            created_at: existing.created_at,
            updated_at: now,
        })
    }

    pub async fn delete(
        &self,
        tenant_id: &str,
        policy_id: &str,
        actor: Option<&str>,
    ) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM ztna_policies WHERE id = ? AND tenant_id = ?")
            .bind(policy_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("ztna policy {policy_id}")));
        }

        audit_ztna_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-ztna".into(),
                actor: actor.map(str::to_string),
                action: "ztna.policy.delete".into(),
                resource_type: Some("ztna_policy".into()),
                resource_id: Some(policy_id.to_string()),
                details: serde_json::json!({}),
            },
        )
        .await?;

        Ok(())
    }
}
