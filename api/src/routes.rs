use auth::{LoginRequest, TeamRole};
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    extract::Query,
    Json, Router,
};
use billing::{CreateSubscriptionRequest, Plan, SubscriptionManager};
use cloud_billing::BillingManager;
use cloud_core::{
    write_audit_event, AssignDeviceRequest, AssignPolicyRequest, AnonymityFleetMonitor,
    AuditWriteRequest, CloudMetricsAggregator, CreateOrganizationRequest, CreateTeamRequest,
    CreateTenantRequest, KernelFleetMonitor, OrganizationManager, TeamManager,
    TeamMembershipRequest, TenantManager,
};
use cloud_metering::{RecordUsageRequest, UsageMeteringService, UsageMetric};
use cloud_provisioning::{
    BackupRequest, HostedControllerManager, ProvisionRequest, RestoreRequest, UpgradeRequest,
};
use cloud_quotas::{QuotaManager, SetQuotaRequest};
use cloud_recovery::{DisasterRecoveryManager, RunRecoveryRequest};
use cloud_ha::{HaManager, RegisterNodeRequest};
use cloud_logging::{LogAggregationService, LogSearchQuery};
use cloud_observability::TelemetryPipeline;
use cloud_regions::RegionManager;
use cloud_storage::{BackupStorageService, StorageError, UploadBackupRequest};
use cloud_ztna::{
    CloudZtnaPolicyService, CreateIdentityProviderRequest, CreatePublishedResourceRequest,
    ResourcePublisher, TenantIdentityService, UpdatePublishedResourceRequest, ZtnaFleetMonitor,
};
use cloud_sse::{
    CreateSsePolicyRequest, SseAnalyticsService, SseFleetMonitor, TenantSsePolicyService,
};
use cloud_xdr::{XdrAnalyticsService, XdrFleetMonitor, TenantXdrPolicyService};
use cloud_cnapp::{CnappAnalyticsService, CnappFleetMonitor, TenantCnappPolicyService};
use cloud_ai::{AiAnalyticsService, AiFleetMonitor, TenantAiPolicyService};
use cloud_wiresock::{
    WiresockAnalyticsService, WiresockFleetMonitor, TenantWiresockPolicyService,
};
use compliance::ComplianceEngine;
use database::DbPool;
use federation::{FederationManager, RegisterControllerRequest};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sync::{CloudSyncEngine, SyncPushRequest};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::error::ApiError;
use crate::middleware::{self as auth_mw, AuthUser};
use crate::openapi;

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub auth: Arc<auth::JwtAuthService>,
    pub tenants: Arc<TenantManager>,
    pub organizations: Arc<OrganizationManager>,
    pub teams: Arc<TeamManager>,
    pub federation: Arc<FederationManager>,
    pub sync: Arc<CloudSyncEngine>,
    pub compliance: Arc<ComplianceEngine>,
    pub metrics: Arc<CloudMetricsAggregator>,
    pub kernel_fleet: Arc<KernelFleetMonitor>,
    pub anonymity_fleet: Arc<AnonymityFleetMonitor>,
    pub ztna_fleet: Arc<ZtnaFleetMonitor>,
    pub ztna_identity: Arc<TenantIdentityService>,
    pub ztna_policies: Arc<CloudZtnaPolicyService>,
    pub ztna_resources: Arc<ResourcePublisher>,
    pub sse_fleet: Arc<SseFleetMonitor>,
    pub sse_policies: Arc<TenantSsePolicyService>,
    pub sse_analytics: Arc<SseAnalyticsService>,
    pub xdr_fleet: Arc<XdrFleetMonitor>,
    pub xdr_policies: Arc<TenantXdrPolicyService>,
    pub xdr_analytics: Arc<XdrAnalyticsService>,
    pub cnapp_fleet: Arc<CnappFleetMonitor>,
    pub cnapp_policies: Arc<TenantCnappPolicyService>,
    pub cnapp_analytics: Arc<CnappAnalyticsService>,
    pub ai_fleet: Arc<AiFleetMonitor>,
    pub ai_policies: Arc<TenantAiPolicyService>,
    pub ai_analytics: Arc<AiAnalyticsService>,
    pub wiresock_fleet: Arc<WiresockFleetMonitor>,
    pub wiresock_policies: Arc<TenantWiresockPolicyService>,
    pub wiresock_analytics: Arc<WiresockAnalyticsService>,
    pub subscriptions: Arc<SubscriptionManager>,
    pub billing: Arc<BillingManager>,
    pub metering: Arc<UsageMeteringService>,
    pub quotas: Arc<QuotaManager>,
    pub provisioning: Arc<HostedControllerManager>,
    pub regions: Arc<RegionManager>,
    pub recovery: Arc<DisasterRecoveryManager>,
    pub backups: Arc<BackupStorageService>,
    pub logging: Arc<LogAggregationService>,
    pub observability: Arc<TelemetryPipeline>,
    pub ha: Arc<HaManager>,
}

