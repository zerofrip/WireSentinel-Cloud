use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub tenant_id: String,
    pub subscription_id: Option<String>,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub stripe_invoice_id: Option<String>,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub created_at: String,
}

pub struct InvoiceManager {
    pool: DbPool,
}

impl InvoiceManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn list_for_tenant(&self, tenant_id: &str) -> Result<Vec<Invoice>, DbError> {
        let rows: Vec<(
            String,
            String,
            Option<String>,
            i64,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, subscription_id, amount_cents, currency, status, stripe_invoice_id, period_start, period_end, created_at FROM invoices WHERE tenant_id = ? ORDER BY created_at DESC",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    tenant_id,
                    subscription_id,
                    amount_cents,
                    currency,
                    status,
                    stripe_invoice_id,
                    period_start,
                    period_end,
                    created_at,
                )| Invoice {
                    id,
                    tenant_id,
                    subscription_id,
                    amount_cents,
                    currency,
                    status,
                    stripe_invoice_id,
                    period_start,
                    period_end,
                    created_at,
                },
            )
            .collect())
    }

    pub async fn create_draft(
        &self,
        tenant_id: &str,
        subscription_id: Option<&str>,
        amount_cents: i64,
    ) -> Result<Invoice, DbError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        sqlx::query(
            "INSERT INTO invoices (id, tenant_id, subscription_id, amount_cents, currency, status, created_at) VALUES (?, ?, ?, ?, 'usd', 'draft', ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(subscription_id)
        .bind(amount_cents)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(Invoice {
            id,
            tenant_id: tenant_id.to_string(),
            subscription_id: subscription_id.map(str::to_string),
            amount_cents,
            currency: "usd".into(),
            status: "draft".into(),
            stripe_invoice_id: None,
            period_start: None,
            period_end: None,
            created_at,
        })
    }

    pub async fn mark_paid(&self, invoice_id: &str, stripe_invoice_id: Option<&str>) -> Result<(), DbError> {
        sqlx::query(
            "UPDATE invoices SET status = 'paid', stripe_invoice_id = COALESCE(?, stripe_invoice_id) WHERE id = ?",
        )
        .bind(stripe_invoice_id)
        .bind(invoice_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
