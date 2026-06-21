use cloud_xdr::{
    CreateXdrHuntRequest, XdrFleetMonitor, XdrRollupPayload, TenantXdrPolicyService,
};
use database::models::now_iso;

async fn seed_tenant(pool: &database::DbPool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = now_iso();
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, created_at) VALUES (?, 'XDR Test', ?, 'active', ?)",
    )
    .bind(&id)
    .bind(format!("xdr-{id}"))
    .bind(&ts)
    .execute(pool)
    .await
    .expect("tenant");
    id
}

#[tokio::test]
async fn xdr_fleet_rollup_and_hunt_policy() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let tenant_id = seed_tenant(&pool).await;

    TenantXdrPolicyService::new(pool.clone())
        .create_hunt(
            &tenant_id,
            CreateXdrHuntRequest {
                name: "lateral-movement-hunt".into(),
                query_kind: Some("behavioral".into()),
                status: Some("draft".into()),
                enabled: Some(true),
                query: None,
            },
            None,
        )
        .await
        .expect("hunt");

    let fleet = XdrFleetMonitor::new(pool);
    fleet
        .record_rollup(
            &tenant_id,
            Some("ctrl-1"),
            &XdrRollupPayload {
                reporting_devices: 8,
                total_incidents: 5,
                open_incidents: 2,
                critical_incidents: 1,
                total_detections: 42,
                active_hunts: 1,
                mitre_techniques_detected: 12,
                mitre_coverage_pct: 68.5,
                avg_incident_mttr_hours: 4.2,
                fleet_threat_score: 72.0,
            },
        )
        .await
        .expect("rollup");

    let overview = fleet.fleet_overview(&tenant_id).await.expect("overview");
    assert_eq!(overview.total_incidents, 5);
    assert_eq!(overview.total_detections, 42);
    assert_eq!(overview.critical_incidents, 1);
}
