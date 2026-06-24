use crate::provider::{IdentityClaims, IdentityProvider};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub issuer: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

impl Default for OidcConfig {
    fn default() -> Self {
        Self {
            issuer: std::env::var("WS_CLOUD_OIDC_ISSUER").ok(),
            client_id: std::env::var("WS_CLOUD_OIDC_CLIENT_ID").ok(),
            client_secret: std::env::var("WS_CLOUD_OIDC_CLIENT_SECRET").ok(),
        }
    }
}

pub struct OidcIdentityProvider {
    config: OidcConfig,
}

impl OidcIdentityProvider {
    pub fn new(config: OidcConfig) -> Self {
        Self { config }
    }

    pub fn is_configured(&self) -> bool {
        self.config.issuer.is_some() && self.config.client_id.is_some()
    }
}

#[async_trait]
impl IdentityProvider for OidcIdentityProvider {
    async fn exchange_code(&self, code: &str) -> Result<IdentityClaims, String> {
        if self.is_configured() {
            return Err("OIDC exchange not implemented for configured IdP".into());
        }
        // Mock exchange for tests when no IdP configured
        Ok(IdentityClaims {
            sub: format!("oidc-{code}"),
            email: Some(format!("{code}@mock.local")),
            username: Some(code.to_string()),
        })
    }

    async fn validate_token(&self, token: &str) -> Result<IdentityClaims, String> {
        if self.is_configured() {
            return Err("OIDC validation not implemented for configured IdP".into());
        }
        Ok(IdentityClaims {
            sub: format!("oidc-token-{token}"),
            email: None,
            username: Some(token.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_oidc_exchange() {
        let provider = OidcIdentityProvider::new(OidcConfig::default());
        let claims = provider.exchange_code("test-user").await.expect("exchange");
        assert!(claims.sub.contains("test-user"));
    }
}
