use crate::fleet::VpnGatewayCompatFleetMonitor;
use database::DbError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VpnGatewayCompatAnalyticsSummary {
    pub tenant_id: String,
    pub reporting_endpoints: i64,
    pub active_split_templates: i64,
    pub tcp_termination_rules: i64,
    pub handshake_proxy_active: i64,
    pub bypass_events: i64,
    pub fleet_health_score: f64,
    pub rollups_recorded: i64,
}

pub struct VpnGatewayCompatAnalyticsService {
    fleet: VpnGatewayCompatFleetMonitor,
}

impl VpnGatewayCompatAnalyticsService {
    pub fn new(fleet: VpnGatewayCompatFleetMonitor) -> Self {
        Self { fleet }
    }

    pub async fn analytics(
        &self,
        tenant_id: &str,
    ) -> Result<VpnGatewayCompatAnalyticsSummary, DbError> {
        let overview = self.fleet.fleet_overview(tenant_id).await?;
        let rollups_recorded = overview.rollups.len() as i64;

        Ok(VpnGatewayCompatAnalyticsSummary {
            tenant_id: tenant_id.to_string(),
            reporting_endpoints: overview.reporting_endpoints,
            active_split_templates: overview.active_split_templates,
            tcp_termination_rules: overview.tcp_termination_rules,
            handshake_proxy_active: overview.handshake_proxy_active,
            bypass_events: overview.bypass_events,
            fleet_health_score: overview.fleet_health_score,
            rollups_recorded,
        })
    }
}
