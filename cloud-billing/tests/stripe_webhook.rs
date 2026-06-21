use cloud_billing::{BillingManager, StripeBillingProvider, StripeMockProvider};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn mock_checkout_session_url() {
    let provider = StripeMockProvider;
    let checkout = provider
        .create_checkout_session("tenant-1", "team", "https://ok", "https://cancel")
        .await
        .expect("checkout");
    assert!(checkout.url.contains("tenant-1"));
    assert!(checkout.session_id.starts_with("cs_mock_"));
}

#[tokio::test]
async fn mock_webhook_parses_metadata() {
    let provider = StripeMockProvider;
    let payload = json!({
        "type": "checkout.session.completed",
        "data": {
            "object": {
                "metadata": { "tenant_id": "t1", "plan_id": "enterprise_plus" },
                "customer": "cus_x",
                "subscription": "sub_x"
            }
        }
    });
    let event = provider
        .handle_webhook(payload.to_string().as_bytes(), "sig")
        .await
        .expect("webhook");
    assert_eq!(event.event_type, "checkout.session.completed");
    assert_eq!(event.tenant_id.as_deref(), Some("t1"));
    assert_eq!(event.plan_id.as_deref(), Some("enterprise_plus"));
}

#[tokio::test]
async fn billing_manager_checkout_uses_mock() {
    std::env::set_var("STRIPE_MOCK", "1");
    let pool = database::setup("sqlite::memory:").await.expect("db");
    let billing = BillingManager::with_stripe(pool, Arc::new(StripeMockProvider));
    let session = billing
        .create_checkout("tenant-x", "team", "https://ok", "https://cancel")
        .await
        .expect("session");
    assert!(session.checkout_url.contains("tenant-x"));
}
