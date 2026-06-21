use cloud_quotas::{QuotaManager, SetQuotaRequest};
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
async fn controller_quota_hard_limit_blocks() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let tenant_id = seed_tenant(&pool).await;
    let quotas = QuotaManager::new(pool.clone());

    quotas
        .set_quota(
            &tenant_id,
            SetQuotaRequest {
                resource: "controllers".into(),
                soft_limit: 1.0,
                hard_limit: 1.0,
            },
        )
        .await
        .expect("set quota");

    let ts = now_iso();
    sqlx::query(
        "INSERT INTO hosted_controllers (id, tenant_id, name, region_id, plan_tier, status, created_at, updated_at) VALUES ('c1', ?, 'ctrl', 'us-east', 'team', 'active', ?, ?)",
    )
    .bind(&tenant_id)
    .bind(&ts)
    .bind(&ts)
    .execute(&pool)
    .await
    .expect("insert controller");

    let err = quotas
        .enforce_controller_quota(&tenant_id)
        .await
        .expect_err("should exceed");
    assert!(err.to_string().contains("quota exceeded"));
}
