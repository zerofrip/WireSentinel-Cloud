use cloud_metering::{RecordUsageRequest, UsageMeteringService, UsageMetric};
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
async fn record_usage_creates_aggregate() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let tenant_id = seed_tenant(&pool).await;
    let metering = UsageMeteringService::new(pool);

    let snap = metering
        .record(RecordUsageRequest {
            tenant_id: tenant_id.clone(),
            metric: UsageMetric::BandwidthBytes,
            quantity: 1024.0,
            metadata: None,
        })
        .await
        .expect("record");
    assert_eq!(snap.value, 1024.0);

    let aggs = metering.list_aggregates(&tenant_id).await.expect("aggs");
    assert!(!aggs.is_empty());
    assert!(aggs[0].total >= 1024.0);
}
