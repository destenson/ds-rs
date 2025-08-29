use crate::{
    AppConfig, RtspServer, VideoSourceManager, WatcherManager,
    network::{GStreamerNetworkSimulator, NetworkConditions, NetworkController, NetworkProfile},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ApiState {
    pub rtsp_server: Option<Arc<RwLock<RtspServer>>>,
    pub source_manager: Arc<VideoSourceManager>,
    pub watcher_manager: Arc<RwLock<WatcherManager>>,
    pub network_simulator: Arc<RwLock<Option<GStreamerNetworkSimulator>>>,
    pub current_config: Arc<RwLock<AppConfig>>,
    pub operation_status: Arc<RwLock<HashMap<String, OperationStatus>>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OperationStatus {
    pub id: String,
    pub operation: String,
    pub status: Status,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Running,
    Completed,
    Failed,
}

impl ApiState {
    pub fn new(
        rtsp_server: Option<Arc<RwLock<RtspServer>>>,
        source_manager: Arc<VideoSourceManager>,
        watcher_manager: Arc<RwLock<WatcherManager>>,
    ) -> Self {
        Self {
            rtsp_server,
            source_manager,
            watcher_manager,
            network_simulator: Arc::new(RwLock::new(None)),
            current_config: Arc::new(RwLock::new(AppConfig::default())),
            operation_status: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_or_create_rtsp_server(
        &self,
        port: u16,
    ) -> Result<Arc<RwLock<RtspServer>>, crate::SourceVideoError> {
        if let Some(ref server) = self.rtsp_server {
            return Ok(server.clone());
        }

        // This is a simplified version - in production you'd want proper synchronization
        Err(crate::SourceVideoError::config(
            "RTSP server not initialized",
        ))
    }

    pub async fn track_operation(&self, id: String, operation: String) -> String {
        let status = OperationStatus {
            id: id.clone(),
            operation,
            status: Status::Running,
            started_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            result: None,
            error: None,
        };

        self.operation_status
            .write()
            .await
            .insert(id.clone(), status);
        id
    }

    pub async fn complete_operation(&self, id: &str, result: serde_json::Value) {
        if let Some(status) = self.operation_status.write().await.get_mut(id) {
            status.status = Status::Completed;
            status.completed_at = Some(chrono::Utc::now().to_rfc3339());
            status.result = Some(result);
        }
    }

    pub async fn fail_operation(&self, id: &str, error: String) {
        if let Some(status) = self.operation_status.write().await.get_mut(id) {
            status.status = Status::Failed;
            status.completed_at = Some(chrono::Utc::now().to_rfc3339());
            status.error = Some(error);
        }
    }

    pub async fn get_operation_status(&self, id: &str) -> Option<OperationStatus> {
        self.operation_status.read().await.get(id).cloned()
    }

    pub async fn apply_network_profile(
        &self,
        profile: NetworkProfile,
    ) -> Result<(), crate::SourceVideoError> {
        let mut sim_guard = self.network_simulator.write().await;

        if sim_guard.is_none() {
            *sim_guard = Some(GStreamerNetworkSimulator::new());
        }

        if let Some(ref mut sim) = *sim_guard {
            sim.apply_profile(profile);
        }

        Ok(())
    }

    pub async fn apply_custom_network_conditions(
        &self,
        conditions: NetworkConditions,
    ) -> Result<(), crate::SourceVideoError> {
        let mut sim_guard = self.network_simulator.write().await;

        if sim_guard.is_none() {
            *sim_guard = Some(GStreamerNetworkSimulator::new());
        }

        if let Some(ref mut sim) = *sim_guard {
            sim.apply_conditions(conditions);
        }

        Ok(())
    }

    pub async fn reset_network(&self) -> Result<(), crate::SourceVideoError> {
        let mut sim_guard = self.network_simulator.write().await;

        if let Some(ref mut sim) = *sim_guard {
            sim.reset();
        }

        Ok(())
    }

    pub async fn get_network_status(&self) -> Option<NetworkConditions> {
        let sim_guard = self.network_simulator.read().await;

        if let Some(ref sim) = *sim_guard {
            Some(sim.get_conditions())
        } else {
            None
        }
    }
}

// Add chrono to dependencies for timestamp handling
// This will be added to Cargo.toml when we update it
