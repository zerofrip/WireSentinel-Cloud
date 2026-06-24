use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Free,
    Team,
    Enterprise,
    #[serde(rename = "enterprise_plus")]
    EnterprisePlus,
}

impl Plan {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Team => "team",
            Self::Enterprise => "enterprise",
            Self::EnterprisePlus => "enterprise_plus",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "free" => Some(Self::Free),
            "team" => Some(Self::Team),
            "enterprise" => Some(Self::Enterprise),
            "enterprise_plus" => Some(Self::EnterprisePlus),
            _ => None,
        }
    }

    pub fn limits(self) -> PlanLimits {
        match self {
            Self::Free => PlanLimits {
                max_users: 5,
                max_teams: 2,
                max_controllers: 1,
            },
            Self::Team => PlanLimits {
                max_users: 50,
                max_teams: 20,
                max_controllers: 5,
            },
            Self::Enterprise => PlanLimits {
                max_users: 10_000,
                max_teams: 1_000,
                max_controllers: 100,
            },
            Self::EnterprisePlus => PlanLimits {
                max_users: 50_000,
                max_teams: 5_000,
                max_controllers: 500,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanLimits {
    pub max_users: i64,
    pub max_teams: i64,
    pub max_controllers: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanInfo {
    pub id: String,
    pub name: String,
    pub limits: PlanLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub tenant_id: String,
    pub plan: String,
    pub status: String,
    pub seats: i64,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub tenant_id: String,
    pub plan: Plan,
    pub seats: Option<i64>,
}

#[derive(Debug, Error)]
pub enum QuotaError {
    #[error("quota exceeded: {0}")]
    Exceeded(String),
    #[error("database error: {0}")]
    Db(#[from] DbError),
}

pub struct SubscriptionManager {
    pool: DbPool,
}

impl SubscriptionManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn list_plans() -> Vec<PlanInfo> {
        [
            Plan::Free,
            Plan::Team,
            Plan::Enterprise,
            Plan::EnterprisePlus,
        ]
        .into_iter()
        .map(|p| PlanInfo {
            id: p.as_str().into(),
            name: match p {
                Plan::Free => "Free".into(),
                Plan::Team => "Team".into(),
                Plan::Enterprise => "Enterprise".into(),
                Plan::EnterprisePlus => "Enterprise Plus".into(),
            },
            limits: p.limits(),
        })
        .collect()
    }

    pub async fn create(&self, req: CreateSubscriptionRequest) -> Result<Subscription, DbError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        let seats = req.seats.unwrap_or(1);
        sqlx::query(
            "INSERT INTO subscriptions (id, tenant_id, plan, status, seats, created_at) VALUES (?, ?, ?, 'active', ?, ?)",
        )
        .bind(&id)
        .bind(&req.tenant_id)
        .bind(req.plan.as_str())
        .bind(seats)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(Subscription {
            id,
            tenant_id: req.tenant_id,
            plan: req.plan.as_str().into(),
            status: "active".into(),
            seats,
            expires_at: None,
            created_at,
        })
    }

    pub async fn get_for_tenant(&self, tenant_id: &str) -> Result<Option<Subscription>, DbError> {
        let row: Option<(String, String, String, String, i64, Option<String>, String)> =
            sqlx::query_as(
                "SELECT id, tenant_id, plan, status, seats, expires_at, created_at FROM subscriptions WHERE tenant_id = ? AND status = 'active' ORDER BY created_at DESC LIMIT 1",
            )
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(
            |(id, tenant_id, plan, status, seats, expires_at, created_at)| Subscription {
                id,
                tenant_id,
                plan,
                status,
                seats,
                expires_at,
                created_at,
            },
        ))
    }

    pub async fn list(&self, tenant_id: &str) -> Result<Vec<Subscription>, DbError> {
        let rows: Vec<(String, String, String, String, i64, Option<String>, String)> =
            sqlx::query_as(
                "SELECT id, tenant_id, plan, status, seats, expires_at, created_at FROM subscriptions WHERE tenant_id = ? ORDER BY created_at DESC",
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, tenant_id, plan, status, seats, expires_at, created_at)| Subscription {
                    id,
                    tenant_id,
                    plan,
                    status,
                    seats,
                    expires_at,
                    created_at,
                },
            )
            .collect())
    }

    pub async fn enforce_user_quota(&self, tenant_id: &str) -> Result<(), QuotaError> {
        let sub = self
            .get_for_tenant(tenant_id)
            .await?
            .unwrap_or(Subscription {
                id: String::new(),
                tenant_id: tenant_id.to_string(),
                plan: Plan::Free.as_str().into(),
                status: "active".into(),
                seats: 1,
                expires_at: None,
                created_at: String::new(),
            });
        let plan = Plan::from_str(&sub.plan).unwrap_or(Plan::Free);
        let limits = plan.limits();
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE tenant_id = ?")
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
            .map_err(DbError::from)?;
        if count.0 >= limits.max_users {
            return Err(QuotaError::Exceeded(format!(
                "user limit {} reached for plan {}",
                limits.max_users, sub.plan
            )));
        }
        Ok(())
    }

    pub async fn enforce_team_quota(&self, tenant_id: &str) -> Result<(), QuotaError> {
        let sub = self.get_for_tenant(tenant_id).await?;
        let plan = sub
            .as_ref()
            .and_then(|s| Plan::from_str(&s.plan))
            .unwrap_or(Plan::Free);
        let limits = plan.limits();
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM teams WHERE tenant_id = ?")
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
            .map_err(DbError::from)?;
        if count.0 >= limits.max_teams {
            return Err(QuotaError::Exceeded(format!(
                "team limit {} reached",
                limits.max_teams
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_limits_ordering() {
        assert!(Plan::Enterprise.limits().max_users > Plan::Free.limits().max_users);
    }
}
