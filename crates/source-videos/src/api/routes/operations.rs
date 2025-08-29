use crate::api::{ApiError, ApiResult, ApiState, models::*};
use crate::{DirectoryConfig, DirectoryScanner, FilterConfig, TestPattern, generate_test_file};
use axum::{Json, extract::State};
use std::path::PathBuf;
use std::sync::Arc;

pub async fn generate_video(
    State(_state): State<Arc<ApiState>>,
    Json(req): Json<GenerateVideoRequest>,
) -> ApiResult<Json<SuccessResponse>> {
    let output_path = PathBuf::from(&req.output);
    let width = req.resolution.width;
    let height = req.resolution.height;
    let fps = req.framerate.numerator / req.framerate.denominator.max(1);

    // Generate the test video file
    generate_test_file(&req.pattern, req.duration, &output_path)
        .map_err(|e| ApiError::internal(format!("Failed to generate video: {}", e)))?;

    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("Generated video at: {}", req.output)),
    }))
}

pub async fn scan_directory(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<ScanDirectoryRequest>,
) -> ApiResult<Json<ScanDirectoryResponse>> {
    let filters = if !req.include.is_empty() || !req.exclude.is_empty() {
        Some(FilterConfig {
            include: req.include,
            exclude: req.exclude,
            extensions: vec![],
        })
    } else {
        None
    };

    let dir_config = DirectoryConfig {
        path: req.path.clone(),
        recursive: req.recursive,
        filters,
        lazy_loading: false,
        mount_prefix: None,
    };

    let mut scanner = DirectoryScanner::new(dir_config);
    let source_configs = scanner
        .scan()
        .map_err(|e| ApiError::internal(format!("Failed to scan directory: {}", e)))?;

    let found_count = source_configs.len();
    let mut added_count = 0;
    let mut sources = Vec::new();

    if req.add_to_server {
        for config in source_configs {
            match state.source_manager.add_source(config.clone()) {
                Ok(id) => {
                    added_count += 1;
                    sources.push(SourceResponse {
                        id,
                        name: config.name.clone(),
                        uri: format!("file://{}", config.name),
                        state: "ready".to_string(),
                        source_type: "file".to_string(),
                        created_at: Some(chrono::Utc::now().to_rfc3339()),
                        metadata: None,
                    });
                }
                Err(_) => {
                    // Skip files that couldn't be added
                }
            }
        }
    }

    Ok(Json(ScanDirectoryResponse {
        found_count,
        added_count,
        sources,
    }))
}

pub async fn list_patterns(
    State(_state): State<Arc<ApiState>>,
) -> ApiResult<Json<Vec<PatternResponse>>> {
    let patterns = TestPattern::all()
        .into_iter()
        .map(|p| {
            let animated = TestPattern::animated_patterns().contains(&p);
            PatternResponse {
                name: format!("{:?}", p).to_lowercase(),
                description: p.description().to_string(),
                animated,
            }
        })
        .collect();

    Ok(Json(patterns))
}

pub async fn start_watching(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<StartWatchingRequest>,
) -> ApiResult<Json<SuccessResponse>> {
    let mut watcher_manager = state.watcher_manager.write().await;
    let path = PathBuf::from(&req.directory);

    let watcher_id = watcher_manager
        .add_directory_watcher(&path, req.recursive)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to start watching: {}", e)))?;

    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!(
            "Started watching directory with ID: {}",
            watcher_id
        )),
    }))
}

pub async fn stop_watching(State(state): State<Arc<ApiState>>) -> ApiResult<Json<SuccessResponse>> {
    let mut watcher_manager = state.watcher_manager.write().await;
    watcher_manager.stop_all();

    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Stopped all file watchers".to_string()),
    }))
}

pub async fn watch_status(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<WatchStatusResponse>> {
    let watcher_manager = state.watcher_manager.read().await;

    // Check if any watchers are active
    let active = true; // Simplified - would check actual watcher state

    // Get watcher info
    let watchers = if active {
        vec![WatcherInfo {
            id: "default".to_string(),
            directory: ".".to_string(),
            recursive: true,
            events_received: 0,
        }]
    } else {
        vec![]
    };

    Ok(Json(WatchStatusResponse { active, watchers }))
}
