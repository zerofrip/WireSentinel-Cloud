use crate::invoices::InvoiceManager;
use crate::plans::PlanManager;
use crate::stripe::{stripe_provider_from_env, StripeBillingProvider, StripeWebhookEvent};
use billing::{CreateSubscriptionRequest, Plan, Subscription, SubscriptionManager};
use cloud_core::{write_audit_event, AuditWriteRequest, CloudSecurityPolicy};
use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum BillingError {
    #[error("billing error: {0}")]
    Message(String),
    #[error("billing security violation: {0}")]
    Security(String),
    #[error("database error: {0}")]
    Db(#[from] DbError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSession {
    pub session_id: String,
    pub checkout_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResult {
    pub handled: bool,
    pub event_type: String,
    pub subscription: Option<Subscription>,
}

pub struct BillingManager {
    pool: DbPool,
    subscriptions: SubscriptionManager,
    pub plans: PlanManager,
    pub invoices: InvoiceManager,
    stripe: Arc<dyn StripeBillingProvider>,
    policy: CloudSecurityPolicy,
}

impl BillingManager {
    pub fn new(pool: DbPool) -> Self {
        let subscriptions = SubscriptionManager::new(pool.clone());
        let plans = PlanManager::new(pool.clone());
        let invoices = InvoiceManager::new(pool.clone());
        Self {
            pool,
            subscriptions,
            plans,
            invoices,
            stripe: Arc::from(stripe_provider_from_env()),
            policy: CloudSecurityPolicy::default(),
        }
    }

    pub fn with_stripe(pool: DbPool, stripe: Arc<dyn StripeBillingProvider>) -> Self {
        let subscriptions = SubscriptionManager::new(pool.clone());
        let plans = PlanManager::new(pool.clone());
        let invoices = InvoiceManager::new(pool.clone());
        Self {
            pool,
            subscriptions,
            plans,
            invoices,
            stripe,
            policy: CloudSecurityPolicy::default(),
        }
    }

    pub async fn list_plans(&self) -> Result<Vec<crate::plans::BillingPlan>, DbError> {
        self.plans.list().await
    }

    pub async fn get_subscription(&self, tenant_id: &str) -> Result<Option<Subscription>, DbError> {
        self.subscriptions.get_for_tenant(tenant_id).await
    }

    pub async fn list_subscriptions(&self, tenant_id: &str) -> Result<Vec<Subscription>, DbError> {
        self.subscriptions.list(tenant_id).await
    }

    pub async fn create_subscription(
        &self,
        req: CreateSubscriptionRequest,
        actor: Option<&str>,
    ) -> Result<Subscription, BillingError> {
        let sub = self.subscriptions.create(req.clone()).await?;
        self.record_event(&sub.tenant_id, "subscription.created", &sub)
            .await?;
        write_audit_event(
            &self.pool,
            AuditWriteRequest {
                tenant_id: sub.tenant_id.clone(),
                source: "cloud-billing".into(),
                actor: actor.map(str::to_string),
                action: "billing.subscription.create".into(),
                resource_type: Some("subscription".into()),
                resource_id: Some(sub.id.clone()),
                details: serde_json::json!({ "plan": req.plan.as_str(), "seats": req.seats }),
            },
        )
        .await?;
        Ok(sub)
    }

    pub async fn list_invoices(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<crate::invoices::Invoice>, DbError> {
        self.invoices.list_for_tenant(tenant_id).await
    }

    pub async fn create_checkout(
        &self,
        tenant_id: &str,
        plan_id: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<CheckoutSession, BillingError> {
        let session = self
            .stripe
            .create_checkout_session(tenant_id, plan_id, success_url, cancel_url)
            .await
            .map_err(BillingError::Message)?;
        Ok(CheckoutSession {
            session_id: session.session_id,
            checkout_url: session.url,
        })
    }

    pub async fn handle_webhook(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<WebhookResult, BillingError> {
        if let Err(reason) = self.policy.validate_billing_webhook(signature) {
            self.record_event_raw(
                "system",
                "billing.security_violation",
                &serde_json::json!({ "reason": reason }),
            )
            .await?;
            return Err(BillingError::Security(reason));
        }

        let event = self
            .stripe
            .handle_webhook(payload, signature)
            .await
            .map_err(BillingError::Security)?;
        let subscription = self.apply_webhook_event(&event).await?;
        Ok(WebhookResult {
            handled: true,
            event_type: event.event_type.clone(),
            subscription,
        })
    }

    async fn apply_webhook_event(
        &self,
        event: &StripeWebhookEvent,
    ) -> Result<Option<Subscription>, BillingError> {
        self.record_event_raw(
            event.tenant_id.as_deref().unwrap_or("unknown"),
            &event.event_type,
            event,
        )
        .await?;

        if let Some(tenant_id) = &event.tenant_id {
            write_audit_event(
                &self.pool,
                AuditWriteRequest {
                    tenant_id: tenant_id.clone(),
                    source: "cloud-billing".into(),
                    actor: None,
                    action: format!("billing.webhook.{}", event.event_type),
                    resource_type: Some("stripe_event".into()),
                    resource_id: event.stripe_subscription_id.clone(),
                    details: serde_json::json!({
                        "stripe_customer_id": event.stripe_customer_id,
                    }),
                },
            )
            .await?;
        }

        let tenant_id = match &event.tenant_id {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        if event.event_type.contains("checkout.session.completed")
            || event.event_type.contains("customer.subscription.created")
        {
            let plan = event
                .plan_id
                .as_deref()
                .and_then(Plan::from_str)
                .unwrap_or(Plan::Team);
            let sub = self
                .subscriptions
                .create(CreateSubscriptionRequest {
                    tenant_id: tenant_id.clone(),
                    plan,
                    seats: None,
                })
                .await?;
            if event.stripe_customer_id.is_some() || event.stripe_subscription_id.is_some() {
                sqlx::query(
                    "UPDATE subscriptions SET stripe_customer_id = COALESCE(?, stripe_customer_id), stripe_subscription_id = COALESCE(?, stripe_subscription_id) WHERE id = ?",
                )
                .bind(&event.stripe_customer_id)
                .bind(&event.stripe_subscription_id)
                .bind(&sub.id)
                .execute(&self.pool)
                .await
                .map_err(DbError::from)?;
            }
            if let Some(invoice_id) = &event.stripe_invoice_id {
                let draft = self
                    .invoices
                    .create_draft(&tenant_id, Some(&sub.id), 0)
                    .await?;
                self.invoices.mark_paid(&draft.id, Some(invoice_id)).await?;
            }
            return Ok(Some(sub));
        }
        Ok(None)
    }

    async fn record_event<T: Serialize>(
        &self,
        tenant_id: &str,
        event_type: &str,
        payload: &T,
    ) -> Result<(), BillingError> {
        self.record_event_raw(tenant_id, event_type, payload).await
    }

    async fn record_event_raw<T: Serialize>(
        &self,
        tenant_id: &str,
        event_type: &str,
        payload: &T,
    ) -> Result<(), BillingError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        let body = serde_json::to_string(payload).unwrap_or_else(|_| "{}".into());
        sqlx::query(
            "INSERT INTO billing_events (id, tenant_id, event_type, payload, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(event_type)
        .bind(&body)
        .bind(&created_at)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;
        Ok(())
    }
}
