use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostedController {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub region_id: String,
    pub plan_tier: String,
    pub status: String,
    pub endpoint_url: Option<String>,
    pub version: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningJob {
    pub id: String,
    pub tenant_id: String,
    pub job_type: String,
    pub status: String,
    pub controller_id: Option<String>,
    pub region_id: Option<String>,
    pub plan_tier: Option<String>,
    pub payload: serde_json::Value,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionRequest {
    pub tenant_id: String,
    pub name: String,
    pub region_id: String,
    pub plan_tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeRequest {
    pub tenant_id: String,
    pub controller_id: String,
    pub plan_tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRequest {
    pub tenant_id: String,
    pub controller_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreRequest {
    pub tenant_id: String,
    pub controller_id: String,
    pub snapshot_id: String,
}

pub struct HostedControllerManager {
    pool: DbPool,
}

impl HostedControllerManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn provision(
        &self,
        req: ProvisionRequest,
    ) -> Result<(HostedController, ProvisioningJob), DbError> {
        let controller_id = Uuid::new_v4().to_string();
        let now = now_iso();
        let endpoint = format!("https://{}.{}.wiresentinel.cloud", req.name, req.region_id);

        sqlx::query(
            "INSERT INTO hosted_controllers (id, tenant_id, name, region_id, plan_tier, status, endpoint_url, version, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'provisioning', ?, '0.1.0', ?, ?)",
        )
        .bind(&controller_id)
        .bind(&req.tenant_id)
        .bind(&req.name)
        .bind(&req.region_id)
        .bind(&req.plan_tier)
        .bind(&endpoint)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let job = self
            .create_job(
                &req.tenant_id,
                "provision",
                Some(&controller_id),
                Some(&req.region_id),
                Some(&req.plan_tier),
                serde_json::json!({ "name": req.name }),
            )
            .await?;

        self.complete_job(&job.id).await?;
        sqlx::query("UPDATE hosted_controllers SET status = 'active', updated_at = ? WHERE id = ?")
            .bind(now_iso())
            .bind(&controller_id)
            .execute(&self.pool)
            .await?;

        let controller = self.get_controller(&req.tenant_id, &controller_id).await?;
        Ok((controller, job))
    }

    pub async fn upgrade(&self, req: UpgradeRequest) -> Result<ProvisioningJob, DbError> {
        let _ = self
            .get_controller(&req.tenant_id, &req.controller_id)
            .await?;
        let job = self
            .create_job(
                &req.tenant_id,
                "upgrade",
                Some(&req.controller_id),
                None,
                Some(&req.plan_tier),
                serde_json::json!({}),
            )
            .await?;
        sqlx::query(
            "UPDATE hosted_controllers SET plan_tier = ?, status = 'upgrading', updated_at = ? WHERE id = ? AND tenant_id = ?",
        )
        .bind(&req.plan_tier)
        .bind(now_iso())
        .bind(&req.controller_id)
        .bind(&req.tenant_id)
        .execute(&self.pool)
        .await?;
        self.complete_job(&job.id).await?;
        sqlx::query("UPDATE hosted_controllers SET status = 'active', updated_at = ? WHERE id = ?")
            .bind(now_iso())
            .bind(&req.controller_id)
            .execute(&self.pool)
            .await?;
        Ok(job)
    }

    pub async fn backup(&self, req: BackupRequest) -> Result<(ProvisioningJob, String), DbError> {
        let controller = self
            .get_controller(&req.tenant_id, &req.controller_id)
            .await?;
        let snapshot_id = Uuid::new_v4().to_string();
        let storage_key = format!("{}/{}", req.controller_id, snapshot_id);
        let now = now_iso();

        sqlx::query(
            "INSERT INTO hosted_controller_snapshots (id, controller_id, tenant_id, snapshot_type, storage_key, size_bytes, created_at) VALUES (?, ?, ?, 'manual', ?, 0, ?)",
        )
        .bind(&snapshot_id)
        .bind(&req.controller_id)
        .bind(&req.tenant_id)
        .bind(&storage_key)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let job = self
            .create_job(
                &req.tenant_id,
                "backup",
                Some(&req.controller_id),
                Some(&controller.region_id),
                None,
                serde_json::json!({ "snapshot_id": snapshot_id }),
            )
            .await?;
        self.complete_job(&job.id).await?;
        Ok((job, snapshot_id))
    }

    pub async fn restore(&self, req: RestoreRequest) -> Result<ProvisioningJob, DbError> {
        let _ = self
            .get_controller(&req.tenant_id, &req.controller_id)
            .await?;
        let job = self
            .create_job(
                &req.tenant_id,
                "restore",
                Some(&req.controller_id),
                None,
                None,
                serde_json::json!({ "snapshot_id": req.snapshot_id }),
            )
            .await?;
        self.complete_job(&job.id).await?;
        Ok(job)
    }

    pub async fn list_controllers(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<HostedController>, DbError> {
        let rows: Vec<(String, String, String, String, String, String, Option<String>, Option<String>, String, String)> =
            sqlx::query_as(
                "SELECT id, tenant_id, name, region_id, plan_tier, status, endpoint_url, version, created_at, updated_at FROM hosted_controllers WHERE tenant_id = ? ORDER BY created_at DESC",
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
                    region_id,
                    plan_tier,
                    status,
                    endpoint_url,
                    version,
                    created_at,
                    updated_at,
                )| {
                    HostedController {
                        id,
                        tenant_id,
                        name,
                        region_id,
                        plan_tier,
                        status,
                        endpoint_url,
                        version,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    async fn get_controller(&self, tenant_id: &str, id: &str) -> Result<HostedController, DbError> {
        let row: Option<(String, String, String, String, String, String, Option<String>, Option<String>, String, String)> =
            sqlx::query_as(
                "SELECT id, tenant_id, name, region_id, plan_tier, status, endpoint_url, version, created_at, updated_at FROM hosted_controllers WHERE id = ? AND tenant_id = ?",
            )
            .bind(id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

        let (
            id,
            tenant_id,
            name,
            region_id,
            plan_tier,
            status,
            endpoint_url,
            version,
            created_at,
            updated_at,
        ) = row.ok_or_else(|| DbError::NotFound(format!("hosted controller {id}")))?;

        Ok(HostedController {
            id,
            tenant_id,
            name,
            region_id,
            plan_tier,
            status,
            endpoint_url,
            version,
            created_at,
            updated_at,
        })
    }

    async fn create_job(
        &self,
        tenant_id: &str,
        job_type: &str,
        controller_id: Option<&str>,
        region_id: Option<&str>,
        plan_tier: Option<&str>,
        payload: serde_json::Value,
    ) -> Result<ProvisioningJob, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let payload_str = payload.to_string();
        sqlx::query(
            "INSERT INTO provisioning_jobs (id, tenant_id, job_type, status, controller_id, region_id, plan_tier, payload, created_at, updated_at) VALUES (?, ?, ?, 'pending', ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(job_type)
        .bind(controller_id)
        .bind(region_id)
        .bind(plan_tier)
        .bind(&payload_str)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(ProvisioningJob {
            id,
            tenant_id: tenant_id.to_string(),
            job_type: job_type.into(),
            status: "pending".into(),
            controller_id: controller_id.map(str::to_string),
            region_id: region_id.map(str::to_string),
            plan_tier: plan_tier.map(str::to_string),
            payload,
            error_message: None,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    async fn complete_job(&self, job_id: &str) -> Result<(), DbError> {
        let now = now_iso();
        sqlx::query(
            "UPDATE provisioning_jobs SET status = 'completed', updated_at = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(job_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
