use auth::{Claims, TeamRole};
use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use cloud_core::TenantContext;

use crate::{error::ApiError, routes::AppState};

#[derive(Clone)]
pub struct AuthUser {
    pub claims: Claims,
    pub tenant_id: String,
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let token = auth_header
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(ApiError::Unauthorized)?;

    let claims = state.auth.validate_token(token)?;

    let tenant_header = req
        .headers()
        .get("X-Tenant-Id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);

    let tenant_id = tenant_header.unwrap_or_else(|| claims.tenant_id.clone());

    if tenant_id != claims.tenant_id {
        return Err(ApiError::Forbidden);
    }

    if !state.tenants.is_active(&tenant_id).await? {
        return Err(ApiError::Forbidden);
    }

    req.extensions_mut().insert(AuthUser {
        claims: claims.clone(),
        tenant_id: tenant_id.clone(),
    });
    req.extensions_mut().insert(TenantContext {
        tenant_id: tenant_id.clone(),
        user_id: claims.sub.clone(),
        username: claims.username.clone(),
        role: claims.role.as_str().into(),
    });

    Ok(next.run(req).await)
}

pub fn require_role(claims: &Claims, min: TeamRole) -> Result<(), ApiError> {
    if claims.role.level() >= min.level() {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

mod extractor {
    use super::AuthUser;
    use axum::extract::FromRequestParts;
    use axum::http::{request::Parts, StatusCode};

    impl<S> FromRequestParts<S> for AuthUser
    where
        S: Send + Sync,
    {
        type Rejection = StatusCode;

        async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
            parts
                .extensions
                .get::<AuthUser>()
                .cloned()
                .ok_or(StatusCode::UNAUTHORIZED)
        }
    }
}
