use cloud_core::{audit_vpn_gateway_compat_mutation, AuditWriteRequest};
use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnGatewayCompatSplitTemplatePolicyRecord {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub description: String,
    pub template_mode: String,
    pub enabled: bool,
    pub app_rules_count: i64,
    pub domain_rules_count: i64,
    pub content: serde_json::Value,
    pub synced_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVpnGatewayCompatSplitTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub template_mode: Option<String>,
    pub enabled: Option<bool>,
    pub app_rules_count: Option<i64>,
    pub domain_rules_count: Option<i64>,
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVpnGatewayCompatSplitTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub template_mode: Option<String>,
    pub enabled: Option<bool>,
    pub app_rules_count: Option<i64>,
    pub domain_rules_count: Option<i64>,
    pub content: Option<serde_json::Value>,
}

pub struct TenantVpnGatewayCompatPolicyService {
    pool: DbPool,
}

impl TenantVpnGatewayCompatPolicyService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn list_split_template_policies(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<VpnGatewayCompatSplitTemplatePolicyRecord>, DbError> {
        let rows: Vec<(
            String,
            String,
            String,
            String,
            String,
            i64,
            i64,
            i64,
            String,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, name, description, template_mode, enabled, app_rules_count,
                    domain_rules_count, content_json, synced_at, created_at, updated_at
             FROM tenant_wiresock_split_templates WHERE tenant_id = ? ORDER BY updated_at DESC",
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
                    description,
                    template_mode,
                    enabled,
                    app_rules_count,
                    domain_rules_count,
                    content_json,
                    synced_at,
                    created_at,
                    updated_at,
                )| {
                    VpnGatewayCompatSplitTemplatePolicyRecord {
                        id,
                        tenant_id,
                        name,
                        description,
                        template_mode,
                        enabled: enabled != 0,
                        app_rules_count,
                        domain_rules_count,
                        content: serde_json::from_str(&content_json)
                            .unwrap_or(serde_json::json!({})),
                        synced_at,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn create_split_template(
        &self,
        tenant_id: &str,
        req: CreateVpnGatewayCompatSplitTemplateRequest,
        actor: Option<&str>,
    ) -> Result<VpnGatewayCompatSplitTemplatePolicyRecord, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let name = req.name.unwrap_or_else(|| "Split tunnel template".into());
        let description = req.description.unwrap_or_default();
        let template_mode = req.template_mode.unwrap_or_else(|| "merge".into());
        let enabled = req.enabled.unwrap_or(true);
        let app_rules_count = req.app_rules_count.unwrap_or(0);
        let domain_rules_count = req.domain_rules_count.unwrap_or(0);
        let content_json = serde_json::to_string(&req.content.unwrap_or(serde_json::json!({})))
            .unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO tenant_wiresock_split_templates (
                id, tenant_id, controller_id, name, description, template_mode, enabled,
                app_rules_count, domain_rules_count, content_json, synced_at, created_at,
                updated_at
             ) VALUES (?, ?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(&name)
        .bind(&description)
        .bind(&template_mode)
        .bind(i64::from(enabled))
        .bind(app_rules_count)
        .bind(domain_rules_count)
        .bind(&content_json)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        audit_vpn_gateway_compat_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-vpn-gateway-compat".into(),
                actor: actor.map(str::to_string),
                action: "vpn_gateway_compat.split_template.create".into(),
                resource_type: Some("vpn_gateway_compat_split_template".into()),
                resource_id: Some(id.clone()),
                details: serde_json::json!({
                    "name": name,
                    "template_mode": template_mode,
                }),
            },
        )
        .await?;

        Ok(VpnGatewayCompatSplitTemplatePolicyRecord {
            id,
            tenant_id: tenant_id.to_string(),
            name,
            description,
            template_mode,
            enabled,
            app_rules_count,
            domain_rules_count,
            content: serde_json::from_str(&content_json).unwrap_or(serde_json::json!({})),
            synced_at: now.clone(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update_split_template(
        &self,
        tenant_id: &str,
        template_id: &str,
        req: UpdateVpnGatewayCompatSplitTemplateRequest,
        actor: Option<&str>,
    ) -> Result<VpnGatewayCompatSplitTemplatePolicyRecord, DbError> {
        let existing = self
            .list_split_template_policies(tenant_id)
            .await?
            .into_iter()
            .find(|p| p.id == template_id)
            .ok_or_else(|| {
                DbError::NotFound(format!("vpn gateway compat split template {template_id}"))
            })?;

        let now = now_iso();
        let name = req.name.unwrap_or(existing.name);
        let description = req.description.unwrap_or(existing.description);
        let template_mode = req.template_mode.unwrap_or(existing.template_mode);
        let enabled = req.enabled.unwrap_or(existing.enabled);
        let app_rules_count = req.app_rules_count.unwrap_or(existing.app_rules_count);
        let domain_rules_count = req
            .domain_rules_count
            .unwrap_or(existing.domain_rules_count);
        let content = req.content.unwrap_or(existing.content);
        let content_json = serde_json::to_string(&content).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "UPDATE tenant_wiresock_split_templates SET name = ?, description = ?, template_mode = ?,
                    enabled = ?, app_rules_count = ?, domain_rules_count = ?, content_json = ?,
                    synced_at = ?, updated_at = ?
             WHERE id = ? AND tenant_id = ?",
        )
        .bind(&name)
        .bind(&description)
        .bind(&template_mode)
        .bind(i64::from(enabled))
        .bind(app_rules_count)
        .bind(domain_rules_count)
        .bind(&content_json)
        .bind(&now)
        .bind(&now)
        .bind(template_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        audit_vpn_gateway_compat_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-vpn-gateway-compat".into(),
                actor: actor.map(str::to_string),
                action: "vpn_gateway_compat.split_template.update".into(),
                resource_type: Some("vpn_gateway_compat_split_template".into()),
                resource_id: Some(template_id.to_string()),
                details: serde_json::json!({
                    "name": name,
                    "template_mode": template_mode,
                    "enabled": enabled,
                }),
            },
        )
        .await?;

        Ok(VpnGatewayCompatSplitTemplatePolicyRecord {
            id: template_id.to_string(),
            tenant_id: tenant_id.to_string(),
            name,
            description,
            template_mode,
            enabled,
            app_rules_count,
            domain_rules_count,
            content,
            synced_at: now.clone(),
            created_at: existing.created_at,
            updated_at: now,
        })
    }
}
