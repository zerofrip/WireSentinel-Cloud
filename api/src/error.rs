use auth::AuthError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use billing::QuotaError as BillingQuotaError;
use cloud_billing::BillingError;
use cloud_metering::MeteringError;
use cloud_quotas::QuotaError as CloudQuotaError;
use cloud_storage::StorageError;
use database::DbError;

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    Unauthorized,
    Forbidden,
    BadRequest(String),
    QuotaExceeded(String),
    Internal(String),
}

impl From<DbError> for ApiError {
    fn from(value: DbError) -> Self {
        match value {
            DbError::NotFound(msg) => Self::NotFound(msg),
            DbError::Sqlx(e) => Self::Internal(e.to_string()),
        }
    }
}

impl From<AuthError> for ApiError {
    fn from(value: AuthError) -> Self {
        match value {
            AuthError::InvalidCredentials | AuthError::InvalidToken => Self::Unauthorized,
            AuthError::Forbidden => Self::Forbidden,
            AuthError::InvalidRole => Self::Internal("invalid role".into()),
            AuthError::Internal(msg) => Self::Internal(msg),
        }
    }
}

impl From<BillingQuotaError> for ApiError {
    fn from(value: BillingQuotaError) -> Self {
        match value {
            BillingQuotaError::Exceeded(msg) => Self::QuotaExceeded(msg),
            BillingQuotaError::Db(e) => e.into(),
        }
    }
}

impl From<CloudQuotaError> for ApiError {
    fn from(value: CloudQuotaError) -> Self {
        match value {
            CloudQuotaError::Exceeded(msg) => Self::QuotaExceeded(msg),
            CloudQuotaError::Db(e) => e.into(),
        }
    }
}

impl From<BillingError> for ApiError {
    fn from(value: BillingError) -> Self {
        match value {
            BillingError::Message(msg) => Self::BadRequest(msg),
            BillingError::Security(msg) => Self::Unauthorized,
            BillingError::Db(e) => e.into(),
        }
    }
}

impl From<StorageError> for ApiError {
    fn from(value: StorageError) -> Self {
        match value {
            StorageError::Message(msg) => Self::BadRequest(msg),
            StorageError::Db(e) => e.into(),
            StorageError::Io(e) => Self::Internal(e.to_string()),
        }
    }
}

impl From<MeteringError> for ApiError {
    fn from(value: MeteringError) -> Self {
        match value {
            MeteringError::InvalidMetric => Self::BadRequest("invalid metric".into()),
            MeteringError::Db(e) => e.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".into()),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::QuotaExceeded(msg) => (StatusCode::PAYMENT_REQUIRED, msg),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = serde_json::json!({ "error": message });
        (status, axum::Json(body)).into_response()
    }
}
