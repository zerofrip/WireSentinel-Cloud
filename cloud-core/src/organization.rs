use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrganizationRequest {
    pub tenant_id: String,
    pub name: String,
}

pub struct OrganizationManager {
    pool: DbPool,
}

impl OrganizationManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateOrganizationRequest) -> Result<Organization, DbError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        sqlx::query(
            "INSERT INTO organizations (id, tenant_id, name, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&req.tenant_id)
        .bind(&req.name)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(Organization {
            id,
            tenant_id: req.tenant_id,
            name: req.name,
            created_at,
        })
    }

    pub async fn get(&self, tenant_id: &str, id: &str) -> Result<Organization, DbError> {
        let row: Option<(String, String, String, String)> = sqlx::query_as(
            "SELECT id, tenant_id, name, created_at FROM organizations WHERE id = ? AND tenant_id = ?",
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        let (id, tenant_id, name, created_at) =
            row.ok_or_else(|| DbError::NotFound(format!("organization {id}")))?;

        Ok(Organization {
            id,
            tenant_id,
            name,
            created_at,
        })
    }

    pub async fn list(&self, tenant_id: &str) -> Result<Vec<Organization>, DbError> {
        let rows: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT id, tenant_id, name, created_at FROM organizations WHERE tenant_id = ? ORDER BY created_at DESC",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, tenant_id, name, created_at)| Organization {
                id,
                tenant_id,
                name,
                created_at,
            })
            .collect())
    }
}
