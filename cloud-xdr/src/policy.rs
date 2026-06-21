use cloud_core::{audit_xdr_mutation, AuditWriteRequest};
use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdrHuntRecord {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub query_kind: String,
    pub status: String,
    pub enabled: bool,
    pub query: serde_json::Value,
    pub results_count: i64,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateXdrHuntRequest {
    pub name: String,
    pub query_kind: Option<String>,
    pub status: Option<String>,
    pub enabled: Option<bool>,
    pub query: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateXdrHuntRequest {
    pub name: Option<String>,
    pub query_kind: Option<String>,
    pub status: Option<String>,
    pub enabled: Option<bool>,
    pub query: Option<serde_json::Value>,
}

pub struct TenantXdrPolicyService {
    pool: DbPool,
}

impl TenantXdrPolicyService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn list_hunts(&self, tenant_id: &str) -> Result<Vec<XdrHuntRecord>, DbError> {
        let rows: Vec<(
            String,
            String,
            String,
            String,
            String,
            i64,
            String,
            i64,
            Option<String>,
            Option<String>,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, name, query_kind, status, enabled, query_json, results_count,
                    started_at, completed_at, created_at, updated_at
             FROM tenant_xdr_hunts WHERE tenant_id = ? ORDER BY updated_at DESC",
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
                    query_kind,
                    status,
                    enabled,
                    query_json,
                    results_count,
                    started_at,
                    completed_at,
                    created_at,
                    updated_at,
                )| {
                    XdrHuntRecord {
                        id,
                        tenant_id,
                        name,
                        query_kind,
                        status,
                        enabled: enabled != 0,
                        query: serde_json::from_str(&query_json).unwrap_or(serde_json::json!({})),
                        results_count,
                        started_at,
                        completed_at,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn create_hunt(
        &self,
        tenant_id: &str,
        req: CreateXdrHuntRequest,
        actor: Option<&str>,
    ) -> Result<XdrHuntRecord, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let query_kind = req.query_kind.unwrap_or_else(|| "historical".into());
        let status = req.status.unwrap_or_else(|| "draft".into());
        let enabled = req.enabled.unwrap_or(true);
        let query_json =
            serde_json::to_string(&req.query.unwrap_or(serde_json::json!({}))).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO tenant_xdr_hunts (
                id, tenant_id, name, query_kind, status, enabled, query_json, results_count,
                started_at, completed_at, created_at, updated_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, NULL, NULL, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&query_kind)
        .bind(&status)
        .bind(enabled)
        .bind(&query_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        audit_xdr_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-xdr".into(),
                actor: actor.map(str::to_string),
                action: "xdr.hunt.create".into(),
                resource_type: Some("xdr_hunt".into()),
                resource_id: Some(id.clone()),
                details: serde_json::json!({ "name": req.name, "query_kind": query_kind }),
            },
        )
        .await?;

        Ok(XdrHuntRecord {
            id,
            tenant_id: tenant_id.to_string(),
            name: req.name,
            query_kind,
            status,
            enabled,
            query: serde_json::from_str(&query_json).unwrap_or(serde_json::json!({})),
            results_count: 0,
            started_at: None,
            completed_at: None,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update_hunt(
        &self,
        tenant_id: &str,
        hunt_id: &str,
        req: UpdateXdrHuntRequest,
        actor: Option<&str>,
    ) -> Result<XdrHuntRecord, DbError> {
        let existing = self
            .list_hunts(tenant_id)
            .await?
            .into_iter()
            .find(|h| h.id == hunt_id)
            .ok_or_else(|| DbError::NotFound(format!("xdr hunt {hunt_id}")))?;

        let now = now_iso();
        let name = req.name.unwrap_or(existing.name);
        let query_kind = req.query_kind.unwrap_or(existing.query_kind);
        let status = req.status.unwrap_or(existing.status);
        let enabled = req.enabled.unwrap_or(existing.enabled);
        let query = req.query.unwrap_or(existing.query);
        let query_json = serde_json::to_string(&query).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "UPDATE tenant_xdr_hunts SET name = ?, query_kind = ?, status = ?, enabled = ?,
                    query_json = ?, updated_at = ?
             WHERE id = ? AND tenant_id = ?",
        )
        .bind(&name)
        .bind(&query_kind)
        .bind(&status)
        .bind(enabled)
        .bind(&query_json)
        .bind(&now)
        .bind(hunt_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        audit_xdr_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-xdr".into(),
                actor: actor.map(str::to_string),
                action: "xdr.hunt.update".into(),
                resource_type: Some("xdr_hunt".into()),
                resource_id: Some(hunt_id.to_string()),
                details: serde_json::json!({ "name": name }),
            },
        )
        .await?;

        Ok(XdrHuntRecord {
            id: hunt_id.to_string(),
            tenant_id: tenant_id.to_string(),
            name,
            query_kind,
            status,
            enabled,
            query,
            results_count: existing.results_count,
            started_at: existing.started_at,
            completed_at: existing.completed_at,
            created_at: existing.created_at,
            updated_at: now,
        })
    }
}
