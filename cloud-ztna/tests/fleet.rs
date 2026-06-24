use cloud_ztna::{
    CreatePublishedResourceRequest, ResourcePublisher, ZtnaFleetMonitor, ZtnaRollupPayload,
};
use database::models::now_iso;

async fn seed_tenant(pool: &database::DbPool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = now_iso();
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, created_at) VALUES (?, 'ZTNA Test', ?, 'active', ?)",
    )
    .bind(&id)
    .bind(format!("ztna-{id}"))
    .bind(&ts)
    .execute(pool)
    .await
    .expect("tenant");
    id
}

#[tokio::test]
async fn fleet_rollup_and_published_resources() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let tenant_id = seed_tenant(&pool).await;

    let publisher = ResourcePublisher::new(pool.clone());
    publisher
        .create(
            &tenant_id,
            CreatePublishedResourceRequest {
                name: "internal-crm".into(),
                resource_type: Some("web_app".into()),
                host: "10.0.0.5".into(),
                port: Some(8080),
                path_prefix: None,
                tags: None,
                published: Some(true),
                access_policy_id: None,
            },
            None,
        )
        .await
        .expect("publish");

    let listed = publisher.list(&tenant_id).await.expect("list");
    assert_eq!(listed.len(), 1);

    let fleet = ZtnaFleetMonitor::new(pool);
    fleet
        .record_rollup(
            &tenant_id,
            Some("ctrl-1"),
            &ZtnaRollupPayload {
                reporting_devices: 3,
                avg_trust_score: 72.0,
                allow_count: 10,
                deny_count: 2,
                challenge_count: 1,
                published_resources: 1,
            },
        )
        .await
        .expect("rollup");

    let overview = fleet.fleet_overview(&tenant_id).await.expect("overview");
    assert_eq!(overview.published_resources, 1);
    assert_eq!(overview.allow_count, 10);
}
