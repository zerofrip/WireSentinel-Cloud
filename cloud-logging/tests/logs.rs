use cloud_logging::{IngestLogRequest, LogAggregationService, LogSearchQuery};
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
async fn ingest_and_search_logs() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let tenant_id = seed_tenant(&pool).await;
    let logs = LogAggregationService::new(pool);

    logs.ingest(
        &tenant_id,
        IngestLogRequest {
            source: "test".into(),
            level: Some("info".into()),
            message: "phase14 log entry".into(),
            fields: None,
        },
    )
    .await
    .expect("ingest");

    let found = logs
        .search(
            &tenant_id,
            LogSearchQuery {
                q: Some("phase14".into()),
                level: None,
                source: None,
                limit: Some(10),
            },
        )
        .await
        .expect("search");
    assert!(!found.is_empty());
}
