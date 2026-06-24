use cloud_core::{audit_ztna_mutation, AuditWriteRequest};
use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedResourceRecord {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub resource_type: String,
    pub host: String,
    pub port: u16,
    pub path_prefix: Option<String>,
    pub tags: Vec<String>,
    pub published: bool,
    pub access_policy_id: Option<String>,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePublishedResourceRequest {
    pub name: String,
    pub resource_type: Option<String>,
    pub host: String,
    pub port: Option<u16>,
    pub path_prefix: Option<String>,
    pub tags: Option<Vec<String>>,
    pub published: Option<bool>,
    pub access_policy_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePublishedResourceRequest {
    pub name: Option<String>,
    pub resource_type: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path_prefix: Option<String>,
    pub tags: Option<Vec<String>>,
    pub published: Option<bool>,
    pub access_policy_id: Option<String>,
}

pub struct ResourcePublisher {
    pool: DbPool,
}

impl ResourcePublisher {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, tenant_id: &str) -> Result<Vec<PublishedResourceRecord>, DbError> {
        let rows: Vec<(
            String,
            String,
            String,
            String,
            String,
            i64,
            Option<String>,
            String,
            i64,
            Option<String>,
            Option<String>,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT id, tenant_id, name, resource_type, host, port, path_prefix, tags_json,
                    published, access_policy_id, published_at, created_at, updated_at
             FROM published_resources WHERE tenant_id = ? ORDER BY updated_at DESC",
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
                    name,
                    resource_type,
                    host,
                    port,
                    path_prefix,
                    tags_json,
                    published,
                    access_policy_id,
                    published_at,
                    created_at,
                    updated_at,
                )| {
                    PublishedResourceRecord {
                        id,
                        tenant_id,
                        name,
                        resource_type,
                        host,
                        port: port.max(0) as u16,
                        path_prefix,
                        tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                        published: published != 0,
                        access_policy_id,
                        published_at,
                        created_at,
                        updated_at,
                    }
                },
            )
            .collect())
    }

    pub async fn create(
        &self,
        tenant_id: &str,
        req: CreatePublishedResourceRequest,
        actor: Option<&str>,
    ) -> Result<PublishedResourceRecord, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = now_iso();
        let resource_type = req.resource_type.unwrap_or_else(|| "https".into());
        let port = req.port.unwrap_or(443);
        let tags_json =
            serde_json::to_string(&req.tags.unwrap_or_default()).unwrap_or_else(|_| "[]".into());
        let published = req.published.unwrap_or(false);
        let published_at = if published { Some(now.clone()) } else { None };

        sqlx::query(
            "INSERT INTO published_resources (
                id, tenant_id, name, resource_type, host, port, path_prefix, tags_json,
                published, access_policy_id, published_at, created_at, updated_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(&req.name)
        .bind(&resource_type)
        .bind(&req.host)
        .bind(i64::from(port))
        .bind(&req.path_prefix)
        .bind(&tags_json)
        .bind(published)
        .bind(&req.access_policy_id)
        .bind(&published_at)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        audit_ztna_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-ztna".into(),
                actor: actor.map(str::to_string),
                action: "ztna.resource.create".into(),
                resource_type: Some("published_resource".into()),
                resource_id: Some(id.clone()),
                details: serde_json::json!({ "name": req.name, "host": req.host }),
            },
        )
        .await?;

        Ok(PublishedResourceRecord {
            id,
            tenant_id: tenant_id.to_string(),
            name: req.name,
            resource_type,
            host: req.host,
            port,
            path_prefix: req.path_prefix,
            tags: serde_json::from_str(&tags_json).unwrap_or_default(),
            published,
            access_policy_id: req.access_policy_id,
            published_at,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update(
        &self,
        tenant_id: &str,
        resource_id: &str,
        req: UpdatePublishedResourceRequest,
        actor: Option<&str>,
    ) -> Result<PublishedResourceRecord, DbError> {
        let existing = self
            .list(tenant_id)
            .await?
            .into_iter()
            .find(|r| r.id == resource_id)
            .ok_or_else(|| DbError::NotFound(format!("resource {resource_id}")))?;

        let now = now_iso();
        let name = req.name.unwrap_or(existing.name);
        let resource_type = req.resource_type.unwrap_or(existing.resource_type);
        let host = req.host.unwrap_or(existing.host);
        let port = req.port.unwrap_or(existing.port);
        let path_prefix = req.path_prefix.or(existing.path_prefix);
        let tags = req.tags.unwrap_or(existing.tags);
        let published = req.published.unwrap_or(existing.published);
        let access_policy_id = req.access_policy_id.or(existing.access_policy_id);
        let published_at = if published {
            existing.published_at.or(Some(now.clone()))
        } else {
            None
        };
        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".into());

        sqlx::query(
            "UPDATE published_resources SET name = ?, resource_type = ?, host = ?, port = ?,
                    path_prefix = ?, tags_json = ?, published = ?, access_policy_id = ?,
                    published_at = ?, updated_at = ?
             WHERE id = ? AND tenant_id = ?",
        )
        .bind(&name)
        .bind(&resource_type)
        .bind(&host)
        .bind(i64::from(port))
        .bind(&path_prefix)
        .bind(&tags_json)
        .bind(published)
        .bind(&access_policy_id)
        .bind(&published_at)
        .bind(&now)
        .bind(resource_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        audit_ztna_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-ztna".into(),
                actor: actor.map(str::to_string),
                action: "ztna.resource.update".into(),
                resource_type: Some("published_resource".into()),
                resource_id: Some(resource_id.to_string()),
                details: serde_json::json!({ "name": name, "published": published }),
            },
        )
        .await?;

        Ok(PublishedResourceRecord {
            id: resource_id.to_string(),
            tenant_id: tenant_id.to_string(),
            name,
            resource_type,
            host,
            port,
            path_prefix,
            tags,
            published,
            access_policy_id,
            published_at,
            created_at: existing.created_at,
            updated_at: now,
        })
    }

    pub async fn delete(
        &self,
        tenant_id: &str,
        resource_id: &str,
        actor: Option<&str>,
    ) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM published_resources WHERE id = ? AND tenant_id = ?")
            .bind(resource_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("resource {resource_id}")));
        }

        audit_ztna_mutation(
            &self.pool,
            AuditWriteRequest {
                tenant_id: tenant_id.to_string(),
                source: "cloud-ztna".into(),
                actor: actor.map(str::to_string),
                action: "ztna.resource.delete".into(),
                resource_type: Some("published_resource".into()),
                resource_id: Some(resource_id.to_string()),
                details: serde_json::json!({}),
            },
        )
        .await?;

        Ok(())
    }
}
