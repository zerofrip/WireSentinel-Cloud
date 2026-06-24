use sqlx::any::AnyPoolOptions;
use sqlx::AnyPool;
use std::sync::Once;
use std::time::Duration;

static INSTALL_DRIVERS: Once = Once::new();

fn ensure_drivers() {
    INSTALL_DRIVERS.call_once(|| {
        sqlx::any::install_default_drivers();
    });
}

pub mod models;

pub type DbPool = AnyPool;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("not found: {0}")]
    NotFound(String),
}

fn default_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://./data/cloud.db?mode=rwc".into())
}

pub fn resolve_database_url(url: Option<&str>) -> String {
    normalize_database_url(&url.map(str::to_string).unwrap_or_else(default_database_url))
}

fn normalize_database_url(url: &str) -> String {
    if url == "sqlite::memory:" {
        format!(
            "sqlite:file:mem_{}?mode=memory&cache=shared",
            uuid::Uuid::new_v4()
        )
    } else {
        url.to_string()
    }
}

fn ensure_sqlite_parent_dir(database_url: &str) {
    if !database_url.starts_with("sqlite:") {
        return;
    }
    let path = database_url
        .strip_prefix("sqlite://")
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("");
    if path.is_empty() || path == ":memory:" {
        return;
    }
    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
}

pub async fn connect(database_url: &str) -> Result<DbPool, DbError> {
    ensure_drivers();
    let database_url = normalize_database_url(database_url);
    ensure_sqlite_parent_dir(&database_url);

    let pool = AnyPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await?;

    Ok(pool)
}

pub async fn migrate(pool: &DbPool) -> Result<(), DbError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| DbError::Sqlx(sqlx::Error::Migrate(Box::new(e))))?;
    Ok(())
}

pub async fn setup(database_url: &str) -> Result<DbPool, DbError> {
    let pool = connect(database_url).await?;
    migrate(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sqlite_memory_migrates() {
        let pool = setup("sqlite::memory:").await.expect("setup");
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tenants")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(count.0, 0);

        let mig_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .expect("migrations");
        assert_eq!(mig_count.0, 22, "expected all 22 migrations");

        sqlx::query("SELECT COUNT(*) FROM cloud_anonymity_rollups")
            .fetch_one(&pool)
            .await
            .expect("cloud_anonymity_rollups table");
    }
}
