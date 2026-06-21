use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub plan_type: String,
    pub target_region_id: Option<String>,
    pub steps: Vec<serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRun {
    pub id: String,
    pub plan_id: String,
    pub tenant_id: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub details: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecoveryPlanRequest {
    pub tenant_id: String,
    pub name: String,
    pub plan_type: Option<String>,
    pub target_region_id: Option<String>,
    pub steps: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecoveryRequest {
    pub tenant_id: String,
    pub plan_id: String,
}

pub struct DisasterRecoveryManager {
    pool: DbPool,
}

impl DisasterRecoveryManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create_plan(&self, req: CreateRecoveryPlanRequest) -> Result<RecoveryPlan, DbError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        let steps = req.steps.unwrap_or_default();
        let steps_json = serde_json::to_string(&steps).unwrap_or_else(|_| "[]".into());
        let plan_type = req.plan_type.unwrap_or_else(|| "failover".into());

        sqlx::query(
            "INSERT INTO recovery_plans (id, tenant_id, name, plan_type, target_region_id, steps_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&req.tenant_id)
        .bind(&req.name)
        .bind(&plan_type)
        .bind(&req.target_region_id)
        .bind(&steps_json)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(RecoveryPlan {
            id,
            tenant_id: req.tenant_id,
            name: req.name,
            plan_type,
            target_region_id: req.target_region_id,
            steps,
            created_at,
        })
    }

    pub async fn run_recovery(&self, req: RunRecoveryRequest) -> Result<RecoveryRun, DbError> {
        let plan = self.get_plan(&req.tenant_id, &req.plan_id).await?;
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        let started_at = now_iso();

        sqlx::query(
            "INSERT INTO recovery_runs (id, plan_id, tenant_id, status, started_at, details, created_at) VALUES (?, ?, ?, 'running', ?, '{}', ?)",
        )
        .bind(&id)
        .bind(&req.plan_id)
        .bind(&req.tenant_id)
        .bind(&started_at)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        let completed_at = now_iso();
        let details = serde_json::json!({
            "plan_name": plan.name,
            "target_region": plan.target_region_id,
            "steps_executed": plan.steps.len(),
        });
        sqlx::query(
            "UPDATE recovery_runs SET status = 'completed', completed_at = ?, details = ? WHERE id = ?",
        )
        .bind(&completed_at)
        .bind(details.to_string())
        .bind(&id)
        .execute(&self.pool)
        .await?;

        Ok(RecoveryRun {
            id,
            plan_id: req.plan_id,
            tenant_id: req.tenant_id,
            status: "completed".into(),
            started_at: Some(started_at),
            completed_at: Some(completed_at),
            details,
            created_at,
        })
    }

    pub async fn list_runs(&self, tenant_id: &str) -> Result<Vec<RecoveryRun>, DbError> {
        let rows: Vec<(String, String, String, String, Option<String>, Option<String>, String, String)> =
            sqlx::query_as(
                "SELECT id, plan_id, tenant_id, status, started_at, completed_at, details, created_at FROM recovery_runs WHERE tenant_id = ? ORDER BY created_at DESC",
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, plan_id, tenant_id, status, started_at, completed_at, details, created_at)| {
                    RecoveryRun {
                        id,
                        plan_id,
                        tenant_id,
                        status,
                        started_at,
                        completed_at,
                        details: serde_json::from_str(&details).unwrap_or(serde_json::json!({})),
                        created_at,
                    }
                },
            )
            .collect())
    }

    async fn get_plan(&self, tenant_id: &str, plan_id: &str) -> Result<RecoveryPlan, DbError> {
        let row: Option<(String, String, String, String, Option<String>, String, String)> =
            sqlx::query_as(
                "SELECT id, tenant_id, name, plan_type, target_region_id, steps_json, created_at FROM recovery_plans WHERE id = ? AND tenant_id = ?",
            )
            .bind(plan_id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

        let (id, tenant_id, name, plan_type, target_region_id, steps_json, created_at) =
            row.ok_or_else(|| DbError::NotFound(format!("recovery plan {plan_id}")))?;

        Ok(RecoveryPlan {
            id,
            tenant_id,
            name,
            plan_type,
            target_region_id,
            steps: serde_json::from_str(&steps_json).unwrap_or_default(),
            created_at,
        })
    }
}
