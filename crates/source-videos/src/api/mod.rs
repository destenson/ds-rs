use crate::{Result, SourceVideoError, RtspServer, VideoSourceManager, WatcherManager, AppConfig};
use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::State,
    middleware,
};
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use log::{info, error, debug};

pub mod state;
pub mod error;
pub mod auth;
pub mod models;
pub mod routes;

pub use state::ApiState;
pub use error::{ApiError, ApiResult};
pub use models::*;

pub struct ControlApi {
    state: Arc<ApiState>,
    router: Router,
    bind_address: SocketAddr,
}

impl ControlApi {
    pub fn new(
        rtsp_server: Option<Arc<RwLock<RtspServer>>>,
        source_manager: Arc<VideoSourceManager>,
        watcher_manager: Arc<RwLock<WatcherManager>>,
    ) -> Result<Self> {
        let bind_address = "0.0.0.0:3000".parse()
            .map_err(|e| SourceVideoError::config(format!("Invalid bind address: {}", e)))?;
        
        let state = Arc::new(ApiState::new(
            rtsp_server,
            source_manager,
            watcher_manager,
        ));
        
        let router = Self::create_router(state.clone());
        
        Ok(Self {
            state,
            router,
            bind_address,
        })
    }
    
    pub fn with_config(config: &AppConfig, bind_address: SocketAddr) -> Result<Self> {
        let source_manager = Arc::new(VideoSourceManager::new());
        let watcher_manager = Arc::new(RwLock::new(WatcherManager::new()));
        
        let state = Arc::new(ApiState::new(
            None,
            source_manager,
            watcher_manager,
        ));
        
        let mut api = Self {
            state,
            router: Router::new(),
            bind_address,
        };
        
        api.router = Self::create_router(api.state.clone());
        
        Ok(api)
    }
    
    pub fn set_bind_address(&mut self, address: SocketAddr) {
        self.bind_address = address;
    }
    
    fn create_router(state: Arc<ApiState>) -> Router {
        let api_v1 = Router::new()
            // Health endpoints
            .route("/health", get(routes::health::health_check))
            .route("/health/live", get(routes::health::liveness))
            .route("/health/ready", get(routes::health::readiness))
            .route("/metrics", get(routes::health::metrics))
            .route("/status", get(routes::health::health_check))  // Alias for health check
            
            // Source management
            .route("/sources", get(routes::sources::list_sources))
            .route("/sources", post(routes::sources::add_source))
            .route("/sources/{id}", get(routes::sources::get_source))
            .route("/sources/{id}", delete(routes::sources::remove_source))
            .route("/sources/{id}", put(routes::sources::update_source))
            .route("/sources/batch", post(routes::sources::batch_operations))
            
            // Server control
            .route("/server/start", post(routes::server::start_server))
            .route("/server/stop", post(routes::server::stop_server))
            .route("/server/restart", post(routes::server::restart_server))
            .route("/server/status", get(routes::server::server_status))
            .route("/server/info", get(routes::server::server_info))
            .route("/server/urls", get(routes::server::list_urls))
            
            // Configuration
            .route("/config", get(routes::config::get_config))
            .route("/config", put(routes::config::update_config))
            .route("/config/defaults", get(routes::config::get_defaults))
            .route("/config/validate", post(routes::config::validate_config))
            
            // Network simulation
            .route("/network/profiles", get(routes::network::list_profiles))
            .route("/network/apply", post(routes::network::apply_profile))
            .route("/network/conditions", put(routes::network::set_conditions))
            .route("/network/status", get(routes::network::get_status))
            .route("/network/reset", post(routes::network::reset_network))
            .route("/network/update", post(routes::network::set_conditions))  // Alias for set_conditions
            
            // Operations
            .route("/generate", post(routes::operations::generate_video))
            .route("/scan", post(routes::operations::scan_directory))
            .route("/patterns", get(routes::operations::list_patterns))
            .route("/watch/start", post(routes::operations::start_watching))
            .route("/watch/stop", post(routes::operations::stop_watching))
            .route("/watch/status", get(routes::operations::watch_status))
            
            .with_state(state.clone());
        
        Router::new()
            .nest("/api/v1", api_v1)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth::auth_middleware,
            ))
            .layer(tower_http::cors::CorsLayer::very_permissive())
            .layer(tower_http::trace::TraceLayer::new_for_http())
    }
    
    pub async fn bind_and_serve(self) -> Result<()> {
        let listener = TcpListener::bind(self.bind_address).await
            .map_err(|e| SourceVideoError::config(format!("Failed to bind to {}: {}", self.bind_address, e)))?;
        
        info!("Control API listening on http://{}", self.bind_address);
        info!("API documentation available at http://{}/api/docs", self.bind_address);
        
        axum::serve(listener, self.router)
            .with_graceful_shutdown(Self::shutdown_signal())
            .await
            .map_err(|e| SourceVideoError::server(format!("API server error: {}", e)))?;
        
        Ok(())
    }
    
    pub async fn serve_with_shutdown(self, shutdown: impl std::future::Future<Output = ()> + Send + 'static) -> Result<()> {
        let listener = TcpListener::bind(self.bind_address).await
            .map_err(|e| SourceVideoError::config(format!("Failed to bind to {}: {}", self.bind_address, e)))?;
        
        info!("Control API listening on http://{}", self.bind_address);
        
        axum::serve(listener, self.router)
            .with_graceful_shutdown(shutdown)
            .await
            .map_err(|e| SourceVideoError::server(format!("API server error: {}", e)))?;
        
        Ok(())
    }
    
    async fn shutdown_signal() {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        
        info!("Shutting down API server...");
    }
    
    pub fn state(&self) -> &Arc<ApiState> {
        &self.state
    }
    
    pub fn router(&self) -> Router {
        self.router.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_api_creation() {
        let source_manager = Arc::new(VideoSourceManager::new());
        let watcher_manager = Arc::new(RwLock::new(WatcherManager::new()));
        
        let api = ControlApi::new(None, source_manager, watcher_manager).unwrap();
        assert!(!api.bind_address.to_string().is_empty());
    }
}