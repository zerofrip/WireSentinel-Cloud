use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedController {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub endpoint_url: String,
    pub status: String,
    pub last_sync_at: Option<String>,
    pub last_health_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterControllerRequest {
    pub tenant_id: String,
    pub name: String,
    pub endpoint_url: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationEvent {
    pub event_type: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub controller_id: String,
    pub healthy: bool,
    pub message: String,
    pub checked_at: String,
}

pub struct FederationManager {
    pool: DbPool,
}

impl FederationManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn hash_api_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub async fn register_controller(
        &self,
        req: RegisterControllerRequest,
    ) -> Result<(FederatedController, FederationEvent), DbError> {
        cloud_quotas::QuotaManager::new(self.pool.clone())
            .enforce_controller_quota(&req.tenant_id)
            .await
            .map_err(|e| DbError::NotFound(e.to_string()))?;

        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        let api_key_hash = Self::hash_api_key(&req.api_key);

        sqlx::query(
            "INSERT INTO federated_controllers (id, tenant_id, name, endpoint_url, api_key_hash, status, created_at) VALUES (?, ?, ?, ?, ?, 'active', ?)",
        )
        .bind(&id)
        .bind(&req.tenant_id)
        .bind(&req.name)
        .bind(&req.endpoint_url)
        .bind(&api_key_hash)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        let controller = FederatedController {
            id: id.clone(),
            tenant_id: req.tenant_id.clone(),
            name: req.name,
            endpoint_url: req.endpoint_url,
            status: "active".into(),
            last_sync_at: None,
            last_health_at: None,
            created_at,
        };

        let event = FederationEvent {
            event_type: "controller.registered".into(),
            tenant_id: req.tenant_id,
            controller_id: Some(id),
            details: serde_json::json!({ "name": controller.name }),
        };

        Ok((controller, event))
    }

    pub async fn revoke(
        &self,
        tenant_id: &str,
        controller_id: &str,
    ) -> Result<(FederatedController, FederationEvent), DbError> {
        let result = sqlx::query(
            "UPDATE federated_controllers SET status = 'revoked' WHERE id = ? AND tenant_id = ?",
        )
        .bind(controller_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("controller {controller_id}")));
        }

        let controller = self.get(tenant_id, controller_id).await?;
        let event = FederationEvent {
            event_type: "controller.revoked".into(),
            tenant_id: tenant_id.to_string(),
            controller_id: Some(controller_id.to_string()),
            details: serde_json::json!({}),
        };
        Ok((controller, event))
    }

    pub async fn get(&self, tenant_id: &str, id: &str) -> Result<FederatedController, DbError> {
        let row: Option<(String, String, String, String, String, Option<String>, Option<String>, String)> =
            sqlx::query_as(
                "SELECT id, tenant_id, name, endpoint_url, status, last_sync_at, last_health_at, created_at FROM federated_controllers WHERE id = ? AND tenant_id = ?",
            )
            .bind(id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

        let (id, tenant_id, name, endpoint_url, status, last_sync_at, last_health_at, created_at) =
            row.ok_or_else(|| DbError::NotFound(format!("controller {id}")))?;

        Ok(FederatedController {
            id,
            tenant_id,
            name,
            endpoint_url,
            status,
            last_sync_at,
            last_health_at,
            created_at,
        })
    }

    pub async fn list(&self, tenant_id: &str) -> Result<Vec<FederatedController>, DbError> {
        let rows: Vec<(String, String, String, String, String, Option<String>, Option<String>, String)> =
            sqlx::query_as(
                "SELECT id, tenant_id, name, endpoint_url, status, last_sync_at, last_health_at, created_at FROM federated_controllers WHERE tenant_id = ? ORDER BY created_at DESC",
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, tenant_id, name, endpoint_url, status, last_sync_at, last_health_at, created_at)| {
                    FederatedController {
                        id,
                        tenant_id,
                        name,
                        endpoint_url,
                        status,
                        last_sync_at,
                        last_health_at,
                        created_at,
                    }
                },
            )
            .collect())
    }

    pub async fn sync(
        &self,
        tenant_id: &str,
        controller_id: &str,
    ) -> Result<FederationEvent, DbError> {
        let _ = self.get(tenant_id, controller_id).await?;
        let synced_at = now_iso();
        sqlx::query(
            "UPDATE federated_controllers SET last_sync_at = ? WHERE id = ? AND tenant_id = ?",
        )
        .bind(&synced_at)
        .bind(controller_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        Ok(FederationEvent {
            event_type: "controller.synced".into(),
            tenant_id: tenant_id.to_string(),
            controller_id: Some(controller_id.to_string()),
            details: serde_json::json!({ "synced_at": synced_at }),
        })
    }

    pub async fn health_check(
        &self,
        tenant_id: &str,
        controller_id: &str,
    ) -> Result<HealthCheckResult, DbError> {
        let controller = self.get(tenant_id, controller_id).await?;
        let checked_at = now_iso();
        let healthy = controller.status == "active";
        let message = if healthy {
            "Controller reachable (stub check)".into()
        } else {
            format!("Controller status: {}", controller.status)
        };

        let status = if healthy { "active" } else { "unhealthy" };
        sqlx::query(
            "UPDATE federated_controllers SET last_health_at = ?, status = ? WHERE id = ?",
        )
        .bind(&checked_at)
        .bind(status)
        .bind(controller_id)
        .execute(&self.pool)
        .await?;

        Ok(HealthCheckResult {
            controller_id: controller_id.to_string(),
            healthy,
            message,
            checked_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_core::{CreateTenantRequest, TenantManager};
    use database::setup;

    #[tokio::test]
    async fn register_and_list_controllers() {
        let pool = setup("sqlite::memory:").await.expect("db");
        let tenant = TenantManager::new(pool.clone())
            .create(CreateTenantRequest {
                name: "Fed".into(),
                slug: "fed".into(),
            })
            .await
            .expect("tenant");
        let mgr = FederationManager::new(pool);
        let (ctrl, event) = mgr
            .register_controller(RegisterControllerRequest {
                tenant_id: tenant.id.clone(),
                name: "ctrl-1".into(),
                endpoint_url: "https://ctrl.example".into(),
                api_key: "secret".into(),
            })
            .await
            .expect("register");
        assert_eq!(event.event_type, "controller.registered");
        let list = mgr.list(&tenant.id).await.expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, ctrl.id);
    }
}
