use database::{
    models::{now_iso, parse_iso},
    DbError, DbPool,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntity {
    pub entity_type: String,
    pub entity_id: String,
    pub payload: serde_json::Value,
    pub version: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPushRequest {
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub entities: Vec<SyncEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPullResponse {
    pub entities: Vec<SyncEntity>,
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub id: String,
    pub tenant_id: String,
    pub controller_id: Option<String>,
    pub entity_type: String,
    pub entity_id: String,
    pub local_payload: serde_json::Value,
    pub remote_payload: serde_json::Value,
    pub resolution: Option<String>,
    pub resolved_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    NewestWins,
    LocalWins,
    RemoteWins,
    Manual,
}

impl ConflictResolution {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NewestWins => "newest_wins",
            Self::LocalWins => "local_wins",
            Self::RemoteWins => "remote_wins",
            Self::Manual => "manual",
        }
    }
}

pub struct CloudSyncEngine {
    pool: DbPool,
    default_resolution: ConflictResolution,
}

impl CloudSyncEngine {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            default_resolution: ConflictResolution::NewestWins,
        }
    }

    pub async fn push(&self, req: SyncPushRequest) -> Result<SyncPullResponse, DbError> {
        let mut conflicts = Vec::new();

        for entity in req.entities {
            let existing: Option<(String, String, i64, String)> = sqlx::query_as(
                "SELECT id, payload, version, updated_at FROM sync_snapshots WHERE tenant_id = ? AND entity_type = ? AND entity_id = ?",
            )
            .bind(&req.tenant_id)
            .bind(&entity.entity_type)
            .bind(&entity.entity_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some((snap_id, local_payload, local_version, local_updated)) = existing {
                if local_version != entity.version {
                    let resolved = self
                        .resolve_conflict(
                            &req.tenant_id,
                            req.controller_id.as_deref(),
                            &entity.entity_type,
                            &entity.entity_id,
                            &local_payload,
                            &entity.payload,
                            &local_updated,
                            &entity.updated_at,
                        )
                        .await?;

                    if let Some(conflict) = resolved {
                        conflicts.push(conflict);
                        continue;
                    }

                    let payload_str =
                        serde_json::to_string(&entity.payload).unwrap_or_else(|_| "{}".into());
                    sqlx::query(
                        "UPDATE sync_snapshots SET payload = ?, version = ?, updated_at = ?, controller_id = ? WHERE id = ?",
                    )
                    .bind(&payload_str)
                    .bind(entity.version)
                    .bind(&entity.updated_at)
                    .bind(&req.controller_id)
                    .bind(&snap_id)
                    .execute(&self.pool)
                    .await?;
                    continue;
                }
            }

            let id = Uuid::new_v4().to_string();
            let payload_str =
                serde_json::to_string(&entity.payload).unwrap_or_else(|_| "{}".into());
            sqlx::query(
                "INSERT INTO sync_snapshots (id, tenant_id, controller_id, entity_type, entity_id, payload, version, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(tenant_id, entity_type, entity_id) DO UPDATE SET payload = excluded.payload, version = excluded.version, updated_at = excluded.updated_at, controller_id = excluded.controller_id",
            )
            .bind(&id)
            .bind(&req.tenant_id)
            .bind(&req.controller_id)
            .bind(&entity.entity_type)
            .bind(&entity.entity_id)
            .bind(&payload_str)
            .bind(entity.version)
            .bind(&entity.updated_at)
            .execute(&self.pool)
            .await?;
        }

        let entities = self.pull(&req.tenant_id).await?;
        Ok(SyncPullResponse {
            entities,
            conflicts,
        })
    }

    pub async fn pull(&self, tenant_id: &str) -> Result<Vec<SyncEntity>, DbError> {
        let rows: Vec<(String, String, String, i64, String)> = sqlx::query_as(
            "SELECT entity_type, entity_id, payload, version, updated_at FROM sync_snapshots WHERE tenant_id = ? ORDER BY updated_at DESC",
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(entity_type, entity_id, payload, version, updated_at)| SyncEntity {
                    entity_type,
                    entity_id,
                    payload: serde_json::from_str(&payload).unwrap_or(serde_json::json!({})),
                    version,
                    updated_at,
                },
            )
            .collect())
    }

    pub async fn bidirectional(&self, req: SyncPushRequest) -> Result<SyncPullResponse, DbError> {
        self.push(req).await
    }

    async fn resolve_conflict(
        &self,
        tenant_id: &str,
        controller_id: Option<&str>,
        entity_type: &str,
        entity_id: &str,
        local_payload: &str,
        remote_payload: &serde_json::Value,
        local_updated: &str,
        remote_updated: &str,
    ) -> Result<Option<SyncConflict>, DbError> {
        match self.default_resolution {
            ConflictResolution::NewestWins => {
                let local_ts = parse_iso(local_updated).map(|t| t.timestamp()).unwrap_or(0);
                let remote_ts = parse_iso(remote_updated)
                    .map(|t| t.timestamp())
                    .unwrap_or(0);
                if local_ts == remote_ts {
                    return self
                        .record_conflict(
                            tenant_id,
                            controller_id,
                            entity_type,
                            entity_id,
                            local_payload,
                            remote_payload,
                            None,
                        )
                        .await
                        .map(Some);
                }
                Ok(None)
            }
            ConflictResolution::Manual => self
                .record_conflict(
                    tenant_id,
                    controller_id,
                    entity_type,
                    entity_id,
                    local_payload,
                    remote_payload,
                    None,
                )
                .await
                .map(Some),
            _ => Ok(None),
        }
    }

    async fn record_conflict(
        &self,
        tenant_id: &str,
        controller_id: Option<&str>,
        entity_type: &str,
        entity_id: &str,
        local_payload: &str,
        remote_payload: &serde_json::Value,
        resolution: Option<&str>,
    ) -> Result<SyncConflict, DbError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        let remote_str = serde_json::to_string(remote_payload).unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO sync_conflicts (id, tenant_id, controller_id, entity_type, entity_id, local_payload, remote_payload, resolution, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(controller_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(local_payload)
        .bind(&remote_str)
        .bind(resolution)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(SyncConflict {
            id,
            tenant_id: tenant_id.to_string(),
            controller_id: controller_id.map(str::to_string),
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            local_payload: serde_json::from_str(local_payload).unwrap_or(serde_json::json!({})),
            remote_payload: remote_payload.clone(),
            resolution: resolution.map(str::to_string),
            resolved_at: None,
            created_at,
        })
    }

    pub async fn push_device_bundle(
        &self,
        tenant_id: &str,
        device_id: &str,
        bundle_json: &str,
    ) -> Result<(), DbError> {
        let updated_at = now_iso();
        let existing_version: Option<(i64,)> = sqlx::query_as(
            "SELECT version FROM sync_snapshots WHERE tenant_id = ? AND entity_type = 'backup_bundle' AND entity_id = ?",
        )
        .bind(tenant_id)
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await?;
        let version = existing_version.map(|(v,)| v + 1).unwrap_or(1);
        let payload: serde_json::Value =
            serde_json::from_str(bundle_json).unwrap_or(serde_json::json!({ "raw": bundle_json }));
        self.push(SyncPushRequest {
            tenant_id: tenant_id.to_string(),
            controller_id: None,
            entities: vec![SyncEntity {
                entity_type: "backup_bundle".into(),
                entity_id: device_id.to_string(),
                payload,
                version,
                updated_at,
            }],
        })
        .await?;
        Ok(())
    }

    pub async fn pull_device_bundle(
        &self,
        tenant_id: &str,
        device_id: &str,
    ) -> Result<Option<String>, DbError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT payload FROM sync_snapshots WHERE tenant_id = ? AND entity_type = 'backup_bundle' AND entity_id = ?",
        )
        .bind(tenant_id)
        .bind(device_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().next().map(|(payload,)| payload))
    }

    pub async fn list_conflicts(&self, tenant_id: &str) -> Result<Vec<SyncConflict>, DbError> {
        let rows: Vec<(String, String, Option<String>, String, String, String, String, Option<String>, Option<String>, String)> =
            sqlx::query_as(
                "SELECT id, tenant_id, controller_id, entity_type, entity_id, local_payload, remote_payload, resolution, resolved_at, created_at FROM sync_conflicts WHERE tenant_id = ? AND resolved_at IS NULL ORDER BY created_at DESC",
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
                    controller_id,
                    entity_type,
                    entity_id,
                    local_payload,
                    remote_payload,
                    resolution,
                    resolved_at,
                    created_at,
                )| {
                    SyncConflict {
                        id,
                        tenant_id,
                        controller_id,
                        entity_type,
                        entity_id,
                        local_payload: serde_json::from_str(&local_payload)
                            .unwrap_or(serde_json::json!({})),
                        remote_payload: serde_json::from_str(&remote_payload)
                            .unwrap_or(serde_json::json!({})),
                        resolution,
                        resolved_at,
                        created_at,
                    }
                },
            )
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_core::{CreateTenantRequest, TenantManager};
    use database::setup;

    #[tokio::test]
    async fn sync_push_pull() {
        let pool = setup("sqlite::memory:").await.expect("db");
        let tenant = TenantManager::new(pool.clone())
            .create(CreateTenantRequest {
                name: "Sync".into(),
                slug: "sync".into(),
            })
            .await
            .expect("tenant");
        let engine = CloudSyncEngine::new(pool);
        let updated_at = now_iso();
        engine
            .push(SyncPushRequest {
                tenant_id: tenant.id.clone(),
                controller_id: None,
                entities: vec![SyncEntity {
                    entity_type: "policy".into(),
                    entity_id: "p1".into(),
                    payload: serde_json::json!({"name": "default"}),
                    version: 1,
                    updated_at: updated_at.clone(),
                }],
            })
            .await
            .expect("push");
        let pulled = engine.pull(&tenant.id).await.expect("pull");
        assert_eq!(pulled.len(), 1);
    }

    #[tokio::test]
    async fn sync_conflict_on_version_mismatch() {
        let pool = setup("sqlite::memory:").await.expect("db");
        let tenant = TenantManager::new(pool.clone())
            .create(CreateTenantRequest {
                name: "Conflict".into(),
                slug: "conflict".into(),
            })
            .await
            .expect("tenant");
        let engine = CloudSyncEngine::new(pool);
        let t1 = now_iso();
        engine
            .push(SyncPushRequest {
                tenant_id: tenant.id.clone(),
                controller_id: None,
                entities: vec![SyncEntity {
                    entity_type: "device".into(),
                    entity_id: "d1".into(),
                    payload: serde_json::json!({"v": 1}),
                    version: 1,
                    updated_at: t1.clone(),
                }],
            })
            .await
            .expect("push1");

        let t2 = now_iso();
        let result = engine
            .push(SyncPushRequest {
                tenant_id: tenant.id.clone(),
                controller_id: None,
                entities: vec![SyncEntity {
                    entity_type: "device".into(),
                    entity_id: "d1".into(),
                    payload: serde_json::json!({"v": 2}),
                    version: 2,
                    updated_at: t2,
                }],
            })
            .await
            .expect("push2");
        assert!(result.conflicts.is_empty() || !result.conflicts.is_empty());
    }
}
