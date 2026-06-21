use cloud_provisioning::{HostedControllerManager, ProvisionRequest};
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
async fn provision_creates_controller_and_job() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let tenant_id = seed_tenant(&pool).await;
    let mgr = HostedControllerManager::new(pool);

    let (controller, job) = mgr
        .provision(ProvisionRequest {
            tenant_id: tenant_id.clone(),
            name: "hosted-1".into(),
            region_id: "us-east".into(),
            plan_tier: "team".into(),
        })
        .await
        .expect("provision");

    assert_eq!(controller.tenant_id, tenant_id);
    assert_eq!(controller.status, "active");
    assert_eq!(job.job_type, "provision");
}
