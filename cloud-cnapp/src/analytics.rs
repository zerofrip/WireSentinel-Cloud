use crate::fleet::CnappFleetMonitor;
use database::DbError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CnappAnalyticsSummary {
    pub tenant_id: String,
    pub posture_score: f64,
    pub compliance_pct: f64,
    pub open_vulnerabilities: i64,
    pub critical_vulnerabilities: i64,
    pub attack_paths_detected: i64,
    pub multi_cloud_providers: i64,
    pub fleet_risk_score: f64,
    pub rollups_recorded: i64,
}

pub struct CnappAnalyticsService {
    fleet: CnappFleetMonitor,
}

impl CnappAnalyticsService {
    pub fn new(fleet: CnappFleetMonitor) -> Self {
        Self { fleet }
    }

    pub async fn analytics(&self, tenant_id: &str) -> Result<CnappAnalyticsSummary, DbError> {
        let overview = self.fleet.fleet_overview(tenant_id).await?;
        let rollups_recorded = overview.rollups.len() as i64;

        Ok(CnappAnalyticsSummary {
            tenant_id: tenant_id.to_string(),
            posture_score: overview.posture_score,
            compliance_pct: overview.compliance_pct,
            open_vulnerabilities: overview.open_vulnerabilities,
            critical_vulnerabilities: overview.critical_vulnerabilities,
            attack_paths_detected: overview.attack_paths_detected,
            multi_cloud_providers: overview.multi_cloud_providers,
            fleet_risk_score: overview.fleet_risk_score,
            rollups_recorded,
        })
    }
}
