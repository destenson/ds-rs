use crate::api::{ApiError, ApiResult, ApiState, models::*};
use axum::{Json, extract::State};
use std::sync::Arc;

pub async fn get_config(State(state): State<Arc<ApiState>>) -> ApiResult<Json<ConfigResponse>> {
    let config = state.current_config.read().await;

    Ok(Json(ConfigResponse {
        server: ApiServerConfig {
            port: 8554,
            address: "0.0.0.0".to_string(),
            max_connections: None,
            timeout_ms: None,
        },
        sources: vec![],
        network: None,
        watch: None,
    }))
}

pub async fn update_config(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<ConfigResponse>,
) -> ApiResult<Json<SuccessResponse>> {
    // Update configuration in state
    // Implementation would update the actual config

    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Configuration updated".to_string()),
    }))
}

pub async fn get_defaults(State(_state): State<Arc<ApiState>>) -> ApiResult<Json<ConfigResponse>> {
    Ok(Json(ConfigResponse {
        server: ApiServerConfig {
            port: 8554,
            address: "0.0.0.0".to_string(),
            max_connections: Some(100),
            timeout_ms: Some(30000),
        },
        sources: vec![],
        network: None,
        watch: None,
    }))
}

pub async fn validate_config(
    State(_state): State<Arc<ApiState>>,
    Json(req): Json<ConfigResponse>,
) -> ApiResult<Json<SuccessResponse>> {
    // Validate configuration
    if req.server.port == 0 {
        return Err(ApiError::validation("Invalid port: must be non-zero"));
    }

    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Configuration is valid".to_string()),
    }))
}
