use cloud_wiresock::{
    WiresockFleetMonitor, WiresockRollupPayload, CreateWiresockSplitTemplateRequest,
    TenantWiresockPolicyService,
};
use database::models::now_iso;

async fn seed_tenant(pool: &database::DbPool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = now_iso();
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, created_at) VALUES (?, 'WireSock Test', ?, 'active', ?)",
    )
    .bind(&id)
    .bind(format!("wiresock-{id}"))
    .bind(&ts)
    .execute(pool)
    .await
    .expect("tenant");
    id
}

#[tokio::test]
async fn wiresock_fleet_rollup_and_split_template_policy() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let tenant_id = seed_tenant(&pool).await;

    TenantWiresockPolicyService::new(pool.clone())
        .create_split_template(
            &tenant_id,
            CreateWiresockSplitTemplateRequest {
                name: Some("Corporate bypass".into()),
                description: Some("Default split tunnel".into()),
                template_mode: Some("merge".into()),
                enabled: Some(true),
                app_rules_count: Some(3),
                domain_rules_count: Some(5),
                content: None,
            },
            None,
        )
        .await
        .expect("split template");

    let fleet = WiresockFleetMonitor::new(pool);
    fleet
        .record_rollup(
            &tenant_id,
            Some("ctrl-1"),
            &WiresockRollupPayload {
                reporting_endpoints: 12,
                active_split_templates: 2,
                tcp_termination_rules: 4,
                handshake_proxy_active: 1,
                bypass_events: 0,
                fleet_health_score: 94.5,
            },
        )
        .await
        .expect("rollup");

    let overview = fleet.fleet_overview(&tenant_id).await.expect("overview");
    assert_eq!(overview.active_split_templates, 2);
    assert_eq!(overview.tcp_termination_rules, 4);
    assert!((overview.fleet_health_score - 94.5).abs() < f64::EPSILON);
    assert_eq!(overview.split_templates.len(), 1);
}