pub fn build_router(state: AppState) -> Router {
    let public = Router::new()
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/plans", get(list_plans))
        .route("/api/v1/billing/webhook", post(billing_webhook))
        .route("/api/v1/openapi.json", get(openapi_doc))
        .route("/health", get(health));

    let protected = Router::new()
        .route("/api/v1/auth/me", get(me))
        .route("/api/v1/tenants", get(list_tenants).post(create_tenant))
        .route("/api/v1/tenants/{id}", get(get_tenant))
        .route("/api/v1/organizations", get(list_organizations).post(create_organization))
        .route("/api/v1/teams", get(list_teams).post(create_team))
        .route("/api/v1/teams/{id}/members", get(list_team_members).post(add_team_member))
        .route("/api/v1/teams/{id}/devices", post(assign_team_device))
        .route("/api/v1/teams/{id}/policies", post(assign_team_policy))
        .route("/api/v1/federation/controllers", get(list_controllers).post(register_controller))
        .route("/api/v1/federation/controllers/{id}/revoke", post(revoke_controller))
        .route("/api/v1/federation/controllers/{id}/sync", post(sync_controller))
        .route("/api/v1/federation/controllers/{id}/health", post(health_controller))
        .route("/api/v1/cloud/sync", get(get_sync).post(post_sync))
        .route(
            "/api/v1/tenants/{tenant_id}/devices/{device_id}/sync/push",
            post(device_sync_push),
        )
        .route(
            "/api/v1/tenants/{tenant_id}/devices/{device_id}/sync/pull",
            get(device_sync_pull),
        )
        .route("/api/v1/compliance", get(list_compliance).post(run_compliance))
        .route("/api/v1/cloud/metrics", get(get_metrics))
        .route("/api/v1/cloud/kernel", get(get_cloud_kernel))
        .route("/api/v1/cloud/kernel/statistics", get(get_cloud_kernel_statistics))
        .route("/api/v1/cloud/anonymity", get(get_cloud_anonymity))
        .route("/api/v1/cloud/anonymity/analytics", get(get_cloud_anonymity_analytics))
        .route("/api/v1/cloud/ztna", get(get_cloud_ztna))
        .route("/api/v1/cloud/ztna/analytics", get(get_cloud_ztna_analytics))
        .route("/api/v1/cloud/sse", get(get_cloud_sse))
        .route("/api/v1/cloud/sse/analytics", get(get_cloud_sse_analytics))
        .route("/api/v1/cloud/xdr", get(get_cloud_xdr))
        .route("/api/v1/cloud/xdr/analytics", get(get_cloud_xdr_analytics))
        .route("/api/v1/cloud/xdr/incidents", get(get_cloud_xdr_incidents))
        .route("/api/v1/cloud/xdr/detections", get(get_cloud_xdr_detections))
        .route("/api/v1/cloud/xdr/mitre-coverage", get(get_cloud_xdr_mitre_coverage))
        .route("/api/v1/cloud/cnapp", get(get_cloud_cnapp))
        .route("/api/v1/cloud/cnapp/posture", get(get_cloud_cnapp_posture))
        .route("/api/v1/cloud/cnapp/compliance", get(get_cloud_cnapp_compliance))
        .route("/api/v1/cloud/cnapp/vulnerabilities", get(get_cloud_cnapp_vulnerabilities))
        .route("/api/v1/cloud/cnapp/analytics", get(get_cloud_cnapp_analytics))
        .route("/api/v1/cloud/ai", get(get_cloud_ai))
        .route("/api/v1/cloud/ai/risk", get(get_cloud_ai_risk))
        .route("/api/v1/cloud/ai/reports", get(get_cloud_ai_reports))
        .route("/api/v1/cloud/ai/investigations", get(get_cloud_ai_investigations))
        .route("/api/v1/cloud/ai/analytics", get(get_cloud_ai_analytics))
        .route("/api/v1/cloud/split-templates", get(get_cloud_split_templates))
        .route("/api/v1/cloud/tcp-termination", get(get_cloud_tcp_termination))
        .route("/api/v1/cloud/handshake-proxy", get(get_cloud_handshake_proxy))
        .route("/api/v1/sse/policies", get(list_sse_policies).post(create_sse_policy))
        .route(
            "/api/v1/identity/providers",
            get(list_identity_providers).post(create_identity_provider),
        )
        .route(
            "/api/v1/resources",
            get(list_published_resources)
                .post(create_published_resource),
        )
        .route(
            "/api/v1/resources/{id}",
            put(update_published_resource).delete(delete_published_resource),
        )
        .route("/api/v1/subscriptions", get(list_subscriptions).post(create_subscription))
        .route("/api/v1/billing/plans", get(list_billing_plans).post(seed_billing_plans))
        .route("/api/v1/billing/subscription", get(get_billing_subscription).post(create_billing_subscription))
        .route("/api/v1/billing/invoices", get(list_billing_invoices))
        .route("/api/v1/billing/checkout", post(billing_checkout))
        .route("/api/v1/quotas", get(get_quotas).put(update_quotas))
        .route("/api/v1/controllers/provision", post(provision_controller))
        .route("/api/v1/controllers/upgrade", post(upgrade_controller))
        .route("/api/v1/controllers/backup", post(backup_controller))
        .route("/api/v1/controllers/restore", post(restore_controller))
        .route("/api/v1/regions", get(list_regions))
        .route("/api/v1/regions/health", get(regions_health))
        .route("/api/v1/recovery/run", post(run_recovery))
        .route("/api/v1/recovery/runs", get(list_recovery_runs))
        .route("/api/v1/backups/upload", post(upload_backup))
        .route("/api/v1/backups/restore", post(restore_backup))
        .route("/api/v1/backups", get(list_backups))
        .route("/api/v1/cloud/usage", get(list_usage).post(ingest_usage))
        .route("/api/v1/cloud/health", post(ingest_cloud_health))
        .route("/api/v1/cloud/logs", post(ingest_cloud_logs))
        .route("/api/v1/logs", get(list_logs))
        .route("/api/v1/logs/search", get(search_logs))
        .route("/api/v1/observability/metrics", get(get_observability_metrics))
        .route("/api/v1/ha/nodes", get(list_ha_nodes).post(register_ha_node))
        .route("/api/v1/ha/nodes/{id}/heartbeat", post(ha_heartbeat))
        .route("/api/v1/ha/leader", get(get_ha_leader))
        .layer(middleware::from_fn_with_state(state.clone(), auth_mw::require_auth));

    Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<auth::LoginResponse>, ApiError> {
    Ok(Json(state.auth.login(req).await?))
}

async fn me(auth: AuthUser) -> Json<MeResponse> {
    Json(MeResponse {
        user_id: auth.claims.sub.clone(),
        username: auth.claims.username.clone(),
        role: auth.claims.role,
        tenant_id: auth.tenant_id.clone(),
    })
}

#[derive(Serialize)]
struct MeResponse {
    user_id: String,
    username: String,
    role: TeamRole,
    tenant_id: String,
}

async fn list_tenants(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_core::Tenant>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Administrator)?;
    Ok(Json(state.tenants.list().await?))
}

