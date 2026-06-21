use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityClaims {
    pub sub: String,
    pub email: Option<String>,
    pub username: Option<String>,
}

#[async_trait]
pub trait IdentityProvider: Send + Sync {
    async fn exchange_code(&self, code: &str) -> Result<IdentityClaims, String>;
    async fn validate_token(&self, token: &str) -> Result<IdentityClaims, String>;
}
