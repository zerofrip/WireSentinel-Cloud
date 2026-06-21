use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntent {
    pub id: String,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
}

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_checkout(
        &self,
        tenant_id: &str,
        plan: &str,
    ) -> Result<PaymentIntent, String>;
    async fn cancel_subscription(&self, subscription_id: &str) -> Result<(), String>;
}

pub struct StubPaymentProvider;

#[async_trait]
impl PaymentProvider for StubPaymentProvider {
    async fn create_checkout(
        &self,
        tenant_id: &str,
        plan: &str,
    ) -> Result<PaymentIntent, String> {
        Ok(PaymentIntent {
            id: format!("pi_stub_{tenant_id}"),
            amount_cents: 0,
            currency: "usd".into(),
            status: format!("pending:{plan}"),
        })
    }

    async fn cancel_subscription(&self, _subscription_id: &str) -> Result<(), String> {
        Ok(())
    }
}
