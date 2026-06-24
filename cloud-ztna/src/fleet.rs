use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZtnaRollupPayload {
    pub reporting_devices: i64,
    pub avg_trust_score: f64,
    pub allow_count: i64,
    pub deny_count: i64,
    pub challenge_count: i64,
    pub published_resources: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZtnaFleetRollup {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub reporting_devices: i64,
    pub avg_trust_score: f64,
    pub allow_count: i64,
    pub deny_count: i64,
    pub challenge_count: i64,
    pub published_resources: i64,
    pub rollup: serde_json::Value,
    pub rolled_up_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZtnaFleetOverview {
    pub tenant_id: String,
    pub reporting_devices: i64,
    pub avg_trust_score: f64,
    pub allow_count: i64,
    pub deny_count: i64,
    pub challenge_count: i64,
    pub published_resources: i64,
    pub controllers_reporting: i64,
    pub rollups: Vec<ZtnaFleetRollup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZtnaAnalyticsSummary {
    pub tenant_id: String,
    pub avg_trust_score: f64,
    pub allow_count: i64,
    pub deny_count: i64,
    pub challenge_count: i64,
    pub published_resources: i64,
    pub deny_ratio: f64,
    pub rollups_recorded: i64,
}

pub struct ZtnaFleetMonitor {
    pool: DbPool,
}

impl ZtnaFleetMonitor {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn record_rollup(
        &self,
        tenant_id: &str,
        controller_id: Option<&str>,
        payload: &ZtnaRollupPayload,
    ) -> Result<ZtnaFleetRollup, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let rollup_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO cloud_ztna_rollups (
                id, tenant_id, controller_id, reporting_devices, avg_trust_score,
                allow_count, deny_count, challenge_count, published_resources,
                rollup_json, rolled_up_at, created_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(controller_id)
        .bind(payload.reporting_devices)
        .bind(payload.avg_trust_score)
        .bind(payload.allow_count)
        .bind(payload.deny_count)
        .bind(payload.challenge_count)
        .bind(payload.published_resources)
        .bind(&rollup_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(ZtnaFleetRollup {
            id,
            tenant_id: tenant_id.to_string(),
            controller_id: controller_id.map(str::to_string),
            reporting_devices: payload.reporting_devices,
            avg_trust_score: payload.avg_trust_score,
            allow_count: payload.allow_count,
            deny_count: payload.deny_count,
            challenge_count: payload.challenge_count,
            published_resources: payload.published_resources,
            rollup: serde_json::from_str(&rollup_json).unwrap_or(serde_json::json!({})),
            rolled_up_at: now.clone(),
            created_at: now,
        })
    }

    pub async fn fleet_overview(&self, tenant_id: &str) -> Result<ZtnaFleetOverview, DbError> {
        let rollups = self.list_rollups(tenant_id, Some(50)).await?;
        let controllers_reporting = rollups
            .iter()
            .filter_map(|r| r.controller_id.as_deref())
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        let reporting_devices: i64 = rollups.iter().map(|r| r.reporting_devices).sum();
        let avg_trust_score = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.avg_trust_score).sum::<f64>() / rollups.len() as f64
        };

        Ok(ZtnaFleetOverview {
            tenant_id: tenant_id.to_string(),
            reporting_devices,
            avg_trust_score,
            allow_count: rollups.iter().map(|r| r.allow_count).sum(),
            deny_count: rollups.iter().map(|r| r.deny_count).sum(),
            challenge_count: rollups.iter().map(|r| r.challenge_count).sum(),
            published_resources: rollups.iter().map(|r| r.published_resources).sum(),
            controllers_reporting,
            rollups,
        })
    }

    pub async fn analytics(&self, tenant_id: &str) -> Result<ZtnaAnalyticsSummary, DbError> {
        let rollups = self.list_rollups(tenant_id, Some(100)).await?;
        let rollups_recorded = rollups.len() as i64;
        let allow_count: i64 = rollups.iter().map(|r| r.allow_count).sum();
        let deny_count: i64 = rollups.iter().map(|r| r.deny_count).sum();
        let challenge_count: i64 = rollups.iter().map(|r| r.challenge_count).sum();
        let total = allow_count + deny_count + challenge_count;
        let deny_ratio = if total > 0 {
            deny_count as f64 / total as f64
        } else {
            0.0
        };
        let avg_trust_score = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.avg_trust_score).sum::<f64>() / rollups.len() as f64
        };

        Ok(ZtnaAnalyticsSummary {
            tenant_id: tenant_id.to_string(),
            avg_trust_score,
            allow_count,
            deny_count,
            challenge_count,
            published_resources: rollups.iter().map(|r| r.published_resources).sum(),
            deny_ratio,
            rollups_recorded,
        })
    }

    async fn list_rollups(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<ZtnaFleetRollup>, DbError> {
        let limit = limit.unwrap_or(50);
        let rows: Vec<(
            String,
            String,
            Option<String>,
            i64,
            f64,
            i64,
            i64,
            i64,
            i64,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, reporting_devices, avg_trust_score,
                    allow_count, deny_count, challenge_count, published_resources,
                    rollup_json, rolled_up_at, created_at
             FROM cloud_ztna_rollups WHERE tenant_id = ? ORDER BY rolled_up_at DESC LIMIT ?",
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
                    avg_trust_score,
                    allow_count,
                    deny_count,
                    challenge_count,
                    published_resources,
                    rollup_json,
                    rolled_up_at,
                    created_at,
                )| {
                    ZtnaFleetRollup {
                        id,
                        tenant_id,
                        controller_id,
                        reporting_devices,
                        avg_trust_score,
                        allow_count,
                        deny_count,
                        challenge_count,
                        published_resources,
                        rollup: serde_json::from_str(&rollup_json).unwrap_or(serde_json::json!({})),
                        rolled_up_at,
                        created_at,
                    }
                },
            )
            .collect())
    }
}
