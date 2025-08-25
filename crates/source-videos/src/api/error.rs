use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    InternalError(String),
    ServiceUnavailable(String),
    ValidationError(String),
}

impl ApiError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }
    
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }
    
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }
    
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
    
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }
    
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }
    
    pub fn service_unavailable(msg: impl Into<String>) -> Self {
        Self::ServiceUnavailable(msg.into())
    }
    
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Self::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            Self::NotFound(msg) => write!(f, "Not Found: {}", msg),
            Self::Conflict(msg) => write!(f, "Conflict: {}", msg),
            Self::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            Self::ServiceUnavailable(msg) => write!(f, "Service Unavailable: {}", msg),
            Self::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            Self::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg),
            Self::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg),
            Self::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable", msg),
            Self::ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, "validation_error", msg),
        };
        
        let body = Json(json!({
            "error": {
                "type": error_type,
                "message": message,
                "status": status.as_u16(),
            }
        }));
        
        (status, body).into_response()
    }
}

impl From<crate::SourceVideoError> for ApiError {
    fn from(err: crate::SourceVideoError) -> Self {
        use crate::SourceVideoError;
        
        match err {
            SourceVideoError::Configuration(_) => Self::BadRequest(err.to_string()),
            SourceVideoError::SourceNotFound(_) => Self::NotFound(err.to_string()),
            SourceVideoError::Server(_) => Self::ServiceUnavailable(err.to_string()),
            SourceVideoError::Resource(_) => Self::Conflict(err.to_string()),
            SourceVideoError::Pipeline(_) => Self::ServiceUnavailable(err.to_string()),
            _ => Self::InternalError(err.to_string()),
        }
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        Self::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::ValidationError(err.to_string())
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for ApiError {
    fn from(err: tokio::sync::oneshot::error::RecvError) -> Self {
        Self::InternalError(format!("Channel error: {}", err))
    }
}

pub fn handle_rejection(err: axum::extract::rejection::JsonRejection) -> ApiError {
    ApiError::ValidationError(format!("Invalid JSON: {}", err))
}