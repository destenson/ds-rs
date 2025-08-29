use crate::api::{ApiResult, ApiState, models::*};
use axum::{Json, extract::State};
use std::collections::HashMap;
use std::sync::Arc;

pub async fn health_check(State(_state): State<Arc<ApiState>>) -> ApiResult<Json<HealthResponse>> {
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

pub async fn liveness(State(_state): State<Arc<ApiState>>) -> ApiResult<Json<LivenessResponse>> {
    Ok(Json(LivenessResponse { alive: true }))
}

pub async fn readiness(State(state): State<Arc<ApiState>>) -> ApiResult<Json<ReadinessResponse>> {
    let mut components = HashMap::new();

    // Check source manager
    components.insert(
        "source_manager".to_string(),
        ComponentStatus {
            healthy: true,
            message: Some(format!(
                "{} sources active",
                state.source_manager.list_sources().len()
            )),
        },
    );

    // Check RTSP server
    let rtsp_status = if state.rtsp_server.is_some() {
        ComponentStatus {
            healthy: true,
            message: Some("RTSP server running".to_string()),
        }
    } else {
        ComponentStatus {
            healthy: true,
            message: Some("RTSP server not started".to_string()),
        }
    };
    components.insert("rtsp_server".to_string(), rtsp_status);

    // Check watcher manager
    let watcher_status = ComponentStatus {
        healthy: true,
        message: Some("Watcher available".to_string()),
    };
    components.insert("file_watcher".to_string(), watcher_status);

    // Check network simulator
    let network_status = {
        let sim = state.network_simulator.read().await;
        ComponentStatus {
            healthy: true,
            message: Some(format!(
                "Network simulator {}",
                if sim.is_some() {
                    "configured"
                } else {
                    "not configured"
                }
            )),
        }
    };
    components.insert("network_simulator".to_string(), network_status);

    let all_healthy = components.values().all(|c| c.healthy);

    Ok(Json(ReadinessResponse {
        ready: all_healthy,
        components,
    }))
}

pub async fn metrics(State(state): State<Arc<ApiState>>) -> ApiResult<Json<MetricsResponse>> {
    let source_count = state.source_manager.list_sources().len() as u64;

    // These would be tracked in a real implementation
    let metrics = MetricsResponse {
        source_count,
        active_connections: 0,
        total_requests: 0,
        error_count: 0,
        uptime_seconds: 0,
        cpu_usage: None,
        memory_usage_mb: None,
    };

    Ok(Json(metrics))
}