async fn create_tenant(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateTenantRequest>,
) -> Result<(StatusCode, Json<cloud_core::Tenant>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Owner)?;
    let tenant = state.tenants.create(req).await?;
    Ok((StatusCode::CREATED, Json(tenant)))
}

async fn get_tenant(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<cloud_core::Tenant>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.tenants.get(&id).await?))
}

async fn list_organizations(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_core::Organization>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.organizations.list(&auth.tenant_id).await?))
}

async fn create_organization(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateOrganizationBody>,
) -> Result<(StatusCode, Json<cloud_core::Organization>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    let org = state
        .organizations
        .create(CreateOrganizationRequest {
            tenant_id: auth.tenant_id.clone(),
            name: body.name,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(org)))
}

#[derive(Deserialize)]
struct CreateOrganizationBody {
    name: String,
}

async fn list_teams(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_core::Team>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.teams.list(&auth.tenant_id).await?))
}

async fn create_team(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateTeamBody>,
) -> Result<(StatusCode, Json<cloud_core::Team>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    state.subscriptions.enforce_team_quota(&auth.tenant_id).await?;
    let team = state
        .teams
        .create(CreateTeamRequest {
            tenant_id: auth.tenant_id.clone(),
            organization_id: body.organization_id,
            name: body.name,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(team)))
}

#[derive(Deserialize)]
struct CreateTeamBody {
    name: String,
    organization_id: Option<String>,
}

