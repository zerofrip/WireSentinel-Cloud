use cloud_recovery::{CreateRecoveryPlanRequest, DisasterRecoveryManager, RunRecoveryRequest};
use database::models::now_iso;

async fn seed_tenant(pool: &database::DbPool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = now_iso();
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, created_at) VALUES (?, 'Test', ?, 'active', ?)",
    )
    .bind(&id)
    .bind(format!("slug-{id}"))
    .bind(&ts)
    .execute(pool)
    .await
    .expect("tenant");
    id
}

#[tokio::test]
async fn recovery_run_completes() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let tenant_id = seed_tenant(&pool).await;
    let mgr = DisasterRecoveryManager::new(pool);

    let plan = mgr
        .create_plan(CreateRecoveryPlanRequest {
            tenant_id: tenant_id.clone(),
            name: "failover-us-west".into(),
            plan_type: Some("region".into()),
            target_region_id: Some("us-west".into()),
            steps: None,
        })
        .await
        .expect("plan");

    let run = mgr
        .run_recovery(RunRecoveryRequest {
            tenant_id,
            plan_id: plan.id,
        })
        .await
        .expect("run");
    assert_eq!(run.status, "completed");
}
