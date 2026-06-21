use cloud_ai::{
    AiFleetMonitor, AiRollupPayload, CreateAiInvestigationRequest, TenantAiPolicyService,
};
use database::models::now_iso;

async fn seed_tenant(pool: &database::DbPool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let ts = now_iso();
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, created_at) VALUES (?, 'AI Test', ?, 'active', ?)",
    )
    .bind(&id)
    .bind(format!("ai-{id}"))
    .bind(&ts)
    .execute(pool)
    .await
    .expect("tenant");
    id
}

#[tokio::test]
async fn ai_fleet_rollup_and_investigation_policy() {
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let tenant_id = seed_tenant(&pool).await;

    TenantAiPolicyService::new(pool.clone())
        .create_investigation(
            &tenant_id,
            CreateAiInvestigationRequest {
                title: Some("Prompt injection cluster".into()),
                status: None,
                severity: Some("high".into()),
                category: Some("prompt_injection".into()),
                model_name: Some("gpt-4o".into()),
                agent_id: Some("agent-1".into()),
                finding_count: Some(5),
                content: None,
            },
            None,
        )
        .await
        .expect("investigation");

    let fleet = AiFleetMonitor::new(pool);
    fleet
        .record_rollup(
            &tenant_id,
            Some("ctrl-1"),
            &AiRollupPayload {
                reporting_agents: 4,
                open_investigations: 2,
                critical_risks: 1,
                total_correlations: 7,
                compliance_pct: 88.5,
                avg_risk_score: 42.0,
                prompt_injection_events: 3,
                data_exfiltration_events: 1,
                fleet_ai_risk_score: 55.0,
            },
        )
        .await
        .expect("rollup");

    let overview = fleet.fleet_overview(&tenant_id).await.expect("overview");
    assert_eq!(overview.open_investigations, 2);
    assert_eq!(overview.critical_risks, 1);
    assert!((overview.fleet_ai_risk_score - 55.0).abs() < f64::EPSILON);
    assert_eq!(overview.investigations.len(), 1);
}
