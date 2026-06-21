use crate::fleet::AiFleetMonitor;
use database::DbError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiAnalyticsSummary {
    pub tenant_id: String,
    pub reporting_agents: i64,
    pub open_investigations: i64,
    pub critical_risks: i64,
    pub total_correlations: i64,
    pub compliance_pct: f64,
    pub avg_risk_score: f64,
    pub prompt_injection_events: i64,
    pub data_exfiltration_events: i64,
    pub fleet_ai_risk_score: f64,
    pub rollups_recorded: i64,
}

pub struct AiAnalyticsService {
    fleet: AiFleetMonitor,
}

impl AiAnalyticsService {
    pub fn new(fleet: AiFleetMonitor) -> Self {
        Self { fleet }
    }

    pub async fn analytics(&self, tenant_id: &str) -> Result<AiAnalyticsSummary, DbError> {
        let overview = self.fleet.fleet_overview(tenant_id).await?;
        let rollups_recorded = overview.rollups.len() as i64;

        Ok(AiAnalyticsSummary {
            tenant_id: tenant_id.to_string(),
            reporting_agents: overview.reporting_agents,
            open_investigations: overview.open_investigations,
            critical_risks: overview.critical_risks,
            total_correlations: overview.total_correlations,
            compliance_pct: overview.compliance_pct,
            avg_risk_score: overview.avg_risk_score,
            prompt_injection_events: overview.prompt_injection_events,
            data_exfiltration_events: overview.data_exfiltration_events,
            fleet_ai_risk_score: overview.fleet_ai_risk_score,
            rollups_recorded,
        })
    }
}
