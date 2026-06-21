use crate::fleet::XdrFleetMonitor;
use database::DbError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct XdrAnalyticsSummary {
    pub tenant_id: String,
    pub total_incidents: i64,
    pub open_incidents: i64,
    pub critical_incidents: i64,
    pub total_detections: i64,
    pub mitre_techniques_detected: i64,
    pub mitre_coverage_pct: f64,
    pub avg_incident_mttr_hours: f64,
    pub fleet_threat_score: f64,
    pub rollups_recorded: i64,
}

pub struct XdrAnalyticsService {
    fleet: XdrFleetMonitor,
}

impl XdrAnalyticsService {
    pub fn new(fleet: XdrFleetMonitor) -> Self {
        Self { fleet }
    }

    pub async fn analytics(&self, tenant_id: &str) -> Result<XdrAnalyticsSummary, DbError> {
        let overview = self.fleet.fleet_overview(tenant_id).await?;
        let rollups_recorded = overview.rollups.len() as i64;

        Ok(XdrAnalyticsSummary {
            tenant_id: tenant_id.to_string(),
            total_incidents: overview.total_incidents,
            open_incidents: overview.open_incidents,
            critical_incidents: overview.critical_incidents,
            total_detections: overview.total_detections,
            mitre_techniques_detected: overview.mitre_techniques_detected,
            mitre_coverage_pct: overview.mitre_coverage_pct,
            avg_incident_mttr_hours: overview.avg_incident_mttr_hours,
            fleet_threat_score: overview.fleet_threat_score,
            rollups_recorded,
        })
    }
}
