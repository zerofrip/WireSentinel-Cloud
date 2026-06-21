use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseRollupPayload {
    pub reporting_devices: i64,
    pub swg_requests: i64,
    pub swg_blocked: i64,
    pub threat_count: i64,
    pub casb_incidents: i64,
    pub dlp_incidents: i64,
    pub avg_risk_score: f64,
    pub ueba_alerts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseFleetRollup {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub reporting_devices: i64,
    pub swg_requests: i64,
    pub swg_blocked: i64,
    pub threat_count: i64,
    pub casb_incidents: i64,
    pub dlp_incidents: i64,
    pub avg_risk_score: f64,
    pub ueba_alerts: i64,
    pub rollup: serde_json::Value,
    pub rolled_up_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseFleetOverview {
    pub tenant_id: String,
    pub reporting_devices: i64,
    pub swg_requests: i64,
    pub swg_blocked: i64,
    pub threat_count: i64,
    pub casb_incidents: i64,
    pub dlp_incidents: i64,
    pub avg_risk_score: f64,
    pub ueba_alerts: i64,
    pub controllers_reporting: i64,
    pub rollups: Vec<SseFleetRollup>,
}

pub struct SseFleetMonitor {
    pool: DbPool,
}

impl SseFleetMonitor {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn record_rollup(
        &self,
        tenant_id: &str,
        controller_id: Option<&str>,
        payload: &SseRollupPayload,
    ) -> Result<SseFleetRollup, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let rollup_json = serde_json::to_string(payload).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO cloud_sse_rollups (
                id, tenant_id, controller_id, reporting_devices, swg_requests, swg_blocked,
                threat_count, casb_incidents, dlp_incidents, avg_risk_score, ueba_alerts,
                rollup_json, rolled_up_at, created_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(controller_id)
        .bind(payload.reporting_devices)
        .bind(payload.swg_requests)
        .bind(payload.swg_blocked)
        .bind(payload.threat_count)
        .bind(payload.casb_incidents)
        .bind(payload.dlp_incidents)
        .bind(payload.avg_risk_score)
        .bind(payload.ueba_alerts)
        .bind(&rollup_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(SseFleetRollup {
            id,
            tenant_id: tenant_id.to_string(),
            controller_id: controller_id.map(str::to_string),
            reporting_devices: payload.reporting_devices,
            swg_requests: payload.swg_requests,
            swg_blocked: payload.swg_blocked,
            threat_count: payload.threat_count,
            casb_incidents: payload.casb_incidents,
            dlp_incidents: payload.dlp_incidents,
            avg_risk_score: payload.avg_risk_score,
            ueba_alerts: payload.ueba_alerts,
            rollup: serde_json::from_str(&rollup_json).unwrap_or(serde_json::json!({})),
            rolled_up_at: now.clone(),
            created_at: now,
        })
    }

    pub async fn fleet_overview(&self, tenant_id: &str) -> Result<SseFleetOverview, DbError> {
        let rollups = self.list_rollups(tenant_id, Some(50)).await?;
        let controllers_reporting = rollups
            .iter()
            .filter_map(|r| r.controller_id.as_deref())
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        let avg_risk_score = if rollups.is_empty() {
            0.0
        } else {
            rollups.iter().map(|r| r.avg_risk_score).sum::<f64>() / rollups.len() as f64
        };

        Ok(SseFleetOverview {
            tenant_id: tenant_id.to_string(),
            reporting_devices: rollups.iter().map(|r| r.reporting_devices).sum(),
            swg_requests: rollups.iter().map(|r| r.swg_requests).sum(),
            swg_blocked: rollups.iter().map(|r| r.swg_blocked).sum(),
            threat_count: rollups.iter().map(|r| r.threat_count).sum(),
            casb_incidents: rollups.iter().map(|r| r.casb_incidents).sum(),
            dlp_incidents: rollups.iter().map(|r| r.dlp_incidents).sum(),
            avg_risk_score,
            ueba_alerts: rollups.iter().map(|r| r.ueba_alerts).sum(),
            controllers_reporting,
            rollups,
        })
    }

    async fn list_rollups(
        &self,
        tenant_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<SseFleetRollup>, DbError> {
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
            f64,
            i64,
            String,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, controller_id, reporting_devices, swg_requests, swg_blocked,
                    threat_count, casb_incidents, dlp_incidents, avg_risk_score, ueba_alerts,
                    rollup_json, rolled_up_at, created_at
             FROM cloud_sse_rollups WHERE tenant_id = ? ORDER BY rolled_up_at DESC LIMIT ?",
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
                    swg_requests,
                    swg_blocked,
                    threat_count,
                    casb_incidents,
                    dlp_incidents,
                    avg_risk_score,
                    ueba_alerts,
                    rollup_json,
                    rolled_up_at,
                    created_at,
                )| {
                    SseFleetRollup {
                        id,
                        tenant_id,
                        controller_id,
                        reporting_devices,
                        swg_requests,
                        swg_blocked,
                        threat_count,
                        casb_incidents,
                        dlp_incidents,
                        avg_risk_score,
                        ueba_alerts,
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