async fn list_team_members(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Vec<cloud_core::TeamMember>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state.teams.list_members(&auth.tenant_id, &id).await?,
    ))
}

async fn add_team_member(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<TeamMembershipRequest>,
) -> Result<(StatusCode, Json<cloud_core::TeamMember>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Administrator)?;
    let member = state
        .teams
        .add_member(&auth.tenant_id, &id, req)
        .await?;
    Ok((StatusCode::CREATED, Json(member)))
}

async fn assign_team_device(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<AssignDeviceRequest>,
) -> Result<StatusCode, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    state
        .teams
        .assign_device(&auth.tenant_id, &id, req)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn assign_team_policy(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<AssignPolicyRequest>,
) -> Result<StatusCode, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    state
        .teams
        .assign_policy(&auth.tenant_id, &id, req)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_controllers(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<federation::FederatedController>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.federation.list(&auth.tenant_id).await?))
}

async fn register_controller(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<RegisterControllerBody>,
) -> Result<(StatusCode, Json<RegisterControllerResponse>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Administrator)?;
    let (controller, event) = state
        .federation
        .register_controller(RegisterControllerRequest {
            tenant_id: auth.tenant_id.clone(),
            name: body.name,
            endpoint_url: body.endpoint_url,
            api_key: body.api_key,
        })
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(RegisterControllerResponse { controller, event }),
    ))
}

#[derive(Deserialize)]
struct RegisterControllerBody {
    name: String,
    endpoint_url: String,
    api_key: String,
}

#[derive(Serialize)]
struct RegisterControllerResponse {
    controller: federation::FederatedController,
    event: federation::FederationEvent,
}

async fn revoke_controller(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<RegisterControllerResponse>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Administrator)?;
    let (controller, event) = state.federation.revoke(&auth.tenant_id, &id).await?;
    Ok(Json(RegisterControllerResponse { controller, event }))
}

async fn sync_controller(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<federation::FederationEvent>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    Ok(Json(
        state.federation.sync(&auth.tenant_id, &id).await?,
    ))
}

async fn health_controller(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<federation::HealthCheckResult>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state.federation.health_check(&auth.tenant_id, &id).await?,
    ))
}

async fn get_sync(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<sync::SyncPullResponse>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    let entities = state.sync.pull(&auth.tenant_id).await?;
    let conflicts = state.sync.list_conflicts(&auth.tenant_id).await?;
    Ok(Json(sync::SyncPullResponse {
        entities,
        conflicts,
    }))
}

async fn post_sync(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<SyncPushBody>,
) -> Result<Json<sync::SyncPullResponse>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    Ok(Json(
        state
            .sync
            .bidirectional(SyncPushRequest {
                tenant_id: auth.tenant_id.clone(),
                controller_id: body.controller_id,
                entities: body.entities,
            })
            .await?,
    ))
}

#[derive(Deserialize)]
struct SyncPushBody {
    controller_id: Option<String>,
    entities: Vec<sync::SyncEntity>,
}

async fn device_sync_push(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((tenant_id, device_id)): Path<(String, String)>,
    body: String,
) -> Result<StatusCode, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    if tenant_id != auth.tenant_id {
        return Err(ApiError::Forbidden);
    }
    state
        .sync
        .push_device_bundle(&tenant_id, &device_id, &body)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn device_sync_pull(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((tenant_id, device_id)): Path<(String, String)>,
) -> Result<Response, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    if tenant_id != auth.tenant_id {
        return Err(ApiError::Forbidden);
    }
    let bundle = state
        .sync
        .pull_device_bundle(&tenant_id, &device_id)
        .await?;
    match bundle {
        Some(json) => Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            json,
        )
            .into_response()),
        None => Ok(StatusCode::NO_CONTENT.into_response()),
    }
}

async fn list_compliance(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<compliance::ComplianceReport>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.compliance.list(&auth.tenant_id).await?))
}

async fn run_compliance(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<compliance::ComplianceReport>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    Ok(Json(
        state.compliance.run_checks(&auth.tenant_id).await?,
    ))
}

async fn get_metrics(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    let snapshot = state.metrics.tenant_snapshot(&auth.tenant_id).await?;
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    if accept.contains("text/plain") || accept.contains("application/openmetrics-text") {
        let global = state.metrics.snapshot().await?;
        let body = CloudMetricsAggregator::to_prometheus(&global);
        Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
            body,
        )
            .into_response())
    } else {
        Ok(Json(snapshot).into_response())
    }
}

