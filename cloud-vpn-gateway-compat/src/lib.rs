mod analytics;
mod fleet;
mod policy;

pub use analytics::{VpnGatewayCompatAnalyticsService, VpnGatewayCompatAnalyticsSummary};
pub use fleet::{
    VpnGatewayCompatFleetMonitor, VpnGatewayCompatFleetOverview, VpnGatewayCompatFleetRollup,
    VpnGatewayCompatHandshakeProxyRecord, VpnGatewayCompatRollupPayload,
    VpnGatewayCompatSplitTemplateRecord, VpnGatewayCompatTcpTerminationRecord,
};
pub use policy::{
    CreateVpnGatewayCompatSplitTemplateRequest, TenantVpnGatewayCompatPolicyService,
    UpdateVpnGatewayCompatSplitTemplateRequest, VpnGatewayCompatSplitTemplatePolicyRecord,
};
