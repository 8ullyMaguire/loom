use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: String,
    pub message: String,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}

impl ApiError {
    pub fn auth(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "auth_required",
            message: message.into(),
        }
    }
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: message.into(),
        }
    }
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid",
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.code.into(),
                message: self.message,
            }),
        )
            .into_response()
    }
}

impl From<loom_store::Error> for ApiError {
    fn from(e: loom_store::Error) -> Self {
        match e {
            loom_store::Error::Auth => Self::auth("a valid edit token is required"),
            loom_store::Error::NotFound => Self::not_found("bundle not found"),
            loom_store::Error::Invalid(m) => Self::invalid(m),
            loom_store::Error::Database(m) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "database",
                message: m.to_string(),
            },
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        Self::from(loom_store::Error::Database(e))
    }
}
