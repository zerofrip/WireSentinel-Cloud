use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymityRollupPayload {
    pub reporting_devices: i64,
    pub healthy_devices: i64,
    pub connected_devices: i64,
    pub federation_peers_total: i64,
    pub avg_anonymity_score: f64,
    pub avg_entropy_bits: f64,
    pub avg_route_entropy: f64,
    pub total_active_routes: i64,
    pub controllers: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymityFleetRollup {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub reporting_devices: i64,
    pub healthy_devices: i64,
    pub connected_devices: i64,
    pub federation_peers_total: i64,
    pub avg_anonymity_score: f64,
    pub avg_entropy_bits: f64,
    pub avg_route_entropy: f64,
    pub total_active_routes: i64,
    pub rollup: serde_json::Value,
    pub rolled_up_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymityFleetOverview {
    pub tenant_id: String,
    pub reporting_devices: i64,
    pub healthy_devices: i64,
    pub connected_devices: i64,
    pub federation_peers_total: i64,
    pub avg_anonymity_score: f64,
    pub total_active_routes: i64,
    pub controllers_reporting: i64,
    pub rollups: Vec<AnonymityFleetRollup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymityPrivacyAnalytics {
    pub tenant_id: String,
    pub avg_entropy_bits: f64,
    pub avg_route_entropy: f64,
    pub avg_anonymity_score: f64,
    pub federation_peers_total: i64,
    pub healthy_ratio: f64,
    pub rollups_recorded: i64,
}

pub struct AnonymityFleetMonitor {
    pool: DbPool,
}

impl AnonymityFleetMonitor {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn record_rollup(
        &self,
        tenant_id: &str,
        controller_id: Option<&str>,
        payload: &AnonymityRollupPayload,
    ) -> Result<AnonymityFleetRollup, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let rollup_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO cloud_anonymity_rollups (
                id, tenant_id, controller_id, reporting_devices, healthy_devices, connected_devices,
                federation_peers_total, avg_anonymity_score, avg_entropy_bits, avg_route_entropy,
                total_active_routes, rollup_json, rolled_up_at, created_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(controller_id)
        .bind(payload.reporting_devices)
        .bind(payload.healthy_devices)
        .bind(payload.connected_devices)
        .bind(payload.federation_peers_total)
        .bind(payload.avg_anonymity_score)
        .bind(payload.avg_entropy_bits)
        .bind(payload.avg_route_entropy)
        .bind(payload.total_active_routes)
        .bind(&rollup_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(AnonymityFleetRollup {
            id,
            tenant_id: tenant_id.to_string(),
            controller_id: controller_id.map(str::to_string),
            reporting_devices: payload.reporting_devices,
            healthy_devices: payload.healthy_devices,
            connected_devices: payload.connected_devices,
            federation_peers_total: payload.federation_peers_total,
            avg_anonymity_score: payload.avg_anonymity_score,
            avg_entropy_bits: payload.avg_entropy_bits,
            avg_route_entropy: payload.avg_route_entropy,
            total_active_routes: payload.total_active_routes,
            rollup: serde_json::from_str(&rollup_json).unwrap_or(serde_json::json!({})),
            rolled_up_at: now.clone(),
            created_at: now,
        })
    }

    pub async fn fleet_overview(&self, tenant_id: &str) -> Result<AnonymityFleetOverview, DbError> {
        let rollups = self.list_rollups(tenant_id, Some(50)).await?;
        let controllers_reporting = rollups
            .iter()
            .filter_map(|r| r.controller_id.as_deref())
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        let reporting_devices: i64 = rollups.iter().map(|r| r.reporting_devices).sum();
        let healthy_devices: i64 = rollups.iter().map(|r| r.healthy_devices).sum();
        let avg_anonymity_score = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.avg_anonymity_score).sum::<f64>() / rollups.len() as f64
        };

        Ok(AnonymityFleetOverview {
            tenant_id: tenant_id.to_string(),
            reporting_devices,
            healthy_devices,
            connected_devices: rollups.iter().map(|r| r.connected_devices).sum(),
            federation_peers_total: rollups.iter().map(|r| r.federation_peers_total).sum(),
            avg_anonymity_score,
            total_active_routes: rollups.iter().map(|r| r.total_active_routes).sum(),
            controllers_reporting,
            rollups,
        })
    }

    pub async fn privacy_analytics(&self, tenant_id: &str) -> Result<AnonymityPrivacyAnalytics, DbError> {
        let rollups = self.list_rollups(tenant_id, Some(100)).await?;
        let rollups_recorded = rollups.len() as i64;
        let reporting_devices: i64 = rollups.iter().map(|r| r.reporting_devices).sum();
        let healthy_devices: i64 = rollups.iter().map(|r| r.healthy_devices).sum();
        let avg_entropy_bits = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.avg_entropy_bits).sum::<f64>() / rollups.len() as f64
        };
        let avg_route_entropy = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.avg_route_entropy).sum::<f64>() / rollups.len() as f64
        };
        let avg_anonymity_score = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.avg_anonymity_score).sum::<f64>() / rollups.len() as f64
        };
        let federation_peers_total: i64 = rollups.iter().map(|r| r.federation_peers_total).sum();
        let healthy_ratio = if reporting_devices > 0 {
            healthy_devices as f64 / reporting_devices as f64
        } else {
            0.0
        };

        Ok(AnonymityPrivacyAnalytics {
            tenant_id: tenant_id.to_string(),
            avg_entropy_bits,
            avg_route_entropy,
            avg_anonymity_score,
            federation_peers_total,
            healthy_ratio,
            rollups_recorded,
        })
    }

    async fn list_rollups(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<AnonymityFleetRollup>, DbError> {
        let limit = limit.unwrap_or(50);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            i64,
            i64,
            i64,
            i64,
            f64,
            f64,
            f64,
            i64,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, reporting_devices, healthy_devices, connected_devices,
                    federation_peers_total, avg_anonymity_score, avg_entropy_bits, avg_route_entropy,
                    total_active_routes, rollup_json, rolled_up_at, created_at
             FROM cloud_anonymity_rollups WHERE tenant_id = ? ORDER BY rolled_up_at DESC LIMIT ?",
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
                    connected_devices,
                    federation_peers_total,
                    avg_anonymity_score,
                    avg_entropy_bits,
                    avg_route_entropy,
                    total_active_routes,
                    rollup_json,
                    rolled_up_at,
                    created_at,
                )| {
                    AnonymityFleetRollup {
                        id,
                        tenant_id,
                        controller_id,
                        reporting_devices,
                        healthy_devices,
                        connected_devices,
                        federation_peers_total,
                        avg_anonymity_score,
                        avg_entropy_bits,
                        avg_route_entropy,
                        total_active_routes,
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
    async fn anonymity_fleet_rollup_and_analytics() {
        let pool = setup("sqlite::memory:").await.expect("db");
        let tenant = TenantManager::new(pool.clone())
            .create(CreateTenantRequest {
                name: "Anonymity".into(),
                slug: "anonymity".into(),
            })
            .await
            .expect("tenant");
        let monitor = AnonymityFleetMonitor::new(pool);
        monitor
            .record_rollup(
                &tenant.id,
                Some("ctrl-1"),
                &AnonymityRollupPayload {
                    reporting_devices: 4,
                    healthy_devices: 3,
                    connected_devices: 4,
                    federation_peers_total: 6,
                    avg_anonymity_score: 82.0,
                    avg_entropy_bits: 120.0,
                    avg_route_entropy: 2.1,
                    total_active_routes: 7,
                    controllers: None,
                },
            )
            .await
            .expect("rollup");

        let overview = monitor.fleet_overview(&tenant.id).await.expect("overview");
        assert_eq!(overview.reporting_devices, 4);
        let analytics = monitor.privacy_analytics(&tenant.id).await.expect("analytics");
        assert_eq!(analytics.avg_entropy_bits, 120.0);
    }
}