async fn get_cloud_kernel(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_core::KernelFleetOverview>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.kernel_fleet.fleet_overview(&auth.tenant_id).await?))
}

async fn get_cloud_kernel_statistics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_core::KernelFleetStatistics>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.kernel_fleet.statistics(&auth.tenant_id).await?))
}

async fn get_cloud_anonymity(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_core::AnonymityFleetOverview>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.anonymity_fleet.fleet_overview(&auth.tenant_id).await?))
}

async fn get_cloud_anonymity_analytics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_core::AnonymityPrivacyAnalytics>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .anonymity_fleet
            .privacy_analytics(&auth.tenant_id)
            .await?,
    ))
}

async fn get_cloud_ztna(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_ztna::ZtnaFleetOverview>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.ztna_fleet.fleet_overview(&auth.tenant_id).await?))
}

async fn get_cloud_ztna_analytics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_ztna::ZtnaAnalyticsSummary>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.ztna_fleet.analytics(&auth.tenant_id).await?))
}

async fn get_cloud_sse(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_sse::SseFleetOverview>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.sse_fleet.fleet_overview(&auth.tenant_id).await?))
}

async fn get_cloud_sse_analytics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_sse::SseAnalyticsSummary>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.sse_analytics.analytics(&auth.tenant_id).await?))
}

async fn get_cloud_xdr(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_xdr::XdrFleetOverview>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.xdr_fleet.fleet_overview(&auth.tenant_id).await?))
}

async fn get_cloud_xdr_analytics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_xdr::XdrAnalyticsSummary>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.xdr_analytics.analytics(&auth.tenant_id).await?))
}

async fn get_cloud_xdr_incidents(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_xdr::XdrIncidentRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .xdr_fleet
            .list_incidents(&auth.tenant_id, None)
            .await?,
    ))
}

async fn get_cloud_xdr_detections(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_xdr::XdrDetectionRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .xdr_fleet
            .list_detections(&auth.tenant_id, None)
            .await?,
    ))
}

async fn get_cloud_xdr_mitre_coverage(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_xdr::XdrMitreCoverageRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .xdr_fleet
            .list_mitre_coverage(&auth.tenant_id)
            .await?,
    ))
}

async fn get_cloud_cnapp(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_cnapp::CnappFleetOverview>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.cnapp_fleet.fleet_overview(&auth.tenant_id).await?))
}

async fn get_cloud_cnapp_posture(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_cnapp::CnappPostureRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .cnapp_fleet
            .list_posture(&auth.tenant_id, None)
            .await?,
    ))
}

async fn get_cloud_cnapp_compliance(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_cnapp::CnappComplianceRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .cnapp_fleet
            .list_compliance(&auth.tenant_id, None)
            .await?,
    ))
}

async fn get_cloud_cnapp_vulnerabilities(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_cnapp::CnappVulnerabilityRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .cnapp_fleet
            .list_vulnerabilities(&auth.tenant_id, None)
            .await?,
    ))
}

async fn get_cloud_cnapp_analytics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_cnapp::CnappAnalyticsSummary>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.cnapp_analytics.analytics(&auth.tenant_id).await?))
}

async fn get_cloud_ai(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_ai::AiFleetOverview>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.ai_fleet.fleet_overview(&auth.tenant_id).await?))
}

async fn get_cloud_ai_risk(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_ai::AiRiskRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .ai_fleet
            .list_risk(&auth.tenant_id, None)
            .await?,
    ))
}

async fn get_cloud_ai_reports(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_ai::AiReportRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .ai_fleet
            .list_reports(&auth.tenant_id, None)
            .await?,
    ))
}

async fn get_cloud_ai_investigations(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_ai::AiInvestigationRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .ai_fleet
            .list_investigations(&auth.tenant_id, None)
            .await?,
    ))
}

async fn get_cloud_ai_analytics(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_ai::AiAnalyticsSummary>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.ai_analytics.analytics(&auth.tenant_id).await?))
}

async fn get_cloud_split_templates(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<cloud_wiresock::WiresockFleetOverview>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .wiresock_fleet
            .fleet_overview(&auth.tenant_id)
            .await?,
    ))
}

async fn get_cloud_tcp_termination(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_wiresock::WiresockTcpTerminationRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .wiresock_fleet
            .list_tcp_termination(&auth.tenant_id, None)
            .await?,
    ))
}

async fn get_cloud_handshake_proxy(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_wiresock::WiresockHandshakeProxyRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .wiresock_fleet
            .list_handshake_proxy(&auth.tenant_id, None)
            .await?,
    ))
}

