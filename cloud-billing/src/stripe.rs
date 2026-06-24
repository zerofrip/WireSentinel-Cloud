use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeCheckout {
    pub session_id: String,
    pub url: String,
}

#[async_trait]
pub trait StripeBillingProvider: Send + Sync {
    async fn create_checkout_session(
        &self,
        tenant_id: &str,
        plan_id: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<StripeCheckout, String>;

    async fn handle_webhook(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<StripeWebhookEvent, String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeWebhookEvent {
    pub event_type: String,
    pub tenant_id: Option<String>,
    pub plan_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub stripe_invoice_id: Option<String>,
}

pub struct StripeMockProvider;

#[async_trait]
impl StripeBillingProvider for StripeMockProvider {
    async fn create_checkout_session(
        &self,
        tenant_id: &str,
        plan_id: &str,
        _success_url: &str,
        _cancel_url: &str,
    ) -> Result<StripeCheckout, String> {
        Ok(StripeCheckout {
            session_id: format!("cs_mock_{tenant_id}_{plan_id}"),
            url: format!("https://checkout.stripe.mock/session/{tenant_id}/{plan_id}"),
        })
    }

    async fn handle_webhook(
        &self,
        payload: &[u8],
        _signature: &str,
    ) -> Result<StripeWebhookEvent, String> {
        let value: serde_json::Value =
            serde_json::from_slice(payload).map_err(|e| e.to_string())?;
        Ok(StripeWebhookEvent {
            event_type: value
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("checkout.session.completed")
                .into(),
            tenant_id: value
                .pointer("/data/object/metadata/tenant_id")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            plan_id: value
                .pointer("/data/object/metadata/plan_id")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            stripe_customer_id: value
                .pointer("/data/object/customer")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            stripe_subscription_id: value
                .pointer("/data/object/subscription")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            stripe_invoice_id: None,
        })
    }
}

pub struct StripeProvider {
    secret_key: String,
}

impl StripeProvider {
    pub fn from_env() -> Option<Self> {
        let key = std::env::var("STRIPE_SECRET_KEY").ok()?;
        if key.is_empty() {
            return None;
        }
        Some(Self { secret_key: key })
    }
}

#[async_trait]
impl StripeBillingProvider for StripeProvider {
    async fn create_checkout_session(
        &self,
        tenant_id: &str,
        plan_id: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<StripeCheckout, String> {
        #[cfg(feature = "stripe")]
        {
            use async_stripe::{
                CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession,
                CreateCheckoutSessionLineItems, Currency,
            };

            let client = Client::new(self.secret_key.clone());
            let mut params = CreateCheckoutSession::new();
            params.mode = Some(CheckoutSessionMode::Subscription);
            params.success_url = Some(success_url);
            params.cancel_url = Some(cancel_url);
            params.client_reference_id = Some(tenant_id);
            params.metadata = Some(std::collections::HashMap::from([
                ("tenant_id".into(), tenant_id.into()),
                ("plan_id".into(), plan_id.into()),
            ]));
            params.line_items = Some(vec![CreateCheckoutSessionLineItems {
                quantity: Some(1),
                price_data: Some(async_stripe::CreateCheckoutSessionLineItemsPriceData {
                    currency: Currency::USD,
                    unit_amount: Some(plan_price_cents(plan_id)),
                    product_data: Some(
                        async_stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
                            name: Some(plan_id.into()),
                            ..Default::default()
                        },
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            }]);

            let session = CheckoutSession::create(&client, params)
                .await
                .map_err(|e| e.to_string())?;
            return Ok(StripeCheckout {
                session_id: session.id.to_string(),
                url: session.url.unwrap_or_default(),
            });
        }

        #[cfg(not(feature = "stripe"))]
        {
            let _ = (tenant_id, plan_id, success_url, cancel_url);
            Err("stripe feature not enabled".into())
        }
    }

    async fn handle_webhook(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<StripeWebhookEvent, String> {
        #[cfg(feature = "stripe")]
        {
            use async_stripe::{Client, Webhook};

            let secret = std::env::var("STRIPE_WEBHOOK_SECRET")
                .map_err(|_| "STRIPE_WEBHOOK_SECRET missing".to_string())?;
            let _client = Client::new(self.secret_key.clone());
            let event =
                Webhook::construct_event(payload, signature, &secret).map_err(|e| e.to_string())?;

            return Ok(StripeWebhookEvent {
                event_type: event.type_.to_string(),
                tenant_id: event
                    .data
                    .object
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("tenant_id"))
                    .cloned(),
                plan_id: event
                    .data
                    .object
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("plan_id"))
                    .cloned(),
                stripe_customer_id: event.data.object.customer.map(|c| c.id().to_string()),
                stripe_subscription_id: event.data.object.subscription.map(|s| s.id().to_string()),
                stripe_invoice_id: None,
            });
        }

        #[cfg(not(feature = "stripe"))]
        {
            let _ = (payload, signature);
            Err("stripe feature not enabled".into())
        }
    }
}

fn plan_price_cents(plan_id: &str) -> i64 {
    match plan_id {
        "team" => 2900,
        "enterprise" => 9900,
        "enterprise_plus" => 19900,
        _ => 0,
    }
}

pub fn stripe_provider_from_env() -> Box<dyn StripeBillingProvider> {
    if std::env::var("STRIPE_MOCK").ok().as_deref() == Some("1") {
        return Box::new(StripeMockProvider);
    }
    if let Some(provider) = StripeProvider::from_env() {
        return Box::new(provider);
    }
    Box::new(StripeMockProvider)
}
