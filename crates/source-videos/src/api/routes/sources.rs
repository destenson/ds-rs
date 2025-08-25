use axum::{
    extract::{Path, State},
    Json,
    response::IntoResponse,
};
use std::sync::Arc;
use crate::api::{
    ApiState, ApiError, ApiResult,
    models::{
        AddSourceRequest, SourceResponse, UpdateSourceRequest,
        BatchOperationRequest, BatchOperationResponse, BatchResult,
        SourceTypeRequest, SuccessResponse
    }
};
use crate::{VideoSourceConfig, VideoSourceType};
use uuid::Uuid;

pub async fn list_sources(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<Vec<SourceResponse>>> {
    let sources = state.source_manager.list_sources();
    let responses: Vec<SourceResponse> = sources.into_iter()
        .map(SourceResponse::from)
        .collect();
    
    Ok(Json(responses))
}

pub async fn get_source(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<SourceResponse>> {
    let sources = state.source_manager.list_sources();
    let source = sources.into_iter()
        .find(|s| s.id == id || s.name == id)
        .ok_or_else(|| ApiError::not_found(format!("Source '{}' not found", id)))?;
    
    Ok(Json(SourceResponse::from(source)))
}

pub async fn add_source(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<AddSourceRequest>,
) -> ApiResult<Json<SourceResponse>> {
    let source_type = match req.source_type {
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
        name: req.name.clone(),
        source_type,
        resolution: req.resolution.unwrap_or(crate::config_types::Resolution {
            width: 1920,
            height: 1080,
        }),
        framerate: req.framerate.unwrap_or(crate::config_types::Framerate {
            numerator: 30,
            denominator: 1,
        }),
        format: req.format.unwrap_or(crate::config_types::VideoFormat::I420),
        duration: req.duration,
        num_buffers: None,
        is_live: req.is_live,
    };
    
    let source_id = state.source_manager.add_source(config)?;
    
    // If RTSP server is running, add the source to it as well
    if let Some(rtsp_server) = &state.rtsp_server {
        let mut server = rtsp_server.write().await;
        let rtsp_config = VideoSourceConfig::rtsp(&req.name, &req.name);
        let _ = server.add_source(rtsp_config);
    }
    
    // Get the created source info
    let sources = state.source_manager.list_sources();
    let source = sources.into_iter()
        .find(|s| s.id == source_id)
        .ok_or_else(|| ApiError::internal("Failed to retrieve created source"))?;
    
    Ok(Json(SourceResponse::from(source)))
}

pub async fn remove_source(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<SuccessResponse>> {
    // Remove from RTSP server if running
    if let Some(rtsp_server) = &state.rtsp_server {
        let mut server = rtsp_server.write().await;
        let _ = server.remove_source(&id);
    }
    
    // Remove from source manager
    state.source_manager.remove_source(&id)?;
    
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("Source '{}' removed successfully", id)),
    }))
}

pub async fn update_source(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSourceRequest>,
) -> ApiResult<Json<SourceResponse>> {
    // Find the existing source
    let sources = state.source_manager.list_sources();
    let existing = sources.iter()
        .find(|s| s.id == id || s.name == id)
        .ok_or_else(|| ApiError::not_found(format!("Source '{}' not found", id)))?;
    
    // For now, we'll return the existing source since update isn't fully implemented
    // In a full implementation, you'd update the source properties here
    
    Ok(Json(SourceResponse::from(existing.clone())))
}

pub async fn batch_operations(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<BatchOperationRequest>,
) -> ApiResult<Json<BatchOperationResponse>> {
    let mut results = Vec::new();
    let mut success_count = 0;
    let mut failure_count = 0;
    
    for operation in req.operations {
        let source_name = operation.source.name.clone();
        let op_type = format!("{:?}", operation.operation);
        
        let result = match operation.operation {
            crate::api::models::BatchOperationType::Add => {
                // Convert and add source
                let source_type = match operation.source.source_type {
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
                    name: source_name.clone(),
                    source_type,
                    resolution: operation.source.resolution.unwrap_or(crate::config_types::Resolution {
                        width: 1920,
                        height: 1080,
                    }),
                    framerate: operation.source.framerate.unwrap_or(crate::config_types::Framerate {
                        numerator: 30,
                        denominator: 1,
                    }),
                    format: operation.source.format.unwrap_or(crate::config_types::VideoFormat::I420),
                    duration: operation.source.duration,
                    num_buffers: None,
                    is_live: operation.source.is_live,
                };
                
                match state.source_manager.add_source(config) {
                    Ok(id) => {
                        success_count += 1;
                        BatchResult {
                            operation: op_type,
                            source_name,
                            success: true,
                            result: Some(serde_json::json!({ "id": id })),
                            error: None,
                        }
                    }
                    Err(e) => {
                        failure_count += 1;
                        BatchResult {
                            operation: op_type,
                            source_name,
                            success: false,
                            result: None,
                            error: Some(e.to_string()),
                        }
                    }
                }
            }
            crate::api::models::BatchOperationType::Remove => {
                match state.source_manager.remove_source(&source_name) {
                    Ok(_) => {
                        success_count += 1;
                        BatchResult {
                            operation: op_type,
                            source_name,
                            success: true,
                            result: Some(serde_json::json!({ "removed": true })),
                            error: None,
                        }
                    }
                    Err(e) => {
                        failure_count += 1;
                        BatchResult {
                            operation: op_type,
                            source_name,
                            success: false,
                            result: None,
                            error: Some(e.to_string()),
                        }
                    }
                }
            }
            crate::api::models::BatchOperationType::Update => {
                // Update not fully implemented yet
                failure_count += 1;
                BatchResult {
                    operation: op_type,
                    source_name,
                    success: false,
                    result: None,
                    error: Some("Update operation not yet implemented".to_string()),
                }
            }
        };
        
        results.push(result);
        
        // If atomic mode and we hit a failure, stop processing
        if req.atomic && failure_count > 0 {
            break;
        }
    }
    
    Ok(Json(BatchOperationResponse {
        results,
        success_count,
        failure_count,
    }))
}