use cloud_regions::RegionManager;

#[tokio::test]
async fn regions_seeded_after_migration() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let regions = RegionManager::new(pool)
        .list_regions()
        .await
        .expect("regions");
    assert!(!regions.is_empty());
    assert!(regions
        .iter()
        .any(|r| r.id == "us-east" || r.name == "us-east"));
}