async fn list_sse_policies(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_sse::SsePolicyRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.sse_policies.list(&auth.tenant_id).await?))
}

async fn create_sse_policy(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateSsePolicyRequest>,
) -> Result<(StatusCode, Json<cloud_sse::SsePolicyRecord>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Administrator)?;
    let record = state
        .sse_policies
        .create(&auth.tenant_id, req, Some(&auth.claims.username))
        .await?;
    Ok((StatusCode::CREATED, Json(record)))
}

async fn list_identity_providers(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_ztna::IdentityProviderRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state
            .ztna_identity
            .list_providers(&auth.tenant_id)
            .await?,
    ))
}

async fn create_identity_provider(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateIdentityProviderRequest>,
) -> Result<(StatusCode, Json<cloud_ztna::IdentityProviderRecord>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    let provider = state
        .ztna_identity
        .create_provider(&auth.tenant_id, req, Some(&auth.claims.username))
        .await?;
    Ok((StatusCode::CREATED, Json(provider)))
}

async fn list_published_resources(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_ztna::PublishedResourceRecord>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.ztna_resources.list(&auth.tenant_id).await?))
}

async fn create_published_resource(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreatePublishedResourceRequest>,
) -> Result<(StatusCode, Json<cloud_ztna::PublishedResourceRecord>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    let resource = state
        .ztna_resources
        .create(&auth.tenant_id, req, Some(&auth.claims.username))
        .await?;
    Ok((StatusCode::CREATED, Json(resource)))
}

async fn update_published_resource(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdatePublishedResourceRequest>,
) -> Result<Json<cloud_ztna::PublishedResourceRecord>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    Ok(Json(
        state
            .ztna_resources
            .update(&auth.tenant_id, &id, req, Some(&auth.claims.username))
            .await?,
    ))
}

async fn delete_published_resource(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    state
        .ztna_resources
        .delete(&auth.tenant_id, &id, Some(&auth.claims.username))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_subscriptions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<billing::Subscription>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state.subscriptions.list(&auth.tenant_id).await?,
    ))
}

#[derive(Deserialize)]
struct CreateSubBody {
    plan: Plan,
    seats: Option<i64>,
}

async fn create_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateSubBody>,
) -> Result<(StatusCode, Json<billing::Subscription>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Owner)?;
    let sub = state
        .subscriptions
        .create(CreateSubscriptionRequest {
            tenant_id: auth.tenant_id.clone(),
            plan: body.plan,
            seats: body.seats,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(sub)))
}

async fn list_plans() -> Json<Vec<billing::PlanInfo>> {
    Json(SubscriptionManager::list_plans())
}

// --- Phase 14 billing ---

async fn list_billing_plans(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_billing::BillingPlan>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.billing.list_plans().await?))
}

async fn seed_billing_plans(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<StatusCode, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Owner)?;
    state.billing.plans.seed_defaults().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_billing_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Option<billing::Subscription>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.billing.get_subscription(&auth.tenant_id).await?))
}

async fn create_billing_subscription(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateSubBody>,
) -> Result<(StatusCode, Json<billing::Subscription>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Owner)?;
    let sub = state
        .billing
        .create_subscription(
            CreateSubscriptionRequest {
                tenant_id: auth.tenant_id.clone(),
                plan: body.plan,
                seats: body.seats,
            },
            Some(&auth.claims.username),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(sub)))
}

async fn list_billing_invoices(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_billing::Invoice>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.billing.list_invoices(&auth.tenant_id).await?))
}

#[derive(Deserialize)]
struct CheckoutBody {
    plan_id: String,
    success_url: String,
    cancel_url: String,
}

async fn billing_checkout(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CheckoutBody>,
) -> Result<Json<cloud_billing::CheckoutSession>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Owner)?;
    Ok(Json(
        state
            .billing
            .create_checkout(
                &auth.tenant_id,
                &body.plan_id,
                &body.success_url,
                &body.cancel_url,
            )
            .await?,
    ))
}

async fn billing_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<cloud_billing::WebhookResult>, ApiError> {
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    Ok(Json(
        state
            .billing
            .handle_webhook(body.as_bytes(), signature)
            .await?,
    ))
}

// --- Phase 14 quotas ---

async fn get_quotas(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_quotas::TenantQuota>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.quotas.get_quotas(&auth.tenant_id).await?))
}

#[derive(Deserialize)]
struct UpdateQuotasBody {
    quotas: Vec<SetQuotaRequest>,
}

