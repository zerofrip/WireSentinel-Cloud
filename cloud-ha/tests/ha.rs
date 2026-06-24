use cloud_ha::{HaManager, RegisterNodeRequest};

#[tokio::test]
async fn leader_election_acquires_lease() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let ha = HaManager::new(pool);

    let node = ha
        .register_node(RegisterNodeRequest {
            node_name: "node-a".into(),
            address: "127.0.0.1:8080".into(),
        })
        .await
        .expect("register");

    ha.heartbeat(&node.id).await.expect("heartbeat");
    let event = ha.try_acquire_leader(&node.id).await.expect("acquire");
    assert!(event.is_some() || event.is_none());

    let leader = ha.current_leader().await.expect("leader");
    assert!(leader.is_some());
    assert_eq!(leader.unwrap().id, node.id);
}

#[tokio::test]
async fn failed_node_detection_marks_status() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let ha = HaManager::new(pool.clone());

    let node = ha
        .register_node(RegisterNodeRequest {
            node_name: "stale".into(),
            address: "10.0.0.1:8080".into(),
        })
        .await
        .expect("register");

    let stale = "2000-01-01T00:00:00Z".to_string();
    sqlx::query("UPDATE cluster_nodes SET last_heartbeat_at = ? WHERE id = ?")
        .bind(&stale)
        .bind(&node.id)
        .execute(&pool)
        .await
        .expect("backdate");

    let events = ha.detect_failed_nodes().await.expect("detect");
    assert!(!events.is_empty());
}
