use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplianceStatus {
    Passed,
    Failed,
    Warning,
}

impl ComplianceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: String,
    pub tenant_id: String,
    pub check_type: String,
    pub status: String,
    pub summary: String,
    pub details: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResult {
    pub check_type: String,
    pub status: ComplianceStatus,
    pub summary: String,
    pub details: serde_json::Value,
}

pub struct ComplianceEngine {
    pool: DbPool,
}

impl ComplianceEngine {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn run_checks(&self, tenant_id: &str) -> Result<Vec<ComplianceReport>, DbError> {
        let checks = [
            self.check_devices(tenant_id).await?,
            self.check_policies(tenant_id).await?,
            self.check_encryption(tenant_id).await?,
            self.check_privacy(tenant_id).await?,
            self.check_kernel(tenant_id).await?,
            self.check_anonymity(tenant_id).await?,
        ];

        let mut reports = Vec::new();
        for check in checks {
            reports.push(self.save_report(tenant_id, check).await?);
        }
        Ok(reports)
    }

    async fn check_devices(&self, tenant_id: &str) -> Result<ComplianceCheckResult, DbError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM team_devices td JOIN teams t ON td.team_id = t.id WHERE t.tenant_id = ?",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let status = if count.0 > 0 {
            ComplianceStatus::Passed
        } else {
            ComplianceStatus::Warning
        };

        Ok(ComplianceCheckResult {
            check_type: "device".into(),
            status,
            summary: format!("{} device assignments tracked", count.0),
            details: serde_json::json!({ "device_assignments": count.0 }),
        })
    }

    async fn check_policies(&self, tenant_id: &str) -> Result<ComplianceCheckResult, DbError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM team_policies tp JOIN teams t ON tp.team_id = t.id WHERE t.tenant_id = ?",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let status = if count.0 > 0 {
            ComplianceStatus::Passed
        } else {
            ComplianceStatus::Warning
        };

        Ok(ComplianceCheckResult {
            check_type: "policy".into(),
            status,
            summary: format!("{} policy assignments", count.0),
            details: serde_json::json!({ "policy_assignments": count.0 }),
        })
    }

    async fn check_encryption(&self, tenant_id: &str) -> Result<ComplianceCheckResult, DbError> {
        let config: Option<(String,)> =
            sqlx::query_as("SELECT config FROM tenant_configs WHERE tenant_id = ?")
                .bind(tenant_id)
                .fetch_optional(&self.pool)
                .await?;

        let cfg: serde_json::Value = config
            .map(|(c,)| serde_json::from_str(&c).unwrap_or(serde_json::json!({})))
            .unwrap_or(serde_json::json!({}));

        let encryption_enabled = cfg
            .get("encryption")
            .and_then(|v| v.get("enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        Ok(ComplianceCheckResult {
            check_type: "encryption".into(),
            status: if encryption_enabled {
                ComplianceStatus::Passed
            } else {
                ComplianceStatus::Failed
            },
            summary: if encryption_enabled {
                "Encryption at rest enabled".into()
            } else {
                "Encryption at rest disabled".into()
            },
            details: serde_json::json!({ "encryption_enabled": encryption_enabled }),
        })
    }

    async fn check_privacy(&self, tenant_id: &str) -> Result<ComplianceCheckResult, DbError> {
        let audit_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM audit_events WHERE tenant_id = ?")
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(ComplianceCheckResult {
            check_type: "privacy".into(),
            status: ComplianceStatus::Passed,
            summary: format!("Audit trail contains {} events", audit_count.0),
            details: serde_json::json!({ "audit_events": audit_count.0 }),
        })
    }

    async fn check_kernel(&self, tenant_id: &str) -> Result<ComplianceCheckResult, DbError> {
        let latest: Option<(i64, i64, i64)> = sqlx::query_as(
            "SELECT reporting_devices, healthy_devices, kernel_devices
             FROM cloud_kernel_rollups WHERE tenant_id = ? ORDER BY rolled_up_at DESC LIMIT 1",
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        let (reporting, healthy, kernel_devices) = latest.unwrap_or((0, 0, 0));
        let status = if reporting > 0 && healthy > 0 {
            ComplianceStatus::Passed
        } else if reporting > 0 {
            ComplianceStatus::Warning
        } else {
            ComplianceStatus::Warning
        };

        Ok(ComplianceCheckResult {
            check_type: "kernel".into(),
            status,
            summary: format!(
                "{} of {} kernel-reporting devices healthy ({} kernel mode)",
                healthy, reporting, kernel_devices
            ),
            details: serde_json::json!({
                "reporting_devices": reporting,
                "healthy_devices": healthy,
                "kernel_devices": kernel_devices,
            }),
        })
    }

    async fn check_anonymity(&self, tenant_id: &str) -> Result<ComplianceCheckResult, DbError> {
        let latest: Option<(i64, i64, f64)> = sqlx::query_as(
            "SELECT reporting_devices, healthy_devices, avg_anonymity_score
             FROM cloud_anonymity_rollups WHERE tenant_id = ? ORDER BY rolled_up_at DESC LIMIT 1",
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        let (reporting, healthy, avg_score) = latest.unwrap_or((0, 0, 0.0));
        let status = if reporting > 0 && healthy > 0 && avg_score >= 50.0 {
            ComplianceStatus::Passed
        } else if reporting > 0 {
            ComplianceStatus::Warning
        } else {
            ComplianceStatus::Warning
        };

        Ok(ComplianceCheckResult {
            check_type: "anonymity".into(),
            status,
            summary: format!(
                "{} of {} anonymity-reporting devices healthy (avg score {:.0})",
                healthy, reporting, avg_score
            ),
            details: serde_json::json!({
                "reporting_devices": reporting,
                "healthy_devices": healthy,
                "avg_anonymity_score": avg_score,
            }),
        })
    }

    async fn save_report(
        &self,
        tenant_id: &str,
        check: ComplianceCheckResult,
    ) -> Result<ComplianceReport, DbError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        let details_str = serde_json::to_string(&check.details).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO compliance_reports (id, tenant_id, check_type, status, summary, details, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(&check.check_type)
        .bind(check.status.as_str())
        .bind(&check.summary)
        .bind(&details_str)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(ComplianceReport {
            id,
            tenant_id: tenant_id.to_string(),
            check_type: check.check_type,
            status: check.status.as_str().into(),
            summary: check.summary,
            details: check.details,
            created_at,
        })
    }

    pub async fn list(&self, tenant_id: &str) -> Result<Vec<ComplianceReport>, DbError> {
        let rows: Vec<(String, String, String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, tenant_id, check_type, status, summary, details, created_at FROM compliance_reports WHERE tenant_id = ? ORDER BY created_at DESC LIMIT 100",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, tenant_id, check_type, status, summary, details, created_at)| {
                    ComplianceReport {
                        id,
                        tenant_id,
                        check_type,
                        status,
                        summary,
                        details: serde_json::from_str(&details).unwrap_or(serde_json::json!({})),
                        created_at,
                    }
                },
            )
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_core::{CreateTenantRequest, TenantManager};
    use database::setup;

    #[tokio::test]
    async fn compliance_run_checks() {
        let pool = setup("sqlite::memory:").await.expect("db");
        let tenant = TenantManager::new(pool.clone())
            .create(CreateTenantRequest {
                name: "Test".into(),
                slug: "test".into(),
            })
            .await
            .expect("tenant");
        let engine = ComplianceEngine::new(pool);
        let reports = engine.run_checks(&tenant.id).await.expect("checks");
        assert_eq!(reports.len(), 6);
    }
}
