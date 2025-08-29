use super::{
    SourceController, SourceEvent, SourceId,
    circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager},
    recovery::{RecoveryConfig, RecoveryManager},
};
use crate::error::Result;
use crate::pipeline::Pipeline;
use gstreamer as gst;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Simple fault-tolerant wrapper around SourceController
pub struct FaultTolerantSourceController {
    inner: Arc<SourceController>,
    recovery_managers: Arc<Mutex<HashMap<SourceId, Arc<RecoveryManager>>>>,
    circuit_breaker: Arc<CircuitBreakerManager>,
    source_uris: Arc<Mutex<HashMap<SourceId, String>>>,
}

impl FaultTolerantSourceController {
    pub fn new(pipeline: Arc<Pipeline>, streammux: gst::Element) -> Self {
        let controller = Arc::new(SourceController::new(pipeline, streammux));
        Self::wrap(controller)
    }

    pub fn wrap(controller: Arc<SourceController>) -> Self {
        let ft_controller = Self {
            inner: controller.clone(),
            recovery_managers: Arc::new(Mutex::new(HashMap::new())),
            circuit_breaker: Arc::new(CircuitBreakerManager::new()),
            source_uris: Arc::new(Mutex::new(HashMap::new())),
        };

        // Register error handler for automatic recovery
        ft_controller.setup_error_handler();

        ft_controller
    }

    fn setup_error_handler(&self) {
        let controller = self.inner.clone();
        let recovery_managers = self.recovery_managers.clone();
        let circuit_breaker = self.circuit_breaker.clone();
        let source_uris = self.source_uris.clone();

        self.inner
            .get_event_handler()
            .register_callback(move |event| {
                if let SourceEvent::Error { id, error } = event {
                    eprintln!("Source {} error: {}", id, error);

                    // Try to recover the source
                    if let Some(uri) = source_uris.lock().unwrap().get(id).cloned() {
                        if let Some(recovery_mgr) = recovery_managers.lock().unwrap().get(id) {
                            if recovery_mgr.should_retry() {
                                // Simple recovery: wait and reconnect
                                let backoff = recovery_mgr.calculate_backoff(1); // Simple retry count
                                thread::sleep(backoff);

                                // Try to restart the source
                                if controller.restart_source(*id).is_ok() {
                                    recovery_mgr.mark_recovered();
                                } else {
                                    recovery_mgr.mark_failed(error.clone());
                                }
                            }
                        }
                    }
                }
            });
    }

    pub fn add_source(&self, uri: &str) -> Result<SourceId> {
        let id = self.inner.add_source(uri)?;

        // Track URI for recovery
        self.source_uris.lock().unwrap().insert(id, uri.to_string());

        // Set up recovery manager with default config
        let recovery_mgr = Arc::new(RecoveryManager::new(RecoveryConfig::default()));
        self.recovery_managers
            .lock()
            .unwrap()
            .insert(id, recovery_mgr);

        // Create circuit breaker for this source
        let cb_config = CircuitBreakerConfig::default();
        self.circuit_breaker
            .get_or_create(format!("source-{}", id), cb_config);

        Ok(id)
    }

    pub fn remove_source(&self, id: SourceId) -> Result<()> {
        // Clean up recovery resources
        self.source_uris.lock().unwrap().remove(&id);
        self.recovery_managers.lock().unwrap().remove(&id);

        self.inner.remove_source(id)
    }

    // Delegate other methods to inner controller
    pub fn list_active_sources(&self) -> Result<Vec<(SourceId, String, super::SourceState)>> {
        self.inner.list_active_sources()
    }

    pub fn restart_source(&self, id: SourceId) -> Result<()> {
        self.inner.restart_source(id)
    }

    pub fn get_inner(&self) -> Arc<SourceController> {
        self.inner.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::Pipeline;

    #[test]
    fn test_fault_tolerant_controller() {
        gst::init().unwrap();

        let pipeline = Arc::new(Pipeline::new("test").unwrap());
        let mux = gst::ElementFactory::make("identity")
            .name("test-mux")
            .build()
            .unwrap();

        let controller = FaultTolerantSourceController::new(pipeline, mux);

        // Should be able to add sources with recovery
        let result = controller.add_source("file:///test.mp4");
        assert!(result.is_ok());
    }
}
