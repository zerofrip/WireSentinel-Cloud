use cloud_cnapp::{
    CnappFleetMonitor, CnappRollupPayload, CreateCnappPostureRequest, TenantCnappPolicyService,
};
use database::models::now_iso;

async fn seed_tenant(pool: &database::DbPool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = now_iso();
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, created_at) VALUES (?, 'CNAPP Test', ?, 'active', ?)",
    )
    .bind(&id)
    .bind(format!("cnapp-{id}"))
    .bind(&ts)
    .execute(pool)
    .await
    .expect("tenant");
    id
}

#[tokio::test]
async fn cnapp_fleet_rollup_and_posture_policy() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let tenant_id = seed_tenant(&pool).await;

    TenantCnappPolicyService::new(pool.clone())
        .create_posture(
            &tenant_id,
            CreateCnappPostureRequest {
                cloud_provider: Some("aws".into()),
                account_id: Some("123456789012".into()),
                resource_kind: Some("account".into()),
                posture_score: Some(82.5),
                risk_level: Some("low".into()),
                findings_count: Some(3),
                content: None,
            },
            None,
        )
        .await
        .expect("posture");

    let fleet = CnappFleetMonitor::new(pool);
    fleet
        .record_rollup(
            &tenant_id,
            Some("ctrl-1"),
            &CnappRollupPayload {
                reporting_accounts: 6,
                posture_score: 82.5,
                compliance_pct: 91.0,
                open_vulnerabilities: 14,
                critical_vulnerabilities: 2,
                attack_paths_detected: 3,
                multi_cloud_providers: 2,
                fleet_risk_score: 38.0,
            },
        )
        .await
        .expect("rollup");

    let overview = fleet.fleet_overview(&tenant_id).await.expect("overview");
    assert_eq!(overview.open_vulnerabilities, 14);
    assert_eq!(overview.critical_vulnerabilities, 2);
    assert!((overview.posture_score - 82.5).abs() < f64::EPSILON);
}
