use crate::fleet::SseFleetMonitor;
use database::DbError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SseAnalyticsSummary {
    pub tenant_id: String,
    pub block_ratio: f64,
    pub avg_risk_score: f64,
    pub threat_count: i64,
    pub casb_incidents: i64,
    pub dlp_incidents: i64,
    pub ueba_alerts: i64,
    pub rollups_recorded: i64,
}

pub struct SseAnalyticsService {
    fleet: SseFleetMonitor,
}

impl SseAnalyticsService {
    pub fn new(fleet: SseFleetMonitor) -> Self {
        Self { fleet }
    }

    pub async fn analytics(&self, tenant_id: &str) -> Result<SseAnalyticsSummary, DbError> {
        let overview = self.fleet.fleet_overview(tenant_id).await?;
        let rollups_recorded = overview.rollups.len() as i64;
        let total = overview.swg_requests;
        let block_ratio = if total > 0 {
            overview.swg_blocked as f64 / total as f64
        } else {
            0.0
        };

        Ok(SseAnalyticsSummary {
            tenant_id: tenant_id.to_string(),
            block_ratio,
            avg_risk_score: overview.avg_risk_score,
            threat_count: overview.threat_count,
            casb_incidents: overview.casb_incidents,
            dlp_incidents: overview.dlp_incidents,
            ueba_alerts: overview.ueba_alerts,
            rollups_recorded,
        })
    }
}
