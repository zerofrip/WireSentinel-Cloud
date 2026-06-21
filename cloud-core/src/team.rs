use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    pub team_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTeamRequest {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMembershipRequest {
    pub user_id: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignDeviceRequest {
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignPolicyRequest {
    pub policy_id: String,
}

pub struct TeamManager {
    pool: DbPool,
}

impl TeamManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateTeamRequest) -> Result<Team, DbError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        sqlx::query(
            "INSERT INTO teams (id, tenant_id, organization_id, name, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&req.tenant_id)
        .bind(&req.organization_id)
        .bind(&req.name)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(Team {
            id,
            tenant_id: req.tenant_id,
            organization_id: req.organization_id,
            name: req.name,
            created_at,
        })
    }

    pub async fn list(&self, tenant_id: &str) -> Result<Vec<Team>, DbError> {
        let rows: Vec<(String, String, Option<String>, String, String)> = sqlx::query_as(
            "SELECT id, tenant_id, organization_id, name, created_at FROM teams WHERE tenant_id = ? ORDER BY created_at DESC",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, tenant_id, organization_id, name, created_at)| Team {
                id,
                tenant_id,
                organization_id,
                name,
                created_at,
            })
            .collect())
    }

    pub async fn get(&self, tenant_id: &str, id: &str) -> Result<Team, DbError> {
        let row: Option<(String, String, Option<String>, String, String)> = sqlx::query_as(
            "SELECT id, tenant_id, organization_id, name, created_at FROM teams WHERE id = ? AND tenant_id = ?",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        let (id, tenant_id, organization_id, name, created_at) =
            row.ok_or_else(|| DbError::NotFound(format!("team {id}")))?;

        Ok(Team {
            id,
            tenant_id,
            organization_id,
            name,
            created_at,
        })
    }

    pub async fn add_member(
        &self,
        tenant_id: &str,
        team_id: &str,
        req: TeamMembershipRequest,
    ) -> Result<TeamMember, DbError> {
        self.get(tenant_id, team_id).await?;
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        sqlx::query(
            "INSERT INTO team_memberships (id, team_id, user_id, role, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(team_id)
        .bind(&req.user_id)
        .bind(&req.role)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(TeamMember {
            id,
            team_id: team_id.to_string(),
            user_id: req.user_id,
            role: req.role,
            created_at,
        })
    }

    pub async fn list_members(&self, tenant_id: &str, team_id: &str) -> Result<Vec<TeamMember>, DbError> {
        self.get(tenant_id, team_id).await?;
        let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, team_id, user_id, role, created_at FROM team_memberships WHERE team_id = ?",
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, team_id, user_id, role, created_at)| TeamMember {
                id,
                team_id,
                user_id,
                role,
                created_at,
            })
            .collect())
    }

    pub async fn assign_device(
        &self,
        tenant_id: &str,
        team_id: &str,
        req: AssignDeviceRequest,
    ) -> Result<(), DbError> {
        self.get(tenant_id, team_id).await?;
        let id = Uuid::new_v4().to_string();
        let assigned_at = now_iso();
        sqlx::query(
            "INSERT INTO team_devices (id, team_id, device_id, assigned_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(team_id)
        .bind(&req.device_id)
        .bind(&assigned_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn assign_policy(
        &self,
        tenant_id: &str,
        team_id: &str,
        req: AssignPolicyRequest,
    ) -> Result<(), DbError> {
        self.get(tenant_id, team_id).await?;
        let id = Uuid::new_v4().to_string();
        let assigned_at = now_iso();
        sqlx::query(
            "INSERT INTO team_policies (id, team_id, policy_id, assigned_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(team_id)
        .bind(&req.policy_id)
        .bind(&assigned_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
