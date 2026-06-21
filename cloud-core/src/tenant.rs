use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub isolated_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub slug: String,
}

pub struct TenantManager {
    pool: DbPool,
}

impl TenantManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    pub async fn create(&self, req: CreateTenantRequest) -> Result<Tenant, DbError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        sqlx::query(
            "INSERT INTO tenants (id, name, slug, status, created_at) VALUES (?, ?, ?, 'active', ?)",
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.slug)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO tenant_configs (tenant_id, config, updated_at) VALUES (?, '{}', ?)",
        )
        .bind(&id)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(Tenant {
            id,
            name: req.name,
            slug: req.slug,
            status: "active".into(),
            isolated_at: None,
            created_at,
        })
    }

    pub async fn get(&self, id: &str) -> Result<Tenant, DbError> {
        let row: Option<(String, String, String, String, Option<String>, String)> = sqlx::query_as(
            "SELECT id, name, slug, status, isolated_at, created_at FROM tenants WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let (id, name, slug, status, isolated_at, created_at) =
            row.ok_or_else(|| DbError::NotFound(format!("tenant {id}")))?;

        Ok(Tenant {
            id,
            name,
            slug,
            status,
            isolated_at,
            created_at,
        })
    }

    pub async fn list(&self) -> Result<Vec<Tenant>, DbError> {
        let rows: Vec<(String, String, String, String, Option<String>, String)> = sqlx::query_as(
            "SELECT id, name, slug, status, isolated_at, created_at FROM tenants ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, name, slug, status, isolated_at, created_at)| Tenant {
                id,
                name,
                slug,
                status,
                isolated_at,
                created_at,
            })
            .collect())
    }

    pub async fn isolate(&self, id: &str) -> Result<Tenant, DbError> {
        let isolated_at = now_iso();
        let result = sqlx::query(
            "UPDATE tenants SET status = 'isolated', isolated_at = ? WHERE id = ? AND status = 'active'",
        )
        .bind(&isolated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("tenant {id}")));
        }
        self.get(id).await
    }

    pub async fn delete(&self, id: &str) -> Result<(), DbError> {
        let result = sqlx::query("UPDATE tenants SET status = 'deleted' WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("tenant {id}")));
        }
        Ok(())
    }

    pub async fn is_active(&self, id: &str) -> Result<bool, DbError> {
        let status: Option<(String,)> =
            sqlx::query_as("SELECT status FROM tenants WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(status.map(|(s,)| s == "active").unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::setup;

    #[tokio::test]
    async fn tenant_crud() {
        let pool = setup("sqlite::memory:").await.expect("db");
        let mgr = TenantManager::new(pool);
        let t = mgr
            .create(CreateTenantRequest {
                name: "Acme".into(),
                slug: "acme".into(),
            })
            .await
            .expect("create");
        assert_eq!(t.slug, "acme");
        let listed = mgr.list().await.expect("list");
        assert_eq!(listed.len(), 1);
        let isolated = mgr.isolate(&t.id).await.expect("isolate");
        assert_eq!(isolated.status, "isolated");
    }
}
