use chrono::{Duration, Utc};
use cloud_core::CloudSecurityPolicy;
use database::{models::now_iso, DbError, DbPool};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TeamRole {
    Owner,
    Administrator,
    Operator,
    Viewer,
}

impl TeamRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Administrator => "administrator",
            Self::Operator => "operator",
            Self::Viewer => "viewer",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "owner" => Some(Self::Owner),
            "administrator" => Some(Self::Administrator),
            "operator" => Some(Self::Operator),
            "viewer" => Some(Self::Viewer),
            _ => None,
        }
    }

    pub fn level(self) -> u8 {
        match self {
            Self::Owner => 4,
            Self::Administrator => 3,
            Self::Operator => 2,
            Self::Viewer => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub tenant_id: String,
    pub role: TeamRole,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: String,
    pub role: TeamRole,
    pub username: String,
    pub tenant_id: String,
}

pub struct JwtAuthService {
    pool: DbPool,
    policy: CloudSecurityPolicy,
}

impl JwtAuthService {
    pub fn new(pool: DbPool, policy: CloudSecurityPolicy) -> Self {
        Self { pool, policy }
    }

    pub async fn ensure_default_admin(&self, default_tenant_id: &str) -> Result<(), DbError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        if count.0 == 0 {
            let id = Uuid::new_v4().to_string();
            let hash = bcrypt::hash("admin", self.policy.bcrypt_cost)
                .map_err(|e| DbError::NotFound(e.to_string()))?;
            sqlx::query(
                "INSERT INTO users (id, tenant_id, username, password_hash, role, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(default_tenant_id)
            .bind("admin")
            .bind(&hash)
            .bind(TeamRole::Owner.as_str())
            .bind(now_iso())
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn login(&self, req: LoginRequest) -> Result<LoginResponse, AuthError> {
        let tenant_id = if let Some(t) = req.tenant_id {
            t
        } else {
            let row: Option<(String,)> =
                sqlx::query_as("SELECT id FROM tenants WHERE status = 'active' ORDER BY created_at LIMIT 1")
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|e| AuthError::Internal(e.to_string()))?;
            row.ok_or(AuthError::InvalidCredentials)?
                .0
        };

        let row: Option<(String, Option<String>, String)> = sqlx::query_as(
            "SELECT id, password_hash, role FROM users WHERE username = ? AND tenant_id = ?",
        )
        .bind(&req.username)
        .bind(&tenant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?;

        let (user_id, password_hash, role_str) =
            row.ok_or(AuthError::InvalidCredentials)?;

        let hash = password_hash.ok_or(AuthError::InvalidCredentials)?;
        let valid = bcrypt::verify(&req.password, &hash)
            .map_err(|e| AuthError::Internal(e.to_string()))?;
        if !valid {
            return Err(AuthError::InvalidCredentials);
        }

        let role = TeamRole::from_str(&role_str).ok_or(AuthError::InvalidRole)?;
        let issued = self.issue_token(&user_id, &req.username, &tenant_id, role)?;

        Ok(LoginResponse {
            token: issued.token,
            expires_at: issued.expires_at,
            role,
            username: req.username,
            tenant_id,
        })
    }

    pub fn issue_token(
        &self,
        user_id: &str,
        username: &str,
        tenant_id: &str,
        role: TeamRole,
    ) -> Result<IssuedToken, AuthError> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.policy.token_ttl_hours);
        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            tenant_id: tenant_id.to_string(),
            role,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.policy.jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(IssuedToken {
            token,
            expires_at: exp.to_rfc3339(),
        })
    }

    pub fn issue_from_oidc(
        &self,
        user_id: &str,
        username: &str,
        tenant_id: &str,
        role: TeamRole,
    ) -> Result<LoginResponse, AuthError> {
        let issued = self.issue_token(user_id, username, tenant_id, role)?;
        Ok(LoginResponse {
            token: issued.token,
            expires_at: issued.expires_at,
            role,
            username: username.to_string(),
            tenant_id: tenant_id.to_string(),
        })
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.policy.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;
        Ok(data.claims)
    }

    pub fn authorize(&self, claims: &Claims, required: TeamRole) -> Result<(), AuthError> {
        if claims.role.level() >= required.level() {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
}

#[derive(Debug, Clone)]
struct IssuedToken {
    token: String,
    expires_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("invalid token")]
    InvalidToken,
    #[error("forbidden")]
    Forbidden,
    #[error("invalid role")]
    InvalidRole,
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<DbError> for AuthError {
    fn from(value: DbError) -> Self {
        Self::Internal(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_hierarchy() {
        assert!(TeamRole::Owner.level() > TeamRole::Viewer.level());
        assert_eq!(TeamRole::Administrator.as_str(), "administrator");
    }
}
