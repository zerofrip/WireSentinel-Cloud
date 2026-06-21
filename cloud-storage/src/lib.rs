use async_trait::async_trait;
use database::{models::now_iso, DbError, DbPool};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupObject {
    pub id: String,
    pub tenant_id: String,
    pub storage_provider: String,
    pub object_key: String,
    pub content_type: Option<String>,
    pub size_bytes: i64,
    pub checksum: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadBackupRequest {
    pub tenant_id: String,
    pub object_key: String,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage error: {0}")]
    Message(String),
    #[error("database error: {0}")]
    Db(#[from] DbError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[async_trait]
pub trait BackupStorageProvider: Send + Sync {
    async fn upload(&self, tenant_id: &str, object_key: &str, data: &[u8]) -> Result<(), String>;
    async fn download(&self, tenant_id: &str, object_key: &str) -> Result<Vec<u8>, String>;
    async fn delete(&self, tenant_id: &str, object_key: &str) -> Result<(), String>;
    fn provider_name(&self) -> &'static str;
}

pub struct LocalBackupProvider {
    root: PathBuf,
}

impl LocalBackupProvider {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    pub fn from_env() -> Self {
        let root = std::env::var("WS_BACKUP_DIR").unwrap_or_else(|_| "./data/backups".into());
        Self::new(root)
    }

    fn path_for(&self, tenant_id: &str, object_key: &str) -> PathBuf {
        self.root.join(tenant_id).join(object_key)
    }
}

#[async_trait]
impl BackupStorageProvider for LocalBackupProvider {
    async fn upload(&self, tenant_id: &str, object_key: &str, data: &[u8]) -> Result<(), String> {
        let path = self.path_for(tenant_id, object_key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }
        tokio::fs::write(&path, data)
            .await
            .map_err(|e| e.to_string())
    }

    async fn download(&self, tenant_id: &str, object_key: &str) -> Result<Vec<u8>, String> {
        let path = self.path_for(tenant_id, object_key);
        tokio::fs::read(&path).await.map_err(|e| e.to_string())
    }

    async fn delete(&self, tenant_id: &str, object_key: &str) -> Result<(), String> {
        let path = self.path_for(tenant_id, object_key);
        if path.exists() {
            tokio::fs::remove_file(&path)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "local"
    }
}

pub struct S3BackupProvider {
    bucket: String,
    endpoint: Option<String>,
}

impl S3BackupProvider {
    pub fn from_env() -> Option<Self> {
        let bucket = std::env::var("WS_S3_BUCKET").ok()?;
        let endpoint = std::env::var("WS_S3_ENDPOINT").ok();
        Some(Self { bucket, endpoint })
    }
}

#[async_trait]
impl BackupStorageProvider for S3BackupProvider {
    async fn upload(&self, tenant_id: &str, object_key: &str, _data: &[u8]) -> Result<(), String> {
        #[cfg(feature = "s3")]
        {
            let _ = (tenant_id, object_key);
            return Err(format!(
                "S3 upload stub for bucket {} (endpoint {:?})",
                self.bucket, self.endpoint
            ));
        }
        #[cfg(not(feature = "s3"))]
        {
            let _ = (tenant_id, object_key);
            Err("s3 feature not enabled".into())
        }
    }

    async fn download(&self, _tenant_id: &str, _object_key: &str) -> Result<Vec<u8>, String> {
        Err("S3 download not implemented (stub)".into())
    }

    async fn delete(&self, _tenant_id: &str, _object_key: &str) -> Result<(), String> {
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "s3"
    }
}

pub struct AzureBackupProvider;

#[async_trait]
impl BackupStorageProvider for AzureBackupProvider {
    async fn upload(&self, _tenant_id: &str, _object_key: &str, _data: &[u8]) -> Result<(), String> {
        Err("Azure backup provider is a stub".into())
    }

    async fn download(&self, _tenant_id: &str, _object_key: &str) -> Result<Vec<u8>, String> {
        Err("Azure backup provider is a stub".into())
    }

    async fn delete(&self, _tenant_id: &str, _object_key: &str) -> Result<(), String> {
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "azure"
    }
}

pub fn backup_provider_from_env() -> Box<dyn BackupStorageProvider> {
    if let Some(s3) = S3BackupProvider::from_env() {
        return Box::new(s3);
    }
    Box::new(LocalBackupProvider::from_env())
}

pub struct BackupStorageService {
    pool: DbPool,
    provider: Box<dyn BackupStorageProvider>,
}

impl BackupStorageService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            provider: backup_provider_from_env(),
        }
    }

    pub async fn upload(&self, req: UploadBackupRequest) -> Result<BackupObject, StorageError> {
        self.provider
            .upload(&req.tenant_id, &req.object_key, &req.data)
            .await
            .map_err(StorageError::Message)?;

        let id = Uuid::new_v4().to_string();
        let created_at = now_iso();
        let metadata = req.metadata.unwrap_or(serde_json::json!({}));
        let metadata_str = metadata.to_string();
        let size = req.data.len() as i64;

        sqlx::query(
            "INSERT INTO backup_objects (id, tenant_id, storage_provider, object_key, content_type, size_bytes, metadata, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&req.tenant_id)
        .bind(self.provider.provider_name())
        .bind(&req.object_key)
        .bind(&req.content_type)
        .bind(size)
        .bind(&metadata_str)
        .bind(&created_at)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(BackupObject {
            id,
            tenant_id: req.tenant_id,
            storage_provider: self.provider.provider_name().into(),
            object_key: req.object_key,
            content_type: req.content_type,
            size_bytes: size,
            checksum: None,
            metadata,
            created_at,
        })
    }

    pub async fn restore(&self, tenant_id: &str, object_key: &str) -> Result<Vec<u8>, StorageError> {
        self.provider
            .download(tenant_id, object_key)
            .await
            .map_err(StorageError::Message)
    }

    pub async fn list(&self, tenant_id: &str) -> Result<Vec<BackupObject>, DbError> {
        let rows: Vec<(String, String, String, String, Option<String>, i64, Option<String>, String, String)> =
            sqlx::query_as(
                "SELECT id, tenant_id, storage_provider, object_key, content_type, size_bytes, checksum, metadata, created_at FROM backup_objects WHERE tenant_id = ? ORDER BY created_at DESC",
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, tenant_id, storage_provider, object_key, content_type, size_bytes, checksum, metadata, created_at)| {
                    BackupObject {
                        id,
                        tenant_id,
                        storage_provider,
                        object_key,
                        content_type,
                        size_bytes,
                        checksum,
                        metadata: serde_json::from_str(&metadata).unwrap_or(serde_json::json!({})),
                        created_at,
                    }
                },
            )
            .collect())
    }

    pub async fn ingest_log(
        &self,
        tenant_id: &str,
        source: &str,
        level: &str,
        message: &str,
        fields: Option<serde_json::Value>,
    ) -> Result<(), DbError> {
        let id = Uuid::new_v4().to_string();
        let ingested_at = now_iso();
        let fields_json = fields.unwrap_or(serde_json::json!({})).to_string();
        sqlx::query(
            "INSERT INTO aggregated_logs (id, tenant_id, source, level, message, fields_json, ingested_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(tenant_id)
        .bind(source)
        .bind(level)
        .bind(message)
        .bind(&fields_json)
        .bind(&ingested_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