async fn update_quotas(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateQuotasBody>,
) -> Result<Json<Vec<cloud_quotas::TenantQuota>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Administrator)?;
    let mut updated = Vec::new();
    for q in body.quotas {
        updated.push(state.quotas.set_quota(&auth.tenant_id, q).await?);
    }
    Ok(Json(updated))
}

// --- Phase 14 provisioning ---

#[derive(Deserialize)]
struct ProvisionBody {
    name: String,
    region_id: String,
    plan_tier: String,
}

async fn provision_controller(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<ProvisionBody>,
) -> Result<(StatusCode, Json<ProvisionResponse>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Administrator)?;
    state.quotas.enforce_controller_quota(&auth.tenant_id).await?;
    let (controller, job) = state
        .provisioning
        .provision(ProvisionRequest {
            tenant_id: auth.tenant_id.clone(),
            name: body.name,
            region_id: body.region_id,
            plan_tier: body.plan_tier,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(ProvisionResponse { controller, job })))
}

#[derive(Serialize)]
struct ProvisionResponse {
    controller: cloud_provisioning::HostedController,
    job: cloud_provisioning::ProvisioningJob,
}

#[derive(Deserialize)]
struct UpgradeBody {
    controller_id: String,
    plan_tier: String,
}

async fn upgrade_controller(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpgradeBody>,
) -> Result<Json<cloud_provisioning::ProvisioningJob>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Administrator)?;
    Ok(Json(
        state
            .provisioning
            .upgrade(UpgradeRequest {
                tenant_id: auth.tenant_id.clone(),
                controller_id: body.controller_id,
                plan_tier: body.plan_tier,
            })
            .await?,
    ))
}

#[derive(Deserialize)]
struct BackupBody {
    controller_id: String,
}

async fn backup_controller(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<BackupBody>,
) -> Result<Json<BackupResponse>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    let (job, snapshot_id) = state
        .provisioning
        .backup(BackupRequest {
            tenant_id: auth.tenant_id.clone(),
            controller_id: body.controller_id,
        })
        .await?;
    Ok(Json(BackupResponse { job, snapshot_id }))
}

#[derive(Serialize)]
struct BackupResponse {
    job: cloud_provisioning::ProvisioningJob,
    snapshot_id: String,
}

#[derive(Deserialize)]
struct RestoreControllerBody {
    controller_id: String,
    snapshot_id: String,
}

async fn restore_controller(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<RestoreControllerBody>,
) -> Result<Json<cloud_provisioning::ProvisioningJob>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    Ok(Json(
        state
            .provisioning
            .restore(RestoreRequest {
                tenant_id: auth.tenant_id.clone(),
                controller_id: body.controller_id,
                snapshot_id: body.snapshot_id,
            })
            .await?,
    ))
}

// --- Phase 14 regions ---

async fn list_regions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_regions::CloudRegion>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.regions.list_regions().await?))
}

async fn regions_health(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_regions::RegionHealth>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.regions.probe_all_regions().await?))
}

// --- Phase 14 recovery ---

#[derive(Deserialize)]
struct RunRecoveryBody {
    plan_id: String,
}

async fn run_recovery(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<RunRecoveryBody>,
) -> Result<(StatusCode, Json<cloud_recovery::RecoveryRun>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Administrator)?;
    let run = state
        .recovery
        .run_recovery(RunRecoveryRequest {
            tenant_id: auth.tenant_id.clone(),
            plan_id: body.plan_id,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(run)))
}

async fn list_recovery_runs(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_recovery::RecoveryRun>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.recovery.list_runs(&auth.tenant_id).await?))
}

// --- Phase 14 backups ---

#[derive(Deserialize)]
struct UploadBackupBody {
    object_key: String,
    content_type: Option<String>,
    data_base64: String,
    metadata: Option<serde_json::Value>,
}

async fn upload_backup(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UploadBackupBody>,
) -> Result<(StatusCode, Json<cloud_storage::BackupObject>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD
        .decode(&body.data_base64)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let obj = state
        .backups
        .upload(UploadBackupRequest {
            tenant_id: auth.tenant_id.clone(),
            object_key: body.object_key,
            content_type: body.content_type,
            data,
            metadata: body.metadata,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(obj)))
}

#[derive(Deserialize)]
struct RestoreBackupBody {
    object_key: String,
}

async fn restore_backup(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<RestoreBackupBody>,
) -> Result<Json<RestoreBackupResponse>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    let data = state
        .backups
        .restore(&auth.tenant_id, &body.object_key)
        .await?;
    use base64::Engine;
    Ok(Json(RestoreBackupResponse {
        object_key: body.object_key,
        data_base64: base64::engine::general_purpose::STANDARD.encode(data),
    }))
}

