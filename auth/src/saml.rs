use crate::provider::{IdentityClaims, IdentityProvider};
use async_trait::async_trait;

/// SAML identity provider stub — real SAML assertion parsing is out of scope for Phase 11.
pub struct SamlIdentityProvider;

#[async_trait]
impl IdentityProvider for SamlIdentityProvider {
    async fn exchange_code(&self, _code: &str) -> Result<IdentityClaims, String> {
        Err("SAML exchange not implemented".into())
    }

    async fn validate_token(&self, _token: &str) -> Result<IdentityClaims, String> {
        Err("SAML validation not implemented".into())
    }
}
