use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TenantRow {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub isolated_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrganizationRow {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionRow {
    pub id: String,
    pub tenant_id: String,
    pub plan: String,
    pub status: String,
    pub seats: i64,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRow {
    pub id: String,
    pub tenant_id: String,
    pub username: String,
    pub password_hash: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub oidc_sub: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamRow {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamMembershipRow {
    pub id: String,
    pub team_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FederatedControllerRow {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub endpoint_url: String,
    pub api_key_hash: String,
    pub status: String,
    pub last_sync_at: Option<String>,
    pub last_health_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SyncConflictRow {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub entity_type: String,
    pub entity_id: String,
    pub local_payload: String,
    pub remote_payload: String,
    pub resolution: Option<String>,
    pub resolved_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceReportRow {
    pub id: String,
    pub tenant_id: String,
    pub check_type: String,
    pub status: String,
    pub summary: String,
    pub details: String,
    pub created_at: String,
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

pub fn parse_iso(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}
