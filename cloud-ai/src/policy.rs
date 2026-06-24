use cloud_core::{audit_ai_mutation, AuditWriteRequest};
use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInvestigationPolicyRecord {
    pub id: String,
    pub tenant_id: String,
    pub title: String,
    pub status: String,
    pub severity: String,
    pub category: String,
    pub model_name: Option<String>,
    pub agent_id: Option<String>,
    pub finding_count: i64,
    pub content: serde_json::Value,
    pub opened_at: String,
    pub resolved_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAiInvestigationRequest {
    pub title: Option<String>,
    pub status: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub model_name: Option<String>,
    pub agent_id: Option<String>,
    pub finding_count: Option<i64>,
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAiInvestigationRequest {
    pub title: Option<String>,
    pub status: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub model_name: Option<String>,
    pub agent_id: Option<String>,
    pub finding_count: Option<i64>,
    pub content: Option<serde_json::Value>,
}

pub struct TenantAiPolicyService {
    pool: DbPool,
}

impl TenantAiPolicyService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn list_investigation_policies(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<AiInvestigationPolicyRecord>, DbError> {
        let rows: Vec<(
            String,
            String,
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            i64,
            String,
            String,
            Option<String>,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, title, status, severity, category, model_name, agent_id,
                    finding_count, content_json, opened_at, resolved_at, created_at, updated_at
             FROM tenant_ai_investigations WHERE tenant_id = ? ORDER BY updated_at DESC",
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
                    title,
                    status,
                    severity,
                    category,
                    model_name,
                    agent_id,
                    finding_count,
                    content_json,
                    opened_at,
                    resolved_at,
                    created_at,
                    updated_at,
                )| {
                    AiInvestigationPolicyRecord {
                        id,
                        tenant_id,
                        title,
                        status,
                        severity,
                        category,
                        model_name,
                        agent_id,
                        finding_count,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        opened_at,
                        resolved_at,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn create_investigation(
        &self,
        tenant_id: &str,
        req: CreateAiInvestigationRequest,
        actor: Option<&str>,
    ) -> Result<AiInvestigationPolicyRecord, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let title = req
            .title
            .unwrap_or_else(|| "AI security investigation".into());
        let status = req.status.unwrap_or_else(|| "open".into());
        let severity = req.severity.unwrap_or_else(|| "medium".into());
        let category = req.category.unwrap_or_else(|| "general".into());
        let finding_count = req.finding_count.unwrap_or(0);
        let content_json = serde_json::to_string(&req.content.unwrap_or(serde_json::json!({})))
            .unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO tenant_ai_investigations (
                id, tenant_id, controller_id, title, status, severity, category, model_name,
                agent_id, finding_count, content_json, opened_at, resolved_at, created_at,
                updated_at
             ) VALUES (?, ?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(&title)
        .bind(&status)
        .bind(&severity)
        .bind(&category)
        .bind(&req.model_name)
        .bind(&req.agent_id)
        .bind(finding_count)
        .bind(&content_json)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        audit_ai_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-ai".into(),
                actor: actor.map(str::to_string),
                action: "ai.investigation.create".into(),
                resource_type: Some("ai_investigation".into()),
                resource_id: Some(id.clone()),
                details: serde_json::json!({
                    "title": title,
                    "category": category,
                    "model_name": req.model_name,
                }),
            },
        )
        .await?;

        Ok(AiInvestigationPolicyRecord {
            id,
            tenant_id: tenant_id.to_string(),
            title,
            status,
            severity,
            category,
            model_name: req.model_name,
            agent_id: req.agent_id,
            finding_count,
            content: serde_json::from_str(&content_json).unwrap_or(serde_json::json!({})),
            opened_at: now.clone(),
            resolved_at: None,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update_investigation(
        &self,
        tenant_id: &str,
        investigation_id: &str,
        req: UpdateAiInvestigationRequest,
        actor: Option<&str>,
    ) -> Result<AiInvestigationPolicyRecord, DbError> {
        let existing = self
            .list_investigation_policies(tenant_id)
            .await?
            .into_iter()
            .find(|p| p.id == investigation_id)
            .ok_or_else(|| DbError::NotFound(format!("ai investigation {investigation_id}")))?;

        let now = now_iso();
        let title = req.title.unwrap_or(existing.title);
        let status = req.status.unwrap_or(existing.status);
        let severity = req.severity.unwrap_or(existing.severity);
        let category = req.category.unwrap_or(existing.category);
        let model_name = req.model_name.or(existing.model_name);
        let agent_id = req.agent_id.or(existing.agent_id);
        let finding_count = req.finding_count.unwrap_or(existing.finding_count);
        let content = req.content.unwrap_or(existing.content);
        let content_json = serde_json::to_string(&content).unwrap_or_else(|_| "{}".into());
        let resolved_at = if status == "resolved" {
            Some(now.clone())
        } else {
            existing.resolved_at
        };

        sqlx::query(
            "UPDATE tenant_ai_investigations SET title = ?, status = ?, severity = ?, category = ?,
                    model_name = ?, agent_id = ?, finding_count = ?, content_json = ?,
                    resolved_at = ?, updated_at = ?
             WHERE id = ? AND tenant_id = ?",
        )
        .bind(&title)
        .bind(&status)
        .bind(&severity)
        .bind(&category)
        .bind(&model_name)
        .bind(&agent_id)
        .bind(finding_count)
        .bind(&content_json)
        .bind(&resolved_at)
        .bind(&now)
        .bind(investigation_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        audit_ai_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-ai".into(),
                actor: actor.map(str::to_string),
                action: "ai.investigation.update".into(),
                resource_type: Some("ai_investigation".into()),
                resource_id: Some(investigation_id.to_string()),
                details: serde_json::json!({ "status": status, "category": category }),
            },
        )
        .await?;

        Ok(AiInvestigationPolicyRecord {
            id: investigation_id.to_string(),
            tenant_id: tenant_id.to_string(),
            title,
            status,
            severity,
            category,
            model_name,
            agent_id,
            finding_count,
            content,
            opened_at: existing.opened_at,
            resolved_at,
            created_at: existing.created_at,
            updated_at: now,
        })
    }
}
