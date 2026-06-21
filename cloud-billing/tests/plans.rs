use cloud_billing::PlanManager;
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
async fn plan_manager_includes_enterprise_plus() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let _tenant = seed_tenant(&pool).await;
    let plans = PlanManager::new(pool).list().await.expect("plans");
    assert!(plans.iter().any(|p| p.id == "enterprise_plus"));
}
