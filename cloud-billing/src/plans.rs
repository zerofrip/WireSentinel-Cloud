use billing::{Plan, PlanInfo, PlanLimits, SubscriptionManager};
use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingPlan {
    pub id: String,
    pub name: String,
    pub tier: String,
    pub price_cents: i64,
    pub currency: String,
    pub limits: PlanLimits,
    pub active: bool,
}

pub struct PlanManager {
    pool: DbPool,
}

impl PlanManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<BillingPlan>, DbError> {
        let rows: Vec<(String, String, String, i64, String, String, i64)> = sqlx::query_as(
            "SELECT id, name, tier, price_cents, currency, limits_json, active FROM billing_plans WHERE active = 1 ORDER BY price_cents",
        )
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Ok(SubscriptionManager::list_plans()
                .into_iter()
                .map(Self::from_plan_info)
                .collect());
        }

        Ok(rows
            .into_iter()
            .map(
                |(id, name, tier, price_cents, currency, limits_json, active)| BillingPlan {
                    id,
                    name,
                    tier,
                    price_cents,
                    currency,
                    limits: serde_json::from_str(&limits_json).unwrap_or_else(|_| Plan::Free.limits()),
                    active: active != 0,
                },
            )
            .collect())
    }

    pub async fn get(&self, plan_id: &str) -> Result<Option<BillingPlan>, DbError> {
        Ok(self
            .list()
            .await?
            .into_iter()
            .find(|p| p.id == plan_id))
    }

    fn from_plan_info(info: PlanInfo) -> BillingPlan {
        let id = info.id.clone();
        BillingPlan {
            id: id.clone(),
            name: info.name,
            tier: id.clone(),
            price_cents: match id.as_str() {
                "team" => 2900,
                "enterprise" => 9900,
                "enterprise_plus" => 19900,
                _ => 0,
            },
            currency: "usd".into(),
            limits: info.limits,
            active: true,
        }
    }

    pub async fn seed_defaults(&self) -> Result<(), DbError> {
        let created_at = now_iso();
        for info in SubscriptionManager::list_plans() {
            let plan = Self::from_plan_info(info);
            sqlx::query(
                "INSERT OR IGNORE INTO billing_plans (id, name, tier, price_cents, currency, limits_json, active, created_at) VALUES (?, ?, ?, ?, ?, ?, 1, ?)",
            )
            .bind(&plan.id)
            .bind(&plan.name)
            .bind(&plan.tier)
            .bind(plan.price_cents)
            .bind(&plan.currency)
            .bind(serde_json::to_string(&plan.limits).unwrap_or_else(|_| "{}".into()))
            .bind(&created_at)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}
