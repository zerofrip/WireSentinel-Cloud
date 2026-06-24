use cloud_sse::{
    CreateSsePolicyRequest, SseFleetMonitor, SseRollupPayload, TenantSsePolicyService,
};
use database::models::now_iso;

async fn seed_tenant(pool: &database::DbPool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = now_iso();
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, created_at) VALUES (?, 'SSE Test', ?, 'active', ?)",
    )
    .bind(&id)
    .bind(format!("sse-{id}"))
    .bind(&ts)
    .execute(pool)
    .await
    .expect("tenant");
    id
}

#[tokio::test]
async fn sse_fleet_rollup_and_policy() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let tenant_id = seed_tenant(&pool).await;

    TenantSsePolicyService::new(pool.clone())
        .create(
            &tenant_id,
            CreateSsePolicyRequest {
                name: "default-swg".into(),
                policy_kind: Some("swg".into()),
                enabled: Some(true),
                rules: None,
                default_action: Some("block".into()),
            },
            None,
        )
        .await
        .expect("policy");

    let fleet = SseFleetMonitor::new(pool);
    fleet
        .record_rollup(
            &tenant_id,
            Some("ctrl-1"),
            &SseRollupPayload {
                reporting_devices: 5,
                swg_requests: 100,
                swg_blocked: 12,
                threat_count: 3,
                casb_incidents: 2,
                dlp_incidents: 1,
                avg_risk_score: 45.0,
                ueba_alerts: 4,
            },
        )
        .await
        .expect("rollup");

    let overview = fleet.fleet_overview(&tenant_id).await.expect("overview");
    assert_eq!(overview.swg_blocked, 12);
    assert_eq!(overview.threat_count, 3);
}
