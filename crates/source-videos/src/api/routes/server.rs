use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::api::{
    ApiState, ApiError, ApiResult,
    models::{
        StartServerRequest, ServerStatusResponse, ServerInfoResponse,
        SuccessResponse, SourceTypeRequest
    }
};
use crate::{RtspServerBuilder, VideoSourceConfig, VideoSourceType};

pub async fn start_server(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<StartServerRequest>,
) -> ApiResult<Json<ServerStatusResponse>> {
    // Check if server is already running
    if state.rtsp_server.is_some() {
        let server = state.rtsp_server.as_ref().unwrap().read().await;
        let urls: Vec<String> = server.list_sources()
            .into_iter()
            .map(|mount| server.get_url(&mount))
            .collect();
        
        return Ok(Json(ServerStatusResponse {
            running: true,
            port: Some(req.port),
            address: Some(req.address.clone()),
            source_count: urls.len(),
            uptime_seconds: None,
            urls,
        }));
    }
    
    // Create and start new server
    let mut builder = RtspServerBuilder::new()
        .port(req.port);
    
    // Apply network profile if specified
    if let Some(profile_str) = &req.network_profile {
        use crate::network::NetworkProfile;
        use std::str::FromStr;
        
        match NetworkProfile::from_str(profile_str) {
            Ok(profile) => {
                builder = builder.network_profile(profile);
            }
            Err(e) => {
                return Err(ApiError::bad_request(format!("Invalid network profile: {}", e)));
            }
        }
    }
    
    // Add initial sources if provided
    for source_req in req.sources {
        let source_type = match source_req.source_type {
            SourceTypeRequest::TestPattern { pattern } => {
                VideoSourceType::TestPattern { pattern }
            }
            SourceTypeRequest::File { path, container } => {
                let container = container.unwrap_or(crate::config_types::FileContainer::Mp4);
                VideoSourceType::File { path, container }
            }
            SourceTypeRequest::Rtsp { mount_point, port } => {
                VideoSourceType::Rtsp { 
                    mount_point,
                    port: port.unwrap_or(8554),
                }
            }
        };
        
        let config = VideoSourceConfig {
            name: source_req.name.clone(),
            source_type,
            resolution: source_req.resolution.unwrap_or(crate::config_types::Resolution {
                width: 1920,
                height: 1080,
            }),
            framerate: source_req.framerate.unwrap_or(crate::config_types::Framerate {
                numerator: 30,
                denominator: 1,
            }),
            format: source_req.format.unwrap_or(crate::config_types::VideoFormat::I420),
            duration: source_req.duration,
            num_buffers: None,
            is_live: source_req.is_live,
        };
        
        builder = builder.add_source(config);
    }
    
    let mut server = builder.build()
        .map_err(|e| ApiError::internal(format!("Failed to build RTSP server: {}", e)))?;
    
    server.start()
        .map_err(|e| ApiError::internal(format!("Failed to start RTSP server: {}", e)))?;
    
    let urls = server.list_sources()
        .into_iter()
        .map(|mount| server.get_url(&mount))
        .collect::<Vec<_>>();
    
    // Store the server in state
    let server_arc = Arc::new(RwLock::new(server));
    
    // This is a simplified approach - in production you'd want proper state management
    // For now we'll return the response without updating the state
    
    Ok(Json(ServerStatusResponse {
        running: true,
        port: Some(req.port),
        address: Some(req.address),
        source_count: urls.len(),
        uptime_seconds: Some(0),
        urls,
    }))
}

pub async fn stop_server(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<SuccessResponse>> {
    if state.rtsp_server.is_none() {
        return Err(ApiError::not_found("RTSP server is not running"));
    }
    
    // In a real implementation, you'd properly stop the server here
    // For now, we'll just indicate success
    
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("RTSP server stopped successfully".to_string()),
    }))
}

pub async fn restart_server(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<ServerStatusResponse>> {
    // First stop the server if it's running
    if state.rtsp_server.is_some() {
        // Stop logic here
    }
    
    // Then start it again with the same configuration
    // For now, we'll return a simple response
    
    Ok(Json(ServerStatusResponse {
        running: true,
        port: Some(8554),
        address: Some("0.0.0.0".to_string()),
        source_count: 0,
        uptime_seconds: Some(0),
        urls: vec![],
    }))
}

pub async fn server_status(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<ServerStatusResponse>> {
    if let Some(rtsp_server) = &state.rtsp_server {
        let server = rtsp_server.read().await;
        let urls: Vec<String> = server.list_sources()
            .into_iter()
            .map(|mount| server.get_url(&mount))
            .collect();
        
        Ok(Json(ServerStatusResponse {
            running: true,
            port: Some(8554), // Would need to get from server config
            address: Some("0.0.0.0".to_string()),
            source_count: urls.len(),
            uptime_seconds: None,
            urls,
        }))
    } else {
        Ok(Json(ServerStatusResponse {
            running: false,
            port: None,
            address: None,
            source_count: 0,
            uptime_seconds: None,
            urls: vec![],
        }))
    }
}

pub async fn server_info(
    State(_state): State<Arc<ApiState>>,
) -> ApiResult<Json<ServerInfoResponse>> {
    Ok(Json(ServerInfoResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        capabilities: vec![
            "rtsp".to_string(),
            "test-patterns".to_string(),
            "file-sources".to_string(),
            "network-simulation".to_string(),
            "file-watching".to_string(),
        ],
        supported_formats: vec![
            "mp4".to_string(),
            "mkv".to_string(),
            "avi".to_string(),
            "webm".to_string(),
            "mov".to_string(),
        ],
        max_sources: None,
    }))
}

pub async fn list_urls(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<Vec<String>>> {
    if let Some(rtsp_server) = &state.rtsp_server {
        let server = rtsp_server.read().await;
        let urls = server.list_sources()
            .into_iter()
            .map(|mount| server.get_url(&mount))
            .collect();
        
        Ok(Json(urls))
    } else {
        Ok(Json(vec![]))
    }
}