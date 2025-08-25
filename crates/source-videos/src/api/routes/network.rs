use axum::{extract::State, Json};
use std::sync::Arc;
use crate::api::{ApiState, ApiError, ApiResult, models::*};
use crate::network::{NetworkProfile, NetworkConditions};
use std::str::FromStr;

pub async fn list_profiles(
    State(_state): State<Arc<ApiState>>,
) -> ApiResult<Json<Vec<NetworkProfileResponse>>> {
    let profiles = vec![
        NetworkProfileResponse {
            name: "perfect".to_string(),
            description: "Perfect network conditions".to_string(),
            packet_loss: 0.0,
            latency_ms: 0,
            bandwidth_kbps: 0,
            jitter_ms: 0,
        },
        NetworkProfileResponse {
            name: "3g".to_string(),
            description: "3G mobile network".to_string(),
            packet_loss: 2.0,
            latency_ms: 100,
            bandwidth_kbps: 1600,
            jitter_ms: 30,
        },
        NetworkProfileResponse {
            name: "4g".to_string(),
            description: "4G/LTE mobile network".to_string(),
            packet_loss: 0.5,
            latency_ms: 50,
            bandwidth_kbps: 12000,
            jitter_ms: 10,
        },
        NetworkProfileResponse {
            name: "5g".to_string(),
            description: "5G mobile network".to_string(),
            packet_loss: 0.1,
            latency_ms: 10,
            bandwidth_kbps: 100000,
            jitter_ms: 2,
        },
        NetworkProfileResponse {
            name: "wifi".to_string(),
            description: "Typical WiFi connection".to_string(),
            packet_loss: 0.2,
            latency_ms: 5,
            bandwidth_kbps: 50000,
            jitter_ms: 3,
        },
        NetworkProfileResponse {
            name: "poor".to_string(),
            description: "Poor network conditions".to_string(),
            packet_loss: 5.0,
            latency_ms: 200,
            bandwidth_kbps: 512,
            jitter_ms: 50,
        },
    ];
    
    Ok(Json(profiles))
}

pub async fn apply_profile(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<ApplyNetworkProfileRequest>,
) -> ApiResult<Json<SuccessResponse>> {
    let profile = NetworkProfile::from_str(&req.profile)
        .map_err(|e| ApiError::bad_request(format!("Invalid profile: {}", e)))?;
    
    state.apply_network_profile(profile).await?;
    
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("Applied network profile: {}", req.profile)),
    }))
}

pub async fn set_conditions(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<CustomNetworkConditionsRequest>,
) -> ApiResult<Json<SuccessResponse>> {
    let conditions = NetworkConditions {
        packet_loss: req.packet_loss.unwrap_or(0.0),
        latency_ms: req.latency_ms.unwrap_or(0),
        bandwidth_kbps: req.bandwidth_kbps.unwrap_or(0),
        jitter_ms: req.jitter_ms.unwrap_or(0),
        connection_dropped: false,
        duplicate_probability: 0.0,
        allow_reordering: true,
        min_delay_ms: 0,
        max_delay_ms: req.latency_ms.unwrap_or(0) + req.jitter_ms.unwrap_or(0),
        delay_probability: if req.latency_ms.unwrap_or(0) > 0 { 100.0 } else { 0.0 },
    };
    
    state.apply_custom_network_conditions(conditions).await?;
    
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Applied custom network conditions".to_string()),
    }))
}

pub async fn get_status(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<NetworkStatusResponse>> {
    let conditions = state.get_network_status().await;
    
    if let Some(conditions) = conditions {
        Ok(Json(NetworkStatusResponse {
            active: true,
            profile: None,
            conditions: NetworkConditionsResponse {
                packet_loss: conditions.packet_loss,
                latency_ms: conditions.latency_ms,
                bandwidth_kbps: conditions.bandwidth_kbps,
                jitter_ms: conditions.jitter_ms,
                connection_dropped: conditions.connection_dropped,
            },
            per_source: None,
        }))
    } else {
        Ok(Json(NetworkStatusResponse {
            active: false,
            profile: None,
            conditions: NetworkConditionsResponse {
                packet_loss: 0.0,
                latency_ms: 0,
                bandwidth_kbps: 0,
                jitter_ms: 0,
                connection_dropped: false,
            },
            per_source: None,
        }))
    }
}

pub async fn reset_network(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<SuccessResponse>> {
    state.reset_network().await?;
    
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Network conditions reset to perfect".to_string()),
    }))
}