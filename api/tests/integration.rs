use auth::TeamRole;
use billing::{CreateSubscriptionRequest, Plan, SubscriptionManager};
use cloud_core::{
    AnonymityFleetMonitor, AnonymityRollupPayload, CloudMetricsAggregator, CloudSecurityPolicy,
    CreateOrganizationRequest, CreateTeamRequest, CreateTenantRequest, KernelFleetMonitor,
    KernelRollupPayload, OrganizationManager, TeamManager, TenantManager,
};
use cloud_api::{build_router, AppState};
use cloud_billing::BillingManager;
use cloud_metering::UsageMeteringService;
use cloud_provisioning::HostedControllerManager;
use cloud_quotas::QuotaManager;
use cloud_recovery::DisasterRecoveryManager;
use cloud_regions::RegionManager;
use cloud_ha::HaManager;
use cloud_logging::LogAggregationService;
use cloud_observability::TelemetryPipeline;
use cloud_storage::BackupStorageService;
use cloud_ztna::{CloudZtnaPolicyService, ResourcePublisher, TenantIdentityService, ZtnaFleetMonitor};
use cloud_sse::{SseAnalyticsService, SseFleetMonitor, TenantSsePolicyService};
use cloud_xdr::{XdrAnalyticsService, XdrFleetMonitor, TenantXdrPolicyService};
use cloud_cnapp::{CnappAnalyticsService, CnappFleetMonitor, TenantCnappPolicyService};
use cloud_ai::{AiAnalyticsService, AiFleetMonitor, TenantAiPolicyService};
use cloud_wiresock::{
    WiresockAnalyticsService, WiresockFleetMonitor, TenantWiresockPolicyService,
};
use compliance::ComplianceEngine;
use database::setup;
use federation::{FederationManager, RegisterControllerRequest};
use serde_json::{json, Value};
use std::sync::Arc;
use sync::{CloudSyncEngine, SyncEntity, SyncPushRequest};

async fn spawn_test_server() -> (
    String,
    String,
    Arc<KernelFleetMonitor>,
    Arc<AnonymityFleetMonitor>,
    tokio::task::JoinHandle<()>,
) {
    let pool = setup("sqlite::memory:").await.expect("db");
    let policy = CloudSecurityPolicy {
        jwt_secret: "integration-test-secret".into(),
        bcrypt_cost: 4,
        ..Default::default()
    };

    let tenants = Arc::new(TenantManager::new(pool.clone()));
    let tenant = tenants
        .create(CreateTenantRequest {
            name: "Test Tenant".into(),
            slug: "test".into(),
        })
        .await
        .expect("tenant");
    let tenant_id = tenant.id.clone();

    let auth = Arc::new(auth::JwtAuthService::new(pool.clone(), policy));
    auth.ensure_default_admin(&tenant_id).await.expect("admin");

    let kernel_fleet = Arc::new(KernelFleetMonitor::new(pool.clone()));
    let anonymity_fleet = Arc::new(AnonymityFleetMonitor::new(pool.clone()));

    let billing = Arc::new(BillingManager::new(pool.clone()));

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
        kernel_fleet: kernel_fleet.clone(),
        anonymity_fleet: anonymity_fleet.clone(),
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
        cnapp_analytics: Arc::new(CnappAnalyticsService::new(CnappFleetMonitor::new(pool.clone()))),
        ai_fleet: Arc::new(AiFleetMonitor::new(pool.clone())),
        ai_policies: Arc::new(TenantAiPolicyService::new(pool.clone())),
        ai_analytics: Arc::new(AiAnalyticsService::new(AiFleetMonitor::new(pool.clone()))),
        wiresock_fleet: Arc::new(WiresockFleetMonitor::new(pool.clone())),
        wiresock_policies: Arc::new(TenantWiresockPolicyService::new(pool.clone())),
        wiresock_analytics: Arc::new(WiresockAnalyticsService::new(WiresockFleetMonitor::new(
            pool.clone(),
        ))),
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let base = format!("http://{addr}");

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });

    (base, tenant_id, kernel_fleet, anonymity_fleet, handle)
}

async fn login(base: &str, tenant_id: &str) -> String {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base}/api/v1/auth/login"))
        .json(&json!({"username":"admin","password":"admin","tenant_id": tenant_id}))
        .send()
        .await
        .expect("login");
    assert!(resp.status().is_success(), "login failed: {}", resp.status());
    let body: Value = resp.json().await.expect("json");
    body["token"].as_str().expect("token").to_string()
}

