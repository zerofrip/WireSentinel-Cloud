use chrono::{DateTime, Duration, Utc};
use cloud_events::{CloudEvent, FailoverTriggered, NodeFailed};
use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const DEFAULT_LEASE_KEY: &str = "cloud-api-leader";
const DEFAULT_LEASE_TTL_SECS: i64 = 15;
const HEARTBEAT_TIMEOUT_SECS: i64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    pub id: String,
    pub node_name: String,
    pub address: String,
    pub role: String,
    pub status: String,
    pub last_heartbeat_at: Option<String>,
    pub lease_expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterNodeRequest {
    pub node_name: String,
    pub address: String,
}

pub struct HaManager {
    pool: DbPool,
    lease_key: String,
    lease_ttl_secs: i64,
}

impl HaManager {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            lease_key: DEFAULT_LEASE_KEY.into(),
            lease_ttl_secs: DEFAULT_LEASE_TTL_SECS,
        }
    }

    pub async fn register_node(&self, req: RegisterNodeRequest) -> Result<ClusterNode, DbError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        sqlx::query(
            "INSERT INTO cluster_nodes (id, node_name, address, role, status, created_at) VALUES (?, ?, ?, 'follower', 'active', ?)",
        )
        .bind(&id)
        .bind(&req.node_name)
        .bind(&req.address)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(ClusterNode {
            id,
            node_name: req.node_name,
            address: req.address,
            role: "follower".into(),
            status: "active".into(),
            last_heartbeat_at: None,
            lease_expires_at: None,
            created_at,
        })
    }

    pub async fn heartbeat(&self, node_id: &str) -> Result<ClusterNode, DbError> {
        let ts = now_iso();
        sqlx::query(
            "UPDATE cluster_nodes SET last_heartbeat_at = ?, status = 'active' WHERE id = ?",
        )
        .bind(&ts)
        .bind(node_id)
        .execute(&self.pool)
        .await?;
        self.get_node(node_id).await
    }

    pub async fn list_nodes(&self) -> Result<Vec<ClusterNode>, DbError> {
        let rows: Vec<(String, String, String, String, String, Option<String>, Option<String>, String)> =
            sqlx::query_as(
                "SELECT id, node_name, address, role, status, last_heartbeat_at, lease_expires_at, created_at FROM cluster_nodes ORDER BY node_name",
            )
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, node_name, address, role, status, last_heartbeat_at, lease_expires_at, created_at)| {
                    ClusterNode {
                        id,
                        node_name,
                        address,
                        role,
                        status,
                        last_heartbeat_at,
                        lease_expires_at,
                        created_at,
                    }
                },
            )
            .collect())
    }

    pub async fn try_acquire_leader(&self, node_id: &str) -> Result<Option<CloudEvent>, DbError> {
        let now = Utc::now();
        let expires = (now + Duration::seconds(self.lease_ttl_secs)).to_rfc3339();
        let updated_at = now.to_rfc3339();

        let existing: Option<(String, String)> = sqlx::query_as(
            "SELECT holder_node_id, expires_at FROM cluster_leases WHERE lease_key = ?",
        )
        .bind(&self.lease_key)
        .fetch_optional(&self.pool)
        .await?;

        let previous_leader = existing
            .as_ref()
            .filter(|(_, exp)| {
                DateTime::parse_from_rfc3339(exp)
                    .map(|d| d.with_timezone(&Utc) > now)
                    .unwrap_or(false)
            })
            .map(|(id, _)| id.clone());

        if let Some((holder, exp)) = existing {
            let still_valid = DateTime::parse_from_rfc3339(&exp)
                .map(|d| d.with_timezone(&Utc) > now)
                .unwrap_or(false);
            if still_valid && holder != node_id {
                return Ok(None);
            }
        }

        sqlx::query(
            "INSERT INTO cluster_leases (lease_key, holder_node_id, expires_at, updated_at) VALUES (?, ?, ?, ?) \
             ON CONFLICT(lease_key) DO UPDATE SET holder_node_id = excluded.holder_node_id, expires_at = excluded.expires_at, updated_at = excluded.updated_at",
        )
        .bind(&self.lease_key)
        .bind(node_id)
        .bind(&expires)
        .bind(&updated_at)
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE cluster_nodes SET role = 'follower' WHERE role = 'leader' AND id != ?")
            .bind(node_id)
            .execute(&self.pool)
            .await?;
        sqlx::query(
            "UPDATE cluster_nodes SET role = 'leader', lease_expires_at = ? WHERE id = ?",
        )
        .bind(&expires)
        .bind(node_id)
        .execute(&self.pool)
        .await?;

        if previous_leader.as_deref() != Some(node_id) {
            return Ok(Some(CloudEvent::FailoverTriggered(FailoverTriggered {
                previous_leader_id: previous_leader,
                new_leader_id: node_id.to_string(),
                lease_key: self.lease_key.clone(),
                triggered_at: now,
            })));
        }
        Ok(None)
    }

    pub async fn detect_failed_nodes(&self) -> Result<Vec<CloudEvent>, DbError> {
        let cutoff = (Utc::now() - Duration::seconds(HEARTBEAT_TIMEOUT_SECS)).to_rfc3339();
        let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT id, node_name, last_heartbeat_at FROM cluster_nodes WHERE status = 'active' AND (last_heartbeat_at IS NULL OR last_heartbeat_at < ?)",
        )
        .bind(&cutoff)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        let now = Utc::now();
        for (id, node_name, last_heartbeat_at) in rows {
            sqlx::query("UPDATE cluster_nodes SET status = 'failed' WHERE id = ?")
                .bind(&id)
                .execute(&self.pool)
                .await?;
            events.push(CloudEvent::NodeFailed(NodeFailed {
                node_id: id,
                node_name,
                last_heartbeat_at: last_heartbeat_at
                    .and_then(|ts| DateTime::parse_from_rfc3339(&ts).ok())
                    .map(|d| d.with_timezone(&Utc)),
                detected_at: now,
            }));
        }
        Ok(events)
    }

    pub async fn current_leader(&self) -> Result<Option<ClusterNode>, DbError> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT holder_node_id FROM cluster_leases WHERE lease_key = ? AND expires_at > ?",
        )
        .bind(&self.lease_key)
        .bind(now_iso())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((id,)) => self.get_node(&id).await.map(Some),
            None => Ok(None),
        }
    }

    async fn get_node(&self, node_id: &str) -> Result<ClusterNode, DbError> {
        let row: Option<(String, String, String, String, String, Option<String>, Option<String>, String)> =
            sqlx::query_as(
                "SELECT id, node_name, address, role, status, last_heartbeat_at, lease_expires_at, created_at FROM cluster_nodes WHERE id = ?",
            )
            .bind(node_id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(
            |(id, node_name, address, role, status, last_heartbeat_at, lease_expires_at, created_at)| {
                ClusterNode {
                    id,
                    node_name,
                    address,
                    role,
                    status,
                    last_heartbeat_at,
                    lease_expires_at,
                    created_at,
                }
            },
        )
        .ok_or_else(|| DbError::NotFound(format!("node {node_id}")))
    }
}
