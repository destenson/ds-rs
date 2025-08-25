#[cfg(test)]
mod api_integration_tests {
    use source_videos::api::{ControlApi, ApiState};
    use source_videos::{VideoSourceManager, WatcherManager};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    
    async fn setup_test_api() -> TestServer {
        let source_manager = Arc::new(VideoSourceManager::new());
        let watcher_manager = Arc::new(RwLock::new(WatcherManager::new()));
        
        let api = ControlApi::new(None, source_manager, watcher_manager).unwrap();
        let app = api.router();
        
        TestServer::new(app).unwrap()
    }
    
    #[tokio::test]
    async fn test_health_endpoint() {
        let server = setup_test_api().await;
        
        let response = server
            .get("/api/v1/health")
            .await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let json: serde_json::Value = response.json();
        assert_eq!(json["status"], "healthy");
    }
    
    #[tokio::test]
    async fn test_list_sources_empty() {
        let server = setup_test_api().await;
        
        let response = server
            .get("/api/v1/sources")
            .await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let json: Vec<serde_json::Value> = response.json();
        assert!(json.is_empty());
    }
    
    #[tokio::test]
    async fn test_add_source() {
        let server = setup_test_api().await;
        
        let response = server
            .post("/api/v1/sources")
            .json(&serde_json::json!({
                "name": "test_source",
                "type": "test_pattern",
                "pattern": "smpte"
            }))
            .await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let json: serde_json::Value = response.json();
        assert_eq!(json["name"], "test_source");
    }
    
    #[tokio::test]
    async fn test_network_profiles() {
        let server = setup_test_api().await;
        
        let response = server
            .get("/api/v1/network/profiles")
            .await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let json: Vec<serde_json::Value> = response.json();
        assert!(!json.is_empty());
    }
    
    #[tokio::test]
    async fn test_metrics_endpoint() {
        let server = setup_test_api().await;
        
        let response = server
            .get("/api/v1/metrics")
            .await;
        
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let json: serde_json::Value = response.json();
        assert!(json["source_count"].is_number());
    }
}