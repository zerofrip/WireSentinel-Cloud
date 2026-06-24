use auth::JwtAuthService;
use billing::SubscriptionManager;
use cloud_ai::{AiAnalyticsService, AiFleetMonitor, TenantAiPolicyService};
use cloud_api::{build_router, AppState};
use cloud_billing::BillingManager;
use cloud_cnapp::{CnappAnalyticsService, CnappFleetMonitor, TenantCnappPolicyService};
use cloud_core::{
    AnonymityFleetMonitor, CloudMetricsAggregator, CloudSecurityPolicy, CreateTenantRequest,
    KernelFleetMonitor, OrganizationManager, TeamManager, TenantManager,
};
use cloud_ha::HaManager;
use cloud_logging::LogAggregationService;
use cloud_metering::UsageMeteringService;
use cloud_observability::TelemetryPipeline;
use cloud_provisioning::HostedControllerManager;
use cloud_quotas::QuotaManager;
use cloud_recovery::DisasterRecoveryManager;
use cloud_regions::RegionManager;
use cloud_sse::{SseAnalyticsService, SseFleetMonitor, TenantSsePolicyService};
use cloud_storage::BackupStorageService;
use cloud_vpn_gateway_compat::{
    TenantVpnGatewayCompatPolicyService, VpnGatewayCompatAnalyticsService,
    VpnGatewayCompatFleetMonitor,
};
use cloud_xdr::{TenantXdrPolicyService, XdrAnalyticsService, XdrFleetMonitor};
use cloud_ztna::{
    CloudZtnaPolicyService, ResourcePublisher, TenantIdentityService, ZtnaFleetMonitor,
};
use compliance::ComplianceEngine;
use database::setup;
use federation::FederationManager;
use std::net::SocketAddr;
use std::sync::Arc;
use sync::CloudSyncEngine;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cloud_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://./data/cloud.db?mode=rwc".into());
    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8090".into());

    let _ = std::fs::create_dir_all("./data");

    let pool = setup(&database_url).await?;
    let policy = CloudSecurityPolicy::default();
    let tenants = Arc::new(TenantManager::new(pool.clone()));

    let default_tenant = ensure_default_tenant(tenants.as_ref()).await?;

    let auth = Arc::new(JwtAuthService::new(pool.clone(), policy));
    auth.ensure_default_admin(&default_tenant).await?;

    let billing = Arc::new(BillingManager::new(pool.clone()));
    let _ = billing.plans.seed_defaults().await;

    let state = AppState {
        pool: pool.clone(),
        auth,
        tenants: tenants.clone(),
        organizations: Arc::new(OrganizationManager::new(pool.clone())),
        teams: Arc::new(TeamManager::new(pool.clone())),
        federation: Arc::new(FederationManager::new(pool.clone())),
        sync: Arc::new(CloudSyncEngine::new(pool.clone())),
        compliance: Arc::new(ComplianceEngine::new(pool.clone())),
        metrics: Arc::new(CloudMetricsAggregator::new(pool.clone())),
        kernel_fleet: Arc::new(KernelFleetMonitor::new(pool.clone())),
        anonymity_fleet: Arc::new(AnonymityFleetMonitor::new(pool.clone())),
        ztna_fleet: Arc::new(ZtnaFleetMonitor::new(pool.clone())),
        ztna_identity: Arc::new(TenantIdentityService::new(pool.clone())),
        ztna_policies: Arc::new(CloudZtnaPolicyService::new(pool.clone())),
        ztna_resources: Arc::new(ResourcePublisher::new(pool.clone())),
        sse_fleet: Arc::new(SseFleetMonitor::new(pool.clone())),
        sse_policies: Arc::new(TenantSsePolicyService::new(pool.clone())),
        sse_analytics: Arc::new(SseAnalyticsService::new(SseFleetMonitor::new(pool.clone()))),
        xdr_fleet: Arc::new(XdrFleetMonitor::new(pool.clone())),
        xdr_policies: Arc::new(TenantXdrPolicyService::new(pool.clone())),
        xdr_analytics: Arc::new(XdrAnalyticsService::new(XdrFleetMonitor::new(pool.clone()))),
        cnapp_fleet: Arc::new(CnappFleetMonitor::new(pool.clone())),
        cnapp_policies: Arc::new(TenantCnappPolicyService::new(pool.clone())),
        cnapp_analytics: Arc::new(CnappAnalyticsService::new(CnappFleetMonitor::new(
            pool.clone(),
        ))),
        ai_fleet: Arc::new(AiFleetMonitor::new(pool.clone())),
        ai_policies: Arc::new(TenantAiPolicyService::new(pool.clone())),
        ai_analytics: Arc::new(AiAnalyticsService::new(AiFleetMonitor::new(pool.clone()))),
        vpn_gateway_compat_fleet: Arc::new(VpnGatewayCompatFleetMonitor::new(pool.clone())),
        vpn_gateway_compat_policies: Arc::new(TenantVpnGatewayCompatPolicyService::new(
            pool.clone(),
        )),
        vpn_gateway_compat_analytics: Arc::new(VpnGatewayCompatAnalyticsService::new(
            VpnGatewayCompatFleetMonitor::new(pool.clone()),
        )),
        subscriptions: Arc::new(SubscriptionManager::new(pool.clone())),
        billing,
        metering: Arc::new(UsageMeteringService::new(pool.clone())),
        quotas: Arc::new(QuotaManager::new(pool.clone())),
        provisioning: Arc::new(HostedControllerManager::new(pool.clone())),
        regions: Arc::new(RegionManager::new(pool.clone())),
        recovery: Arc::new(DisasterRecoveryManager::new(pool.clone())),
        backups: Arc::new(BackupStorageService::new(pool.clone())),
        logging: Arc::new(LogAggregationService::new(pool.clone())),
        observability: Arc::new(TelemetryPipeline::new(pool.clone())),
        ha: Arc::new(HaManager::new(pool)),
    };

    let app = build_router(state);
    let addr: SocketAddr = bind.parse()?;
    tracing::info!("WireSentinel Cloud listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ensure_default_tenant(tenants: &TenantManager) -> anyhow::Result<String> {
    let list = tenants.list().await?;
    if let Some(t) = list.first() {
        return Ok(t.id.clone());
    }
    let t = tenants
        .create(CreateTenantRequest {
            name: "Default".into(),
            slug: "default".into(),
        })
        .await?;
    Ok(t.id)
}
