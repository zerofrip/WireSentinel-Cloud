use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelRollupPayload {
    pub reporting_devices: i64,
    pub healthy_devices: i64,
    pub kernel_devices: i64,
    pub ndis_devices: i64,
    pub stub_devices: i64,
    pub total_active_routes: i64,
    pub classify_count: i64,
    pub packets_per_sec: i64,
    pub controllers: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelFleetRollup {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub reporting_devices: i64,
    pub healthy_devices: i64,
    pub kernel_devices: i64,
    pub ndis_devices: i64,
    pub stub_devices: i64,
    pub total_active_routes: i64,
    pub classify_count: i64,
    pub packets_per_sec: i64,
    pub rollup: serde_json::Value,
    pub rolled_up_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelFleetOverview {
    pub tenant_id: String,
    pub reporting_devices: i64,
    pub healthy_devices: i64,
    pub kernel_devices: i64,
    pub ndis_devices: i64,
    pub stub_devices: i64,
    pub total_active_routes: i64,
    pub controllers_reporting: i64,
    pub rollups: Vec<KernelFleetRollup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelFleetStatistics {
    pub tenant_id: String,
    pub classify_count: i64,
    pub packets_per_sec: i64,
    pub avg_healthy_ratio: f64,
    pub kernel_adoption_ratio: f64,
    pub rollups_recorded: i64,
}

pub struct KernelFleetMonitor {
    pool: DbPool,
}

impl KernelFleetMonitor {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn record_rollup(
        &self,
        tenant_id: &str,
        controller_id: Option<&str>,
        payload: &KernelRollupPayload,
    ) -> Result<KernelFleetRollup, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let rollup_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO cloud_kernel_rollups (
                id, tenant_id, controller_id, reporting_devices, healthy_devices, kernel_devices,
                ndis_devices, stub_devices, total_active_routes, classify_count, packets_per_sec,
                rollup_json, rolled_up_at, created_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(controller_id)
        .bind(payload.reporting_devices)
        .bind(payload.healthy_devices)
        .bind(payload.kernel_devices)
        .bind(payload.ndis_devices)
        .bind(payload.stub_devices)
        .bind(payload.total_active_routes)
        .bind(payload.classify_count)
        .bind(payload.packets_per_sec)
        .bind(&rollup_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(KernelFleetRollup {
            id,
            tenant_id: tenant_id.to_string(),
            controller_id: controller_id.map(str::to_string),
            reporting_devices: payload.reporting_devices,
            healthy_devices: payload.healthy_devices,
            kernel_devices: payload.kernel_devices,
            ndis_devices: payload.ndis_devices,
            stub_devices: payload.stub_devices,
            total_active_routes: payload.total_active_routes,
            classify_count: payload.classify_count,
            packets_per_sec: payload.packets_per_sec,
            rollup: serde_json::from_str(&rollup_json).unwrap_or(serde_json::json!({})),
            rolled_up_at: now.clone(),
            created_at: now,
        })
    }

    pub async fn fleet_overview(&self, tenant_id: &str) -> Result<KernelFleetOverview, DbError> {
        let rollups = self.list_rollups(tenant_id, Some(50)).await?;
        let controllers_reporting = rollups
            .iter()
            .filter_map(|r| r.controller_id.as_deref())
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        Ok(KernelFleetOverview {
            tenant_id: tenant_id.to_string(),
            reporting_devices: rollups.iter().map(|r| r.reporting_devices).sum(),
            healthy_devices: rollups.iter().map(|r| r.healthy_devices).sum(),
            kernel_devices: rollups.iter().map(|r| r.kernel_devices).sum(),
            ndis_devices: rollups.iter().map(|r| r.ndis_devices).sum(),
            stub_devices: rollups.iter().map(|r| r.stub_devices).sum(),
            total_active_routes: rollups.iter().map(|r| r.total_active_routes).sum(),
            controllers_reporting,
            rollups,
        })
    }

    pub async fn statistics(&self, tenant_id: &str) -> Result<KernelFleetStatistics, DbError> {
        let rollups = self.list_rollups(tenant_id, Some(100)).await?;
        let rollups_recorded = rollups.len() as i64;
        let reporting_devices: i64 = rollups.iter().map(|r| r.reporting_devices).sum();
        let healthy_devices: i64 = rollups.iter().map(|r| r.healthy_devices).sum();
        let kernel_devices: i64 = rollups.iter().map(|r| r.kernel_devices).sum();
        let classify_count: i64 = rollups.iter().map(|r| r.classify_count).sum();
        let packets_per_sec: i64 = rollups.iter().map(|r| r.packets_per_sec).sum();

        let avg_healthy_ratio = if reporting_devices > 0 {
            healthy_devices as f64 / reporting_devices as f64
        } else {
            0.0
        };
        let kernel_adoption_ratio = if reporting_devices > 0 {
            kernel_devices as f64 / reporting_devices as f64
        } else {
            0.0
        };

        Ok(KernelFleetStatistics {
            tenant_id: tenant_id.to_string(),
            classify_count,
            packets_per_sec,
            avg_healthy_ratio,
            kernel_adoption_ratio,
            rollups_recorded,
        })
    }

    async fn list_rollups(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<KernelFleetRollup>, DbError> {
        let limit = limit.unwrap_or(50);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, reporting_devices, healthy_devices, kernel_devices,
                    ndis_devices, stub_devices, total_active_routes, classify_count, packets_per_sec,
                    rollup_json, rolled_up_at, created_at
             FROM cloud_kernel_rollups WHERE tenant_id = ? ORDER BY rolled_up_at DESC LIMIT ?",
        )
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    tenant_id,
                    controller_id,
                    reporting_devices,
                    healthy_devices,
                    kernel_devices,
                    ndis_devices,
                    stub_devices,
                    total_active_routes,
                    classify_count,
                    packets_per_sec,
                    rollup_json,
                    rolled_up_at,
                    created_at,
                )| {
                    KernelFleetRollup {
                        id,
                        tenant_id,
                        controller_id,
                        reporting_devices,
                        healthy_devices,
                        kernel_devices,
                        ndis_devices,
                        stub_devices,
                        total_active_routes,
                        classify_count,
                        packets_per_sec,
                        rollup: serde_json::from_str(&rollup_json)
                            .unwrap_or(serde_json::json!({})),
                        rolled_up_at,
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
    use crate::{CreateTenantRequest, TenantManager};
    use database::setup;

    #[tokio::test]
    async fn kernel_fleet_rollup_and_statistics() {
        let pool = setup("sqlite::memory:").await.expect("db");
        let tenant = TenantManager::new(pool.clone())
            .create(CreateTenantRequest {
                name: "Kernel".into(),
                slug: "kernel".into(),
            })
            .await
            .expect("tenant");
        let monitor = KernelFleetMonitor::new(pool);
        monitor
            .record_rollup(
                &tenant.id,
                Some("ctrl-1"),
                &KernelRollupPayload {
                    reporting_devices: 3,
                    healthy_devices: 2,
                    kernel_devices: 2,
                    ndis_devices: 1,
                    stub_devices: 0,
                    total_active_routes: 5,
                    classify_count: 1000,
                    packets_per_sec: 120,
                    controllers: None,
                },
            )
            .await
            .expect("rollup");

        let overview = monitor.fleet_overview(&tenant.id).await.expect("overview");
        assert_eq!(overview.reporting_devices, 3);
        let stats = monitor.statistics(&tenant.id).await.expect("stats");
        assert_eq!(stats.classify_count, 1000);
    }
}
