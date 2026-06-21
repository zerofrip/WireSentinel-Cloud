use database::DbError;
use database::DbPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudMetricsSnapshot {
    pub tenants_active: i64,
    pub tenants_isolated: i64,
    pub organizations_total: i64,
    pub teams_total: i64,
    pub users_total: i64,
    pub federated_controllers_total: i64,
    pub sync_conflicts_open: i64,
    pub compliance_reports_total: i64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMetricsSnapshot {
    pub tenant_id: String,
    pub organizations: i64,
    pub teams: i64,
    pub users: i64,
    pub federated_controllers: i64,
    pub open_sync_conflicts: i64,
    pub compliance_reports: i64,
}

pub struct CloudMetricsAggregator {
    pool: DbPool,
    started_at: std::time::Instant,
}

impl CloudMetricsAggregator {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            started_at: std::time::Instant::now(),
        }
    }

    pub async fn snapshot(&self) -> Result<CloudMetricsSnapshot, DbError> {
        let active: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM tenants WHERE status = 'active'")
                .fetch_one(&self.pool)
                .await?;
        let isolated: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM tenants WHERE status = 'isolated'")
                .fetch_one(&self.pool)
                .await?;
        let orgs: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM organizations")
            .fetch_one(&self.pool)
            .await?;
        let teams: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM teams")
            .fetch_one(&self.pool)
            .await?;
        let users: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        let controllers: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM federated_controllers")
                .fetch_one(&self.pool)
                .await?;
        let conflicts: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sync_conflicts WHERE resolved_at IS NULL",
        )
        .fetch_one(&self.pool)
        .await?;
        let compliance: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM compliance_reports")
            .fetch_one(&self.pool)
            .await?;

        Ok(CloudMetricsSnapshot {
            tenants_active: active.0,
            tenants_isolated: isolated.0,
            organizations_total: orgs.0,
            teams_total: teams.0,
            users_total: users.0,
            federated_controllers_total: controllers.0,
            sync_conflicts_open: conflicts.0,
            compliance_reports_total: compliance.0,
            uptime_seconds: self.started_at.elapsed().as_secs(),
        })
    }

    pub async fn tenant_snapshot(&self, tenant_id: &str) -> Result<TenantMetricsSnapshot, DbError> {
        let orgs: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM organizations WHERE tenant_id = ?",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;
        let teams: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM teams WHERE tenant_id = ?")
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await?;
        let users: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM users WHERE tenant_id = ?")
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await?;
        let controllers: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM federated_controllers WHERE tenant_id = ?",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;
        let conflicts: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sync_conflicts WHERE tenant_id = ? AND resolved_at IS NULL",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;
        let compliance: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM compliance_reports WHERE tenant_id = ?",
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TenantMetricsSnapshot {
            tenant_id: tenant_id.to_string(),
            organizations: orgs.0,
            teams: teams.0,
            users: users.0,
            federated_controllers: controllers.0,
            open_sync_conflicts: conflicts.0,
            compliance_reports: compliance.0,
        })
    }

    pub fn to_prometheus(snapshot: &CloudMetricsSnapshot) -> String {
        format!(
            "# HELP ws_cloud_tenants_active Active tenants\n\
             # TYPE ws_cloud_tenants_active gauge\n\
             ws_cloud_tenants_active {}\n\
             # HELP ws_cloud_tenants_isolated Isolated tenants\n\
             # TYPE ws_cloud_tenants_isolated gauge\n\
             ws_cloud_tenants_isolated {}\n\
             # HELP ws_cloud_organizations_total Organizations\n\
             # TYPE ws_cloud_organizations_total gauge\n\
             ws_cloud_organizations_total {}\n\
             # HELP ws_cloud_teams_total Teams\n\
             # TYPE ws_cloud_teams_total gauge\n\
             ws_cloud_teams_total {}\n\
             # HELP ws_cloud_users_total Users\n\
             # TYPE ws_cloud_users_total gauge\n\
             ws_cloud_users_total {}\n\
             # HELP ws_cloud_uptime_seconds Uptime\n\
             # TYPE ws_cloud_uptime_seconds counter\n\
             ws_cloud_uptime_seconds {}\n",
            snapshot.tenants_active,
            snapshot.tenants_isolated,
            snapshot.organizations_total,
            snapshot.teams_total,
            snapshot.users_total,
            snapshot.uptime_seconds,
        )
    }
}
