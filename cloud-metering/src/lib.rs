use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UsageMetric {
    BandwidthBytes,
    ApiRequests,
    ActiveDevices,
    StorageBytes,
}

impl UsageMetric {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "bandwidth_bytes" => Some(Self::BandwidthBytes),
            "api_requests" => Some(Self::ApiRequests),
            "active_devices" => Some(Self::ActiveDevices),
            "storage_bytes" => Some(Self::StorageBytes),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::BandwidthBytes => "bandwidth_bytes",
            Self::ApiRequests => "api_requests",
            Self::ActiveDevices => "active_devices",
            Self::StorageBytes => "storage_bytes",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordUsageRequest {
    pub tenant_id: String,
    pub metric: UsageMetric,
    pub quantity: f64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub id: String,
    pub tenant_id: String,
    pub metric: String,
    pub value: f64,
    pub window_start: String,
    pub window_end: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAggregate {
    pub tenant_id: String,
    pub metric: String,
    pub period: String,
    pub total: f64,
    pub peak: f64,
    pub sample_count: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingSummary {
    pub tenant_id: String,
    pub mrr_cents: i64,
    pub active_subscriptions: i64,
    pub paid_invoices_cents: i64,
    pub bandwidth_bytes: f64,
}

#[derive(Debug, Error)]
pub enum MeteringError {
    #[error("invalid metric")]
    InvalidMetric,
    #[error("database error: {0}")]
    Db(#[from] DbError),
}

pub struct UsageMeteringService {
    pool: DbPool,
}

impl UsageMeteringService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn record(&self, req: RecordUsageRequest) -> Result<UsageSnapshot, MeteringError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        let window_end = created_at.clone();
        let window_start = created_at.clone();

        sqlx::query(
            "INSERT INTO usage_snapshots (id, tenant_id, metric, value, window_start, window_end, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&req.tenant_id)
        .bind(req.metric.as_str())
        .bind(req.quantity)
        .bind(&window_start)
        .bind(&window_end)
        .bind(&created_at)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        sqlx::query(
            "INSERT INTO usage_records (id, tenant_id, metric, quantity, recorded_at, metadata) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&req.tenant_id)
        .bind(req.metric.as_str())
        .bind(req.quantity)
        .bind(&created_at)
        .bind(req.metadata.unwrap_or(serde_json::json!({})).to_string())
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        let period = created_at[..7].to_string();
        self.aggregate(&req.tenant_id, req.metric.as_str(), &period, req.quantity)
            .await?;

        Ok(UsageSnapshot {
            id,
            tenant_id: req.tenant_id,
            metric: req.metric.as_str().into(),
            value: req.quantity,
            window_start,
            window_end,
            created_at,
        })
    }

    pub async fn list_aggregates(&self, tenant_id: &str) -> Result<Vec<UsageAggregate>, DbError> {
        let rows: Vec<(String, String, String, f64, f64, i64, String)> = sqlx::query_as(
            "SELECT tenant_id, metric, period, total, peak, sample_count, updated_at FROM usage_aggregates WHERE tenant_id = ? ORDER BY updated_at DESC",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(tenant_id, metric, period, total, peak, sample_count, updated_at)| {
                    UsageAggregate {
                        tenant_id,
                        metric,
                        period,
                        total,
                        peak,
                        sample_count,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    async fn aggregate(
        &self,
        tenant_id: &str,
        metric: &str,
        period: &str,
        quantity: f64,
    ) -> Result<(), MeteringError> {
        let id = Uuid::new_v4().to_string();
        let updated_at = now_iso();
        sqlx::query(
            "INSERT INTO usage_aggregates (id, tenant_id, metric, period, total, peak, sample_count, updated_at) VALUES (?, ?, ?, ?, ?, ?, 1, ?) \
             ON CONFLICT(tenant_id, metric, period) DO UPDATE SET total = total + excluded.total, peak = MAX(peak, excluded.peak), sample_count = sample_count + 1, updated_at = excluded.updated_at",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(metric)
        .bind(period)
        .bind(quantity)
        .bind(quantity)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;
        Ok(())
    }

    pub async fn billing_summary(&self, tenant_id: &str) -> Result<BillingSummary, DbError> {
        let subs: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM subscriptions WHERE tenant_id = ? AND status = 'active'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let paid: (i64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(amount_cents), 0) FROM invoices WHERE tenant_id = ? AND status = 'paid'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let plan_row: Option<(String,)> = sqlx::query_as(
            "SELECT plan FROM subscriptions WHERE tenant_id = ? AND status = 'active' ORDER BY created_at DESC LIMIT 1",
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        let mrr_cents = plan_row.map(|(plan,)| plan_mrr_cents(&plan)).unwrap_or(0);

        let bandwidth: (f64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(total), 0) FROM usage_aggregates WHERE tenant_id = ? AND metric = 'bandwidth_bytes'",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(BillingSummary {
            tenant_id: tenant_id.to_string(),
            mrr_cents,
            active_subscriptions: subs.0,
            paid_invoices_cents: paid.0,
            bandwidth_bytes: bandwidth.0,
        })
    }
}

fn plan_mrr_cents(plan: &str) -> i64 {
    match plan {
        "team" => 2900,
        "enterprise" => 9900,
        "enterprise_plus" => 19900,
        _ => 0,
    }
}