fn auth_headers(token: &str, tenant_id: &str) -> Vec<(String, String)> {
    vec![
        ("Authorization".into(), format!("Bearer {token}")),
        ("X-Tenant-Id".into(), tenant_id.into()),
    ]
}

#[tokio::test]
async fn integration_login_and_me() {
    let (base, tenant_id, _kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/api/v1/auth/me"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("me");
    assert_eq!(resp.status(), 200);

    handle.abort();
}

#[tokio::test]
async fn tenant_isolation_blocks_cross_tenant() {
    let (base, tenant_id, _kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();

    let other_tenant = "00000000-0000-0000-0000-000000000099";
    let resp = client
        .get(format!("{base}/api/v1/organizations"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", other_tenant)
        .send()
        .await
        .expect("orgs");
    assert_eq!(resp.status(), 403);

    handle.abort();
}

#[tokio::test]
async fn federation_register_and_list() {
    let (base, tenant_id, _kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();

    let register = client
        .post(format!("{base}/api/v1/federation/controllers"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .json(&json!({
            "name": "edge-ctrl",
            "endpoint_url": "https://ctrl.local",
            "api_key": "secret-key"
        }))
        .send()
        .await
        .expect("register");
    assert_eq!(register.status(), 201);

    let list = client
        .get(format!("{base}/api/v1/federation/controllers"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("list");
    assert_eq!(list.status(), 200);
    let controllers: Vec<Value> = list.json().await.expect("json");
    assert_eq!(controllers.len(), 1);

    handle.abort();
}

#[tokio::test]
async fn sync_push_and_pull() {
    let (base, tenant_id, _kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();

    let push = client
        .post(format!("{base}/api/v1/cloud/sync"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .json(&json!({
            "entities": [{
                "entity_type": "policy",
                "entity_id": "p1",
                "payload": {"name": "default"},
                "version": 1,
                "updated_at": "2026-01-01T00:00:00Z"
            }]
        }))
        .send()
        .await
        .expect("push");
    assert!(push.status().is_success());

    let pull = client
        .get(format!("{base}/api/v1/cloud/sync"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("pull");
    assert_eq!(pull.status(), 200);
    let body: Value = pull.json().await.expect("json");
    assert!(body["entities"].as_array().unwrap().len() >= 1);

    handle.abort();
}

#[tokio::test]
async fn compliance_run_and_list() {
    let (base, tenant_id, _kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();

    let run = client
        .post(format!("{base}/api/v1/compliance"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("run");
    assert!(run.status().is_success());
    let reports: Vec<Value> = run.json().await.expect("json");
    assert_eq!(reports.len(), 6);

    handle.abort();
}

#[tokio::test]
async fn rbac_viewer_cannot_create_team() {
    let pool = setup("sqlite::memory:").await.expect("db");
    let policy = CloudSecurityPolicy {
        jwt_secret: "rbac-test".into(),
        bcrypt_cost: 4,
        ..Default::default()
    };
    let tenants = Arc::new(TenantManager::new(pool.clone()));
    let tenant = tenants
        .create(CreateTenantRequest {
            name: "RBAC".into(),
            slug: "rbac".into(),
        })
        .await
        .expect("tenant");

    let auth = Arc::new(auth::JwtAuthService::new(pool.clone(), policy));
    auth.ensure_default_admin(&tenant.id).await.expect("admin");

    let viewer_id = uuid::Uuid::new_v4().to_string();
    let hash = bcrypt::hash("viewer", 4).expect("hash");
    sqlx::query(
        "INSERT INTO users (id, tenant_id, username, password_hash, role, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&viewer_id)
    .bind(&tenant.id)
    .bind("viewer")
    .bind(&hash)
    .bind(TeamRole::Viewer.as_str())
    .bind(database::models::now_iso())
    .execute(&pool)
    .await
    .expect("viewer");

    let state = AppState {
        pool: pool.clone(),
        auth: auth.clone(),
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
        cnapp_analytics: Arc::new(CnappAnalyticsService::new(CnappFleetMonitor::new(pool.clone()))),
        ai_fleet: Arc::new(AiFleetMonitor::new(pool.clone())),
        ai_policies: Arc::new(TenantAiPolicyService::new(pool.clone())),
        ai_analytics: Arc::new(AiAnalyticsService::new(AiFleetMonitor::new(pool.clone()))),
        wiresock_fleet: Arc::new(WiresockFleetMonitor::new(pool.clone())),
        wiresock_policies: Arc::new(TenantWiresockPolicyService::new(pool.clone())),
        wiresock_analytics: Arc::new(WiresockAnalyticsService::new(WiresockFleetMonitor::new(
            pool.clone(),
        ))),
        subscriptions: Arc::new(SubscriptionManager::new(pool.clone())),
        billing: Arc::new(BillingManager::new(pool.clone())),
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let base = format!("http://{}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });

    let client = reqwest::Client::new();
    let login_resp = client
        .post(format!("{base}/api/v1/auth/login"))
        .json(&json!({"username":"viewer","password":"viewer","tenant_id": tenant.id}))
        .send()
        .await
        .expect("login");
    let token = login_resp.json::<Value>().await.expect("json")["token"]
        .as_str()
        .unwrap()
        .to_string();

    let create = client
        .post(format!("{base}/api/v1/teams"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant.id)
        .json(&json!({"name": "ops"}))
        .send()
        .await
        .expect("create team");
    assert_eq!(create.status(), 403);

    handle.abort();
}

#[tokio::test]
async fn device_bundle_sync_push_pull() {
    let (base, tenant_id, _kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();
    let device_id = "endpoint-device-1";
    let bundle = json!({"version": 2, "settings": {"dns": "encrypted"}});

    let push = client
        .post(format!(
            "{base}/api/v1/tenants/{tenant_id}/devices/{device_id}/sync/push"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .header("content-type", "application/json")
        .body(bundle.to_string())
        .send()
        .await
        .expect("push");
    assert!(push.status().is_success(), "push status {}", push.status());

    let pull = client
        .get(format!(
            "{base}/api/v1/tenants/{tenant_id}/devices/{device_id}/sync/pull"
        ))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("pull");
    assert!(pull.status().is_success());
    let body: Value = pull.json().await.expect("json");
    assert_eq!(body["settings"]["dns"], "encrypted");

    handle.abort();
}

#[tokio::test]
async fn integration_health_is_public() {
    let (base, _tenant_id, _kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base}/health"))
        .send()
        .await
        .expect("health");
    assert_eq!(resp.status(), 200);
    handle.abort();
}

#[tokio::test]
async fn metrics_json_and_prometheus() {
    let (base, tenant_id, _kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();

    let json_resp = client
        .get(format!("{base}/api/v1/cloud/metrics"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("metrics json");
    assert!(json_resp.status().is_success());

    let prom = client
        .get(format!("{base}/api/v1/cloud/metrics"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .header("Accept", "text/plain")
        .send()
        .await
        .expect("metrics prom");
    assert!(prom.status().is_success());
    let body = prom.text().await.expect("text");
    assert!(body.contains("ws_cloud_tenants_active"));

    handle.abort();
}

#[tokio::test]
async fn integration_cloud_kernel_endpoints() {
    let (base, tenant_id, kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    kernel_fleet
        .record_rollup(
            &tenant_id,
            Some("ctrl-test"),
            &KernelRollupPayload {
                reporting_devices: 2,
                healthy_devices: 2,
                kernel_devices: 1,
                ndis_devices: 1,
                stub_devices: 0,
                total_active_routes: 3,
                classify_count: 500,
                packets_per_sec: 80,
                controllers: None,
            },
        )
        .await
        .expect("rollup");
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();

    let overview = client
        .get(format!("{base}/api/v1/cloud/kernel"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("kernel");
    assert!(overview.status().is_success());
    let overview_body: Value = overview.json().await.expect("json");
    assert_eq!(overview_body["reporting_devices"], 2);

    let stats = client
        .get(format!("{base}/api/v1/cloud/kernel/statistics"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("stats");
    assert!(stats.status().is_success());
    let stats_body: Value = stats.json().await.expect("json");
    assert_eq!(stats_body["classify_count"], 500);

    handle.abort();
}

#[tokio::test]
async fn integration_cloud_anonymity_endpoints() {
    let (base, tenant_id, _kernel_fleet, anonymity_fleet, handle) = spawn_test_server().await;
    anonymity_fleet
        .record_rollup(
            &tenant_id,
            Some("ctrl-anon"),
            &AnonymityRollupPayload {
                reporting_devices: 3,
                healthy_devices: 2,
                connected_devices: 3,
                federation_peers_total: 5,
                avg_anonymity_score: 78.0,
                avg_entropy_bits: 110.0,
                avg_route_entropy: 2.0,
                total_active_routes: 4,
                controllers: None,
            },
        )
        .await
        .expect("rollup");
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();

    let overview = client
        .get(format!("{base}/api/v1/cloud/anonymity"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("anonymity");
    assert!(overview.status().is_success());
    let overview_body: Value = overview.json().await.expect("json");
    assert_eq!(overview_body["reporting_devices"], 3);

    let analytics = client
        .get(format!("{base}/api/v1/cloud/anonymity/analytics"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("analytics");
    assert!(analytics.status().is_success());
    let analytics_body: Value = analytics.json().await.expect("json");
    assert_eq!(analytics_body["avg_entropy_bits"], 110.0);

    handle.abort();
}

#[tokio::test]
async fn integration_billing_plans_and_subscription() {
    std::env::set_var("STRIPE_MOCK", "1");
    let (base, tenant_id, _kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();

    let plans = client
        .get(format!("{base}/api/v1/billing/plans"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("plans");
    assert!(plans.status().is_success());
    let plans_body: Vec<Value> = plans.json().await.expect("json");
    assert!(!plans_body.is_empty());

    let sub = client
        .post(format!("{base}/api/v1/billing/subscription"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .json(&json!({"plan": "team", "seats": 2}))
        .send()
        .await
        .expect("create sub");
    assert_eq!(sub.status(), 201);

    handle.abort();
}

#[tokio::test]
async fn integration_quotas_list_and_update() {
    let (base, tenant_id, _kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();

    let list = client
        .get(format!("{base}/api/v1/quotas"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("quotas");
    assert!(list.status().is_success());
    let quotas: Vec<Value> = list.json().await.expect("json");
    assert!(!quotas.is_empty());

    let update = client
        .put(format!("{base}/api/v1/quotas"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .json(&json!({
            "quotas": [{
                "resource": "controllers",
                "soft_limit": 2.0,
                "hard_limit": 4.0
            }]
        }))
        .send()
        .await
        .expect("update quotas");
    assert!(update.status().is_success());

    handle.abort();
}

#[tokio::test]
async fn integration_regions_list_and_health() {
    let (base, tenant_id, _kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();

    let regions = client
        .get(format!("{base}/api/v1/regions"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("regions");
    assert!(regions.status().is_success());
    let regions_body: Vec<Value> = regions.json().await.expect("json");
    assert!(!regions_body.is_empty());

    let health = client
        .get(format!("{base}/api/v1/regions/health"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("health");
    assert!(health.status().is_success());

    handle.abort();
}

#[tokio::test]
async fn integration_logs_and_observability() {
    let (base, tenant_id, _kernel_fleet, _anonymity_fleet, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();

    let ingest = client
        .post(format!("{base}/api/v1/cloud/logs"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .json(&json!({
            "source": "integration-test",
            "level": "info",
            "message": "hello phase14"
        }))
        .send()
        .await
        .expect("ingest");
    assert!(ingest.status().is_success());

    let logs = client
        .get(format!("{base}/api/v1/logs"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("logs");
    assert!(logs.status().is_success());
    let logs_body: Vec<Value> = logs.json().await.expect("json");
    assert!(!logs_body.is_empty());

    let search = client
        .get(format!("{base}/api/v1/logs/search?q=phase14"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("search");
    assert!(search.status().is_success());

    let metrics = client
        .get(format!("{base}/api/v1/observability/metrics"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("observability");
    assert!(metrics.status().is_success());

    handle.abort();
}

#[allow(dead_code)]
fn _unused_imports() {
    let _ = CreateOrganizationRequest { tenant_id: String::new(), name: String::new() };
    let _ = CreateTeamRequest { tenant_id: String::new(), organization_id: None, name: String::new() };
    let _ = RegisterControllerRequest {
        tenant_id: String::new(),
        name: String::new(),
        endpoint_url: String::new(),
        api_key: String::new(),
    };
    let _ = SyncPushRequest { tenant_id: String::new(), controller_id: None, entities: vec![] };
    let _ = SyncEntity {
        entity_type: String::new(),
        entity_id: String::new(),
        payload: json!({}),
        version: 0,
        updated_at: String::new(),
    };
    let _ = CreateSubscriptionRequest {
        tenant_id: String::new(),
        plan: Plan::Free,
        seats: None,
    };
    let _ = auth_headers("", "");
}

#[tokio::test]
async fn integration_cloud_ztna_fleet_and_resources() {
    let (base, tenant_id, _kernel, _anonymity, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();
    let auth = format!("Bearer {token}");

    let provider = client
        .post(format!("{base}/api/v1/identity/providers"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .json(&json!({
            "name": "Corp OIDC",
            "provider_kind": "generic_oidc",
            "enabled": true
        }))
        .send()
        .await
        .expect("create provider");
    assert_eq!(provider.status(), 201);

    let resource = client
        .post(format!("{base}/api/v1/resources"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .json(&json!({
            "name": "Finance Portal",
            "host": "finance.internal",
            "port": 443,
            "published": true
        }))
        .send()
        .await
        .expect("create resource");
    assert_eq!(resource.status(), 201);

    let resources = client
        .get(format!("{base}/api/v1/resources"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("list resources");
    assert!(resources.status().is_success());
    let resource_list: Vec<Value> = resources.json().await.expect("json");
    assert_eq!(resource_list.len(), 1);

    let ztna = client
        .get(format!("{base}/api/v1/cloud/ztna"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("ztna fleet");
    assert!(ztna.status().is_success());

    let analytics = client
        .get(format!("{base}/api/v1/cloud/ztna/analytics"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("ztna analytics");
    assert!(analytics.status().is_success());

    handle.abort();
}

#[tokio::test]
async fn integration_cloud_sse_policies_and_fleet() {
    let (base, tenant_id, _kernel, _anonymity, handle) = spawn_test_server().await;
    let token = login(&base, &tenant_id).await;
    let client = reqwest::Client::new();
    let auth = format!("Bearer {token}");

    let policy = client
        .post(format!("{base}/api/v1/sse/policies"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .json(&json!({
            "name": "Corp SWG",
            "policy_kind": "swg",
            "enabled": true,
            "default_action": "block"
        }))
        .send()
        .await
        .expect("create sse policy");
    assert_eq!(policy.status(), 201);

    let policies = client
        .get(format!("{base}/api/v1/sse/policies"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("list sse policies");
    assert!(policies.status().is_success());
    let policy_list: Vec<Value> = policies.json().await.expect("json");
    assert_eq!(policy_list.len(), 1);

    let sse = client
        .get(format!("{base}/api/v1/cloud/sse"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("sse fleet");
    assert!(sse.status().is_success());

    let analytics = client
        .get(format!("{base}/api/v1/cloud/sse/analytics"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("sse analytics");
    assert!(analytics.status().is_success());

    let xdr = client
        .get(format!("{base}/api/v1/cloud/xdr"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("xdr fleet");
    assert!(xdr.status().is_success());

    let xdr_analytics = client
        .get(format!("{base}/api/v1/cloud/xdr/analytics"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("xdr analytics");
    assert!(xdr_analytics.status().is_success());

    let incidents = client
        .get(format!("{base}/api/v1/cloud/xdr/incidents"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("xdr incidents");
    assert!(incidents.status().is_success());

    let detections = client
        .get(format!("{base}/api/v1/cloud/xdr/detections"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("xdr detections");
    assert!(detections.status().is_success());

    let mitre = client
        .get(format!("{base}/api/v1/cloud/xdr/mitre-coverage"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("xdr mitre");
    assert!(mitre.status().is_success());

    let cnapp = client
        .get(format!("{base}/api/v1/cloud/cnapp"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("cnapp fleet");
    assert!(cnapp.status().is_success());

    let cnapp_analytics = client
        .get(format!("{base}/api/v1/cloud/cnapp/analytics"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("cnapp analytics");
    assert!(cnapp_analytics.status().is_success());

    let posture = client
        .get(format!("{base}/api/v1/cloud/cnapp/posture"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("cnapp posture");
    assert!(posture.status().is_success());

    let compliance = client
        .get(format!("{base}/api/v1/cloud/cnapp/compliance"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("cnapp compliance");
    assert!(compliance.status().is_success());

    let vulnerabilities = client
        .get(format!("{base}/api/v1/cloud/cnapp/vulnerabilities"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("cnapp vulnerabilities");
    assert!(vulnerabilities.status().is_success());

    let ai = client
        .get(format!("{base}/api/v1/cloud/ai"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("ai fleet");
    assert!(ai.status().is_success());

    let ai_analytics = client
        .get(format!("{base}/api/v1/cloud/ai/analytics"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("ai analytics");
    assert!(ai_analytics.status().is_success());

    let ai_risk = client
        .get(format!("{base}/api/v1/cloud/ai/risk"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("ai risk");
    assert!(ai_risk.status().is_success());

    let ai_reports = client
        .get(format!("{base}/api/v1/cloud/ai/reports"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("ai reports");
    assert!(ai_reports.status().is_success());

    let ai_investigations = client
        .get(format!("{base}/api/v1/cloud/ai/investigations"))
        .header("Authorization", &auth)
        .header("X-Tenant-Id", &tenant_id)
        .send()
        .await
        .expect("ai investigations");
    assert!(ai_investigations.status().is_success());

    handle.abort();
}
