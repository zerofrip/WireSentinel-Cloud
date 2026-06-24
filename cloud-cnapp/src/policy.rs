use cloud_core::{audit_cnapp_mutation, AuditWriteRequest};
use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CnappPosturePolicyRecord {
    pub id: String,
    pub tenant_id: String,
    pub cloud_provider: String,
    pub account_id: Option<String>,
    pub resource_kind: String,
    pub posture_score: f64,
    pub risk_level: String,
    pub findings_count: i64,
    pub content: serde_json::Value,
    pub assessed_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCnappPostureRequest {
    pub cloud_provider: Option<String>,
    pub account_id: Option<String>,
    pub resource_kind: Option<String>,
    pub posture_score: Option<f64>,
    pub risk_level: Option<String>,
    pub findings_count: Option<i64>,
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCnappPostureRequest {
    pub cloud_provider: Option<String>,
    pub account_id: Option<String>,
    pub resource_kind: Option<String>,
    pub posture_score: Option<f64>,
    pub risk_level: Option<String>,
    pub findings_count: Option<i64>,
    pub content: Option<serde_json::Value>,
}

pub struct TenantCnappPolicyService {
    pool: DbPool,
}

impl TenantCnappPolicyService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn list_posture_policies(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<CnappPosturePolicyRecord>, DbError> {
        let rows: Vec<(
            String,
            String,
            String,
            Option<String>,
            String,
            f64,
            String,
            i64,
            String,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, cloud_provider, account_id, resource_kind, posture_score,
                    risk_level, findings_count, content_json, assessed_at, created_at, updated_at
             FROM tenant_cnapp_posture WHERE tenant_id = ? ORDER BY updated_at DESC",
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
                    cloud_provider,
                    account_id,
                    resource_kind,
                    posture_score,
                    risk_level,
                    findings_count,
                    content_json,
                    assessed_at,
                    created_at,
                    updated_at,
                )| {
                    CnappPosturePolicyRecord {
                        id,
                        tenant_id,
                        cloud_provider,
                        account_id,
                        resource_kind,
                        posture_score,
                        risk_level,
                        findings_count,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        assessed_at,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn create_posture(
        &self,
        tenant_id: &str,
        req: CreateCnappPostureRequest,
        actor: Option<&str>,
    ) -> Result<CnappPosturePolicyRecord, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let cloud_provider = req.cloud_provider.unwrap_or_else(|| "aws".into());
        let resource_kind = req.resource_kind.unwrap_or_else(|| "account".into());
        let posture_score = req.posture_score.unwrap_or(0.0);
        let risk_level = req.risk_level.unwrap_or_else(|| "medium".into());
        let findings_count = req.findings_count.unwrap_or(0);
        let content_json = serde_json::to_string(&req.content.unwrap_or(serde_json::json!({})))
            .unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO tenant_cnapp_posture (
                id, tenant_id, controller_id, cloud_provider, account_id, resource_kind,
                posture_score, risk_level, findings_count, content_json, assessed_at,
                created_at, updated_at
             ) VALUES (?, ?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(&cloud_provider)
        .bind(&req.account_id)
        .bind(&resource_kind)
        .bind(posture_score)
        .bind(&risk_level)
        .bind(findings_count)
        .bind(&content_json)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        audit_cnapp_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-cnapp".into(),
                actor: actor.map(str::to_string),
                action: "cnapp.posture.create".into(),
                resource_type: Some("cnapp_posture".into()),
                resource_id: Some(id.clone()),
                details: serde_json::json!({
                    "cloud_provider": cloud_provider,
                    "account_id": req.account_id,
                }),
            },
        )
        .await?;

        Ok(CnappPosturePolicyRecord {
            id,
            tenant_id: tenant_id.to_string(),
            cloud_provider,
            account_id: req.account_id,
            resource_kind,
            posture_score,
            risk_level,
            findings_count,
            content: serde_json::from_str(&content_json).unwrap_or(serde_json::json!({})),
            assessed_at: now.clone(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update_posture(
        &self,
        tenant_id: &str,
        posture_id: &str,
        req: UpdateCnappPostureRequest,
        actor: Option<&str>,
    ) -> Result<CnappPosturePolicyRecord, DbError> {
        let existing = self
            .list_posture_policies(tenant_id)
            .await?
            .into_iter()
            .find(|p| p.id == posture_id)
            .ok_or_else(|| DbError::NotFound(format!("cnapp posture {posture_id}")))?;

        let now = now_iso();
        let cloud_provider = req.cloud_provider.unwrap_or(existing.cloud_provider);
        let account_id = req.account_id.or(existing.account_id);
        let resource_kind = req.resource_kind.unwrap_or(existing.resource_kind);
        let posture_score = req.posture_score.unwrap_or(existing.posture_score);
        let risk_level = req.risk_level.unwrap_or(existing.risk_level);
        let findings_count = req.findings_count.unwrap_or(existing.findings_count);
        let content = req.content.unwrap_or(existing.content);
        let content_json = serde_json::to_string(&content).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "UPDATE tenant_cnapp_posture SET cloud_provider = ?, account_id = ?, resource_kind = ?,
                    posture_score = ?, risk_level = ?, findings_count = ?, content_json = ?,
                    assessed_at = ?, updated_at = ?
             WHERE id = ? AND tenant_id = ?",
        )
        .bind(&cloud_provider)
        .bind(&account_id)
        .bind(&resource_kind)
        .bind(posture_score)
        .bind(&risk_level)
        .bind(findings_count)
        .bind(&content_json)
        .bind(&now)
        .bind(&now)
        .bind(posture_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        audit_cnapp_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-cnapp".into(),
                actor: actor.map(str::to_string),
                action: "cnapp.posture.update".into(),
                resource_type: Some("cnapp_posture".into()),
                resource_id: Some(posture_id.to_string()),
                details: serde_json::json!({ "cloud_provider": cloud_provider }),
            },
        )
        .await?;

        Ok(CnappPosturePolicyRecord {
            id: posture_id.to_string(),
            tenant_id: tenant_id.to_string(),
            cloud_provider,
            account_id,
            resource_kind,
            posture_score,
            risk_level,
            findings_count,
            content,
            assessed_at: now.clone(),
            created_at: existing.created_at,
            updated_at: now,
        })
    }
}
