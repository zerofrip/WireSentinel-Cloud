use cloud_observability::TelemetryPipeline;
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
async fn telemetry_pipeline_exports_prometheus() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let _tenant = seed_tenant(&pool).await;
    let pipeline = TelemetryPipeline::new(pool);

    let snap = pipeline.observability_snapshot().await.expect("snap");
    assert!(snap.prometheus_text.contains("ws_cloud"));
}
