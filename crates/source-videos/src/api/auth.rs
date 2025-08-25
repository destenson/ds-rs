use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use super::{ApiError, ApiState};

const AUTH_HEADER: &str = "authorization";
const API_KEY_HEADER: &str = "x-api-key";

#[derive(Clone)]
pub struct ApiAuthConfig {
    pub enabled: bool,
    pub token: Option<String>,
    pub api_key: Option<String>,
    pub bypass_local: bool,
}

impl Default for ApiAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            token: std::env::var("API_AUTH_TOKEN").ok(),
            api_key: std::env::var("API_KEY").ok(),
            bypass_local: true,
        }
    }
}

impl ApiAuthConfig {
    pub fn from_env() -> Self {
        let enabled = std::env::var("API_AUTH_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        
        let bypass_local = std::env::var("API_AUTH_BYPASS_LOCAL")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        Self {
            enabled,
            token: std::env::var("API_AUTH_TOKEN").ok(),
            api_key: std::env::var("API_KEY").ok(),
            bypass_local,
        }
    }
    
    pub fn is_valid_token(&self, token: &str) -> bool {
        if let Some(expected) = &self.token {
            if token.starts_with("Bearer ") {
                return &token[7..] == expected;
            }
        }
        false
    }
    
    pub fn is_valid_api_key(&self, key: &str) -> bool {
        if let Some(expected) = &self.api_key {
            return key == expected;
        }
        false
    }
}

pub async fn auth_middleware(
    State(state): State<Arc<ApiState>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let path = request.uri().path();
    
    // Always allow health check endpoints
    if path.starts_with("/api/v1/health") || path == "/api/v1/metrics" {
        return Ok(next.run(request).await);
    }
    
    let auth_config = ApiAuthConfig::from_env();
    
    // If auth is not enabled, allow all requests
    if !auth_config.enabled {
        return Ok(next.run(request).await);
    }
    
    // Check if request is from localhost and bypass is enabled
    if auth_config.bypass_local {
        // This would need proper implementation to check request origin
        // For now, we'll skip this check
    }
    
    // Check Authorization header
    if let Some(auth_header) = request.headers().get(AUTH_HEADER) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_config.is_valid_token(auth_str) {
                return Ok(next.run(request).await);
            }
        }
    }
    
    // Check API key header
    if let Some(api_key_header) = request.headers().get(API_KEY_HEADER) {
        if let Ok(api_key) = api_key_header.to_str() {
            if auth_config.is_valid_api_key(api_key) {
                return Ok(next.run(request).await);
            }
        }
    }
    
    Err(ApiError::unauthorized("Invalid or missing authentication"))
}

pub fn require_auth(enabled: bool) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |req: Request, next: Next| {
        Box::pin(async move {
            if !enabled {
                return Ok(next.run(req).await);
            }
            
            let auth_config = ApiAuthConfig::from_env();
            
            // Check Authorization header
            if let Some(auth_header) = req.headers().get(AUTH_HEADER) {
                if let Ok(auth_str) = auth_header.to_str() {
                    if auth_config.is_valid_token(auth_str) {
                        return Ok(next.run(req).await);
                    }
                }
            }
            
            // Check API key header
            if let Some(api_key_header) = req.headers().get(API_KEY_HEADER) {
                if let Ok(api_key) = api_key_header.to_str() {
                    if auth_config.is_valid_api_key(api_key) {
                        return Ok(next.run(req).await);
                    }
                }
            }
            
            Err(StatusCode::UNAUTHORIZED)
        })
    }
}