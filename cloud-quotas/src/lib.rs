use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuota {
    pub tenant_id: String,
    pub resource: String,
    pub soft_limit: f64,
    pub hard_limit: f64,
    pub current_usage: f64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetQuotaRequest {
    pub resource: String,
    pub soft_limit: f64,
    pub hard_limit: f64,
}

#[derive(Debug, Error)]
pub enum QuotaError {
    #[error("quota exceeded: {0}")]
    Exceeded(String),
    #[error("database error: {0}")]
    Db(#[from] DbError),
}

pub struct QuotaManager {
    pool: DbPool,
}

impl QuotaManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn get_quotas(&self, tenant_id: &str) -> Result<Vec<TenantQuota>, DbError> {
        let rows: Vec<(String, String, f64, f64, f64, String)> = sqlx::query_as(
            "SELECT tenant_id, resource, soft_limit, hard_limit, current_usage, updated_at FROM tenant_quotas WHERE tenant_id = ? ORDER BY resource",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            self.seed_defaults(tenant_id).await?;
            let rows: Vec<(String, String, f64, f64, f64, String)> = sqlx::query_as(
                "SELECT tenant_id, resource, soft_limit, hard_limit, current_usage, updated_at FROM tenant_quotas WHERE tenant_id = ? ORDER BY resource",
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;
            return Ok(rows
                .into_iter()
                .map(
                    |(tenant_id, resource, soft_limit, hard_limit, current_usage, updated_at)| {
                        TenantQuota {
                            tenant_id,
                            resource,
                            soft_limit,
                            hard_limit,
                            current_usage,
                            updated_at,
                        }
                    },
                )
                .collect());
        }

        Ok(rows
            .into_iter()
            .map(
                |(tenant_id, resource, soft_limit, hard_limit, current_usage, updated_at)| {
                    TenantQuota {
                        tenant_id,
                        resource,
                        soft_limit,
                        hard_limit,
                        current_usage,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn set_quota(
        &self,
        tenant_id: &str,
        req: SetQuotaRequest,
    ) -> Result<TenantQuota, DbError> {
        let updated_at = now_iso();
        sqlx::query(
            "INSERT INTO tenant_quotas (tenant_id, resource, soft_limit, hard_limit, current_usage, updated_at) VALUES (?, ?, ?, ?, 0, ?) \
             ON CONFLICT(tenant_id, resource) DO UPDATE SET soft_limit = excluded.soft_limit, hard_limit = excluded.hard_limit, updated_at = excluded.updated_at",
        )
        .bind(tenant_id)
        .bind(&req.resource)
        .bind(req.soft_limit)
        .bind(req.hard_limit)
        .bind(&updated_at)
        .execute(&self.pool)
        .await?;

        Ok(TenantQuota {
            tenant_id: tenant_id.to_string(),
            resource: req.resource,
            soft_limit: req.soft_limit,
            hard_limit: req.hard_limit,
            current_usage: 0.0,
            updated_at,
        })
    }

    pub async fn enforce_controller_quota(&self, tenant_id: &str) -> Result<(), QuotaError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM hosted_controllers WHERE tenant_id = ? AND status != 'terminated'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        let quota: Option<(f64, f64)> = sqlx::query_as(
            "SELECT soft_limit, hard_limit FROM tenant_quotas WHERE tenant_id = ? AND resource = 'controllers'",
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        if let Some((_, hard)) = quota {
            if count.0 as f64 >= hard {
                return Err(QuotaError::Exceeded(format!(
                    "controller quota exceeded ({}/{hard})",
                    count.0
                )));
            }
        }
        Ok(())
    }

    async fn seed_defaults(&self, tenant_id: &str) -> Result<(), DbError> {
        for (resource, soft, hard) in [
            ("teams", 5.0, 10.0),
            ("controllers", 3.0, 5.0),
            ("bandwidth_gb", 100.0, 200.0),
        ] {
            self.set_quota(
                tenant_id,
                SetQuotaRequest {
                    resource: resource.into(),
                    soft_limit: soft,
                    hard_limit: hard,
                },
            )
            .await?;
        }
        Ok(())
    }

    pub async fn record_usage(
        &self,
        tenant_id: &str,
        resource: &str,
        usage: f64,
    ) -> Result<(), DbError> {
        let updated_at = now_iso();
        sqlx::query(
            "UPDATE tenant_quotas SET current_usage = ?, updated_at = ? WHERE tenant_id = ? AND resource = ?",
        )
        .bind(usage)
        .bind(&updated_at)
        .bind(tenant_id)
        .bind(resource)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