#[derive(Serialize)]
struct RestoreBackupResponse {
    object_key: String,
    data_base64: String,
}

async fn list_backups(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_storage::BackupObject>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.backups.list(&auth.tenant_id).await?))
}

// --- Phase 14 metering / cloud ingest ---

#[derive(Deserialize)]
struct IngestUsageBody {
    metric: String,
    quantity: f64,
    metadata: Option<serde_json::Value>,
}

async fn list_usage(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_metering::UsageAggregate>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state.metering.list_aggregates(&auth.tenant_id).await?,
    ))
}

async fn ingest_usage(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<IngestUsageBody>,
) -> Result<(StatusCode, Json<cloud_metering::UsageSnapshot>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    let metric = UsageMetric::from_str(&body.metric)
        .ok_or_else(|| ApiError::BadRequest("invalid metric".into()))?;
    let snapshot = state
        .metering
        .record(RecordUsageRequest {
            tenant_id: auth.tenant_id.clone(),
            metric,
            quantity: body.quantity,
            metadata: body.metadata,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(snapshot)))
}

#[derive(Deserialize)]
struct IngestHealthBody {
    region_id: String,
    healthy: bool,
    latency_ms: Option<f64>,
    message: Option<String>,
}

async fn ingest_cloud_health(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<IngestHealthBody>,
) -> Result<Json<cloud_regions::RegionHealth>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    Ok(Json(
        state
            .regions
            .record_health(
                &body.region_id,
                body.healthy,
                body.latency_ms,
                body.message.as_deref(),
            )
            .await?,
    ))
}

#[derive(Deserialize)]
struct IngestLogsBody {
    source: String,
    level: Option<String>,
    message: String,
    fields: Option<serde_json::Value>,
}

async fn ingest_cloud_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<IngestLogsBody>,
) -> Result<StatusCode, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    state
        .logging
        .ingest(
            &auth.tenant_id,
            cloud_logging::IngestLogRequest {
                source: body.source,
                level: body.level,
                message: body.message,
                fields: body.fields,
            },
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<LogListParams>,
) -> Result<Json<Vec<cloud_logging::AggregatedLogEntry>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    let limit = params.limit.unwrap_or(100).clamp(1, 500);
    Ok(Json(
        state.logging.list(&auth.tenant_id, limit).await?,
    ))
}

#[derive(Deserialize)]
struct LogListParams {
    limit: Option<i64>,
}

async fn search_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<LogSearchQuery>,
) -> Result<Json<Vec<cloud_logging::AggregatedLogEntry>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(
        state.logging.search(&auth.tenant_id, params).await?,
    ))
}

async fn get_observability_metrics(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    let snapshot = state.observability.observability_snapshot().await?;
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    if accept.contains("text/plain") || accept.contains("application/openmetrics-text") {
        Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
            snapshot.prometheus_text,
        )
            .into_response())
    } else {
        Ok(Json(snapshot).into_response())
    }
}

async fn list_ha_nodes(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<cloud_ha::ClusterNode>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Administrator)?;
    Ok(Json(state.ha.list_nodes().await?))
}

async fn register_ha_node(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<RegisterNodeRequest>,
) -> Result<(StatusCode, Json<cloud_ha::ClusterNode>), ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Owner)?;
    let node = state.ha.register_node(body).await?;
    Ok((StatusCode::CREATED, Json(node)))
}

async fn ha_heartbeat(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<HaHeartbeatResponse>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Operator)?;
    let node = state.ha.heartbeat(&id).await?;
    let leader_event = state.ha.try_acquire_leader(&id).await?;
    let failed = state.ha.detect_failed_nodes().await?;
    Ok(Json(HaHeartbeatResponse {
        node,
        leader_event,
        failed_events: failed,
    }))
}

#[derive(Serialize)]
struct HaHeartbeatResponse {
    node: cloud_ha::ClusterNode,
    leader_event: Option<cloud_events::CloudEvent>,
    failed_events: Vec<cloud_events::CloudEvent>,
}

async fn get_ha_leader(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Option<cloud_ha::ClusterNode>>, ApiError> {
    auth_mw::require_role(&auth.claims, TeamRole::Viewer)?;
    Ok(Json(state.ha.current_leader().await?))
}

async fn openapi_doc() -> Json<openapi::OpenApiDocument> {
    Json(openapi::document())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "wiresentinel-cloud",
    })
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}
