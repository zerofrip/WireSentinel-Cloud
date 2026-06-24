use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedLogEntry {
    pub id: String,
    pub tenant_id: String,
    pub source: String,
    pub level: String,
    pub message: String,
    pub fields: serde_json::Value,
    pub ingested_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestLogRequest {
    pub source: String,
    pub level: Option<String>,
    pub message: String,
    pub fields: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSearchQuery {
    pub q: Option<String>,
    pub level: Option<String>,
    pub source: Option<String>,
    pub limit: Option<i64>,
}

pub struct LogAggregationService {
    pool: DbPool,
}

impl LogAggregationService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn ingest(
        &self,
        tenant_id: &str,
        req: IngestLogRequest,
    ) -> Result<AggregatedLogEntry, DbError> {
        let id = Uuid::new_v4().to_string();
        let ingested_at = now_iso();
        let level = req.level.unwrap_or_else(|| "info".into());
        let fields_json = serde_json::to_string(&req.fields.unwrap_or(serde_json::json!({})))
            .unwrap_or_else(|_| "{}".into());

        sqlx::query(
            "INSERT INTO aggregated_logs (id, tenant_id, source, level, message, fields_json, ingested_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(&req.source)
        .bind(&level)
        .bind(&req.message)
        .bind(&fields_json)
        .bind(&ingested_at)
        .execute(&self.pool)
        .await?;

        Ok(AggregatedLogEntry {
            id,
            tenant_id: tenant_id.to_string(),
            source: req.source,
            level,
            message: req.message,
            fields: serde_json::from_str(&fields_json).unwrap_or(serde_json::json!({})),
            ingested_at,
        })
    }

    pub async fn list(
        &self,
        tenant_id: &str,
        limit: i64,
    ) -> Result<Vec<AggregatedLogEntry>, DbError> {
        let rows: Vec<(String, String, String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, tenant_id, source, level, message, fields_json, ingested_at FROM aggregated_logs WHERE tenant_id = ? ORDER BY ingested_at DESC LIMIT ?",
        )
        .bind(tenant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, tenant_id, source, level, message, fields_json, ingested_at)| {
                    AggregatedLogEntry {
                        id,
                        tenant_id,
                        source,
                        level,
                        message,
                        fields: serde_json::from_str(&fields_json).unwrap_or(serde_json::json!({})),
                        ingested_at,
                    }
                },
            )
            .collect())
    }

    pub async fn search(
        &self,
        tenant_id: &str,
        query: LogSearchQuery,
    ) -> Result<Vec<AggregatedLogEntry>, DbError> {
        let limit = query.limit.unwrap_or(100).clamp(1, 500);
        let pattern = query
            .q
            .as_deref()
            .map(|q| format!("%{q}%"))
            .unwrap_or_else(|| "%".into());

        let rows: Vec<(String, String, String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, tenant_id, source, level, message, fields_json, ingested_at \
             FROM aggregated_logs \
             WHERE tenant_id = ? \
               AND message LIKE ? \
               AND (? IS NULL OR level = ?) \
               AND (? IS NULL OR source = ?) \
             ORDER BY ingested_at DESC LIMIT ?",
        )
        .bind(tenant_id)
        .bind(&pattern)
        .bind(&query.level)
        .bind(&query.level)
        .bind(&query.source)
        .bind(&query.source)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, tenant_id, source, level, message, fields_json, ingested_at)| {
                    AggregatedLogEntry {
                        id,
                        tenant_id,
                        source,
                        level,
                        message,
                        fields: serde_json::from_str(&fields_json).unwrap_or(serde_json::json!({})),
                        ingested_at,
                    }
                },
            )
            .collect())
    }
}
