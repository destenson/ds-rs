use super::{SourceId, SourceInfo};
use crate::error::{DeepStreamError, Result};
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Isolation policy for source failures
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IsolationPolicy {
    /// No isolation - failure affects pipeline
    None,
    /// Basic isolation - catch panics
    Basic,
    /// Full isolation - separate thread with panic catching
    Full,
}

/// Result of an isolated operation
#[derive(Debug)]
pub enum IsolationResult<T> {
    /// Operation succeeded
    Success(T),
    /// Operation failed with error
    Error(DeepStreamError),
    /// Operation panicked
    Panic(String),
    /// Operation timed out
    Timeout,
}

/// Error boundary for isolating source failures
pub struct ErrorBoundary {
    source_id: SourceId,
    policy: IsolationPolicy,
    timeout: Option<Duration>,
    panic_count: Arc<Mutex<usize>>,
    error_count: Arc<Mutex<usize>>,
}

impl ErrorBoundary {
    pub fn new(source_id: SourceId, policy: IsolationPolicy) -> Self {
        Self {
            source_id,
            policy,
            timeout: Some(Duration::from_secs(30)),
            panic_count: Arc::new(Mutex::new(0)),
            error_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Set timeout for isolated operations
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Execute a function within the error boundary
    pub fn execute<F, T>(&self, f: F) -> IsolationResult<T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        match self.policy {
            IsolationPolicy::None => {
                // No isolation, execute directly
                match f() {
                    Ok(result) => IsolationResult::Success(result),
                    Err(e) => {
                        self.record_error();
                        IsolationResult::Error(e)
                    }
                }
            }
            IsolationPolicy::Basic => {
                // Basic isolation with panic catching
                self.execute_with_panic_handler(f)
            }
            IsolationPolicy::Full => {
                // Full isolation in separate thread
                self.execute_in_thread(f)
            }
        }
    }

    /// Execute with panic handler
    fn execute_with_panic_handler<F, T>(&self, f: F) -> IsolationResult<T>
    where
        F: FnOnce() -> Result<T>,
        T: Send,
    {
        let result = panic::catch_unwind(AssertUnwindSafe(|| f()));

        match result {
            Ok(Ok(value)) => IsolationResult::Success(value),
            Ok(Err(e)) => {
                self.record_error();
                IsolationResult::Error(e)
            }
            Err(panic_info) => {
                self.record_panic();
                let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                    (*s).to_string()
                } else {
                    "Unknown panic".to_string()
                };

                log::error!("Source {} panicked: {}", self.source_id, msg);
                IsolationResult::Panic(msg)
            }
        }
    }

    /// Execute in a separate thread with timeout
    fn execute_in_thread<F, T>(&self, f: F) -> IsolationResult<T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();
        let source_id = self.source_id;

        let handle = thread::spawn(move || {
            let result = panic::catch_unwind(AssertUnwindSafe(|| f()));

            match result {
                Ok(Ok(value)) => {
                    let _ = tx.send(IsolationResult::Success(value));
                }
                Ok(Err(e)) => {
                    let _ = tx.send(IsolationResult::Error(e));
                }
                Err(panic_info) => {
                    let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                        (*s).to_string()
                    } else {
                        "Unknown panic".to_string()
                    };

                    log::error!("Source {} thread panicked: {}", source_id, msg);
                    let _ = tx.send(IsolationResult::Panic(msg));
                }
            }
        });

        // Wait with timeout
        let timeout = self.timeout.unwrap_or(Duration::from_secs(30));
        match rx.recv_timeout(timeout) {
            Ok(result) => {
                // Thread completed
                let _ = handle.join();

                match &result {
                    IsolationResult::Error(_) => self.record_error(),
                    IsolationResult::Panic(_) => self.record_panic(),
                    _ => {}
                }

                result
            }
            Err(_) => {
                // Timeout occurred
                log::error!("Source {} operation timed out", self.source_id);
                IsolationResult::Timeout
            }
        }
    }

    /// Record an error occurrence
    fn record_error(&self) {
        let mut count = self.error_count.lock().unwrap();
        *count += 1;
    }

    /// Record a panic occurrence
    fn record_panic(&self) {
        let mut count = self.panic_count.lock().unwrap();
        *count += 1;
    }

    /// Get error statistics
    pub fn get_stats(&self) -> (usize, usize) {
        let errors = *self.error_count.lock().unwrap();
        let panics = *self.panic_count.lock().unwrap();
        (errors, panics)
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.error_count.lock().unwrap() = 0;
        *self.panic_count.lock().unwrap() = 0;
    }
}

/// Wrapper for isolated source operations
pub struct IsolatedSource {
    source_id: SourceId,
    boundary: ErrorBoundary,
    quarantined: Arc<Mutex<bool>>,
    failure_count: Arc<Mutex<usize>>,
    max_failures: usize,
}

impl IsolatedSource {
    pub fn new(source_id: SourceId, policy: IsolationPolicy) -> Self {
        Self {
            source_id,
            boundary: ErrorBoundary::new(source_id, policy),
            quarantined: Arc::new(Mutex::new(false)),
            failure_count: Arc::new(Mutex::new(0)),
            max_failures: 10,
        }
    }

    /// Check if source is quarantined
    pub fn is_quarantined(&self) -> bool {
        *self.quarantined.lock().unwrap()
    }

    /// Quarantine the source
    pub fn quarantine(&self, reason: String) {
        *self.quarantined.lock().unwrap() = true;
        log::warn!("Source {} quarantined: {}", self.source_id, reason);
    }

    /// Release from quarantine
    pub fn release_quarantine(&self) {
        *self.quarantined.lock().unwrap() = false;
        *self.failure_count.lock().unwrap() = 0;
        self.boundary.reset_stats();
        log::info!("Source {} released from quarantine", self.source_id);
    }

    /// Execute an operation with automatic quarantine on repeated failures
    pub fn execute_with_quarantine<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        if self.is_quarantined() {
            return Err(DeepStreamError::Unknown(format!(
                "Source {} is quarantined",
                self.source_id
            )));
        }

        let result = self.boundary.execute(f);

        match result {
            IsolationResult::Success(value) => {
                // Reset failure count on success
                *self.failure_count.lock().unwrap() = 0;
                Ok(value)
            }
            IsolationResult::Error(e) => {
                self.handle_failure();
                Err(e)
            }
            IsolationResult::Panic(msg) => {
                self.handle_failure();
                Err(DeepStreamError::Unknown(format!("Panic: {}", msg)))
            }
            IsolationResult::Timeout => {
                self.handle_failure();
                Err(DeepStreamError::Timeout("Operation timed out".to_string()))
            }
        }
    }

    /// Handle a failure and check for quarantine
    fn handle_failure(&self) {
        let mut count = self.failure_count.lock().unwrap();
        *count += 1;

        if *count >= self.max_failures {
            self.quarantine(format!("Exceeded {} failures", self.max_failures));
        }
    }
}

/// Manager for isolated sources
pub struct IsolationManager {
    sources: Arc<Mutex<std::collections::HashMap<SourceId, Arc<IsolatedSource>>>>,
    default_policy: IsolationPolicy,
}

impl IsolationManager {
    pub fn new(default_policy: IsolationPolicy) -> Self {
        Self {
            sources: Arc::new(Mutex::new(std::collections::HashMap::new())),
            default_policy,
        }
    }

    /// Add a source to isolation management
    pub fn add_source(&self, source_id: SourceId) -> Arc<IsolatedSource> {
        let mut sources = self.sources.lock().unwrap();

        sources
            .entry(source_id)
            .or_insert_with(|| Arc::new(IsolatedSource::new(source_id, self.default_policy)))
            .clone()
    }

    /// Remove a source from isolation management
    pub fn remove_source(&self, source_id: SourceId) {
        let mut sources = self.sources.lock().unwrap();
        sources.remove(&source_id);
    }

    /// Get an isolated source
    pub fn get_source(&self, source_id: SourceId) -> Option<Arc<IsolatedSource>> {
        let sources = self.sources.lock().unwrap();
        sources.get(&source_id).cloned()
    }

    /// Get all quarantined sources
    pub fn get_quarantined_sources(&self) -> Vec<SourceId> {
        let sources = self.sources.lock().unwrap();
        sources
            .iter()
            .filter(|(_, source)| source.is_quarantined())
            .map(|(id, _)| *id)
            .collect()
    }

    /// Release all quarantined sources
    pub fn release_all_quarantines(&self) {
        let sources = self.sources.lock().unwrap();
        for source in sources.values() {
            if source.is_quarantined() {
                source.release_quarantine();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_boundary_success() {
        let boundary = ErrorBoundary::new(SourceId(0), IsolationPolicy::Basic);

        let result = boundary.execute(|| Ok(42));

        match result {
            IsolationResult::Success(value) => assert_eq!(value, 42),
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_error_boundary_error() {
        let boundary = ErrorBoundary::new(SourceId(0), IsolationPolicy::Basic);

        let result =
            boundary.execute(|| Err::<i32, _>(DeepStreamError::Unknown("Test error".to_string())));

        match result {
            IsolationResult::Error(_) => {
                let (errors, _) = boundary.get_stats();
                assert_eq!(errors, 1);
            }
            _ => panic!("Expected error"),
        }
    }

    #[test]
    fn test_error_boundary_panic() {
        let boundary = ErrorBoundary::new(SourceId(0), IsolationPolicy::Basic);

        let result = boundary.execute(|| -> Result<i32> {
            panic!("Test panic");
        });

        match result {
            IsolationResult::Panic(msg) => {
                assert!(msg.contains("Test panic"));
                let (_, panics) = boundary.get_stats();
                assert_eq!(panics, 1);
            }
            _ => panic!("Expected panic"),
        }
    }

    #[test]
    fn test_isolated_source_quarantine() {
        let mut source = IsolatedSource::new(SourceId(0), IsolationPolicy::Basic);
        source.max_failures = 2;

        // First failure
        let _ = source.execute_with_quarantine(|| {
            Err::<i32, _>(DeepStreamError::Unknown("Error 1".to_string()))
        });
        assert!(!source.is_quarantined());

        // Second failure - should quarantine
        let _ = source.execute_with_quarantine(|| {
            Err::<i32, _>(DeepStreamError::Unknown("Error 2".to_string()))
        });
        assert!(source.is_quarantined());

        // Further operations should fail
        let result = source.execute_with_quarantine(|| Ok(42));
        assert!(result.is_err());

        // Release quarantine
        source.release_quarantine();
        assert!(!source.is_quarantined());

        // Should work again
        let result = source.execute_with_quarantine(|| Ok(42));
        assert!(result.is_ok());
    }

    #[test]
    fn test_isolation_manager() {
        let manager = IsolationManager::new(IsolationPolicy::Basic);

        // Add sources
        let source1 = manager.add_source(SourceId(1));
        let source2 = manager.add_source(SourceId(2));

        // Quarantine one source
        source1.quarantine("Test quarantine".to_string());

        // Check quarantined sources
        let quarantined = manager.get_quarantined_sources();
        assert_eq!(quarantined.len(), 1);
        assert_eq!(quarantined[0], SourceId(1));

        // Release all
        manager.release_all_quarantines();

        let quarantined = manager.get_quarantined_sources();
        assert_eq!(quarantined.len(), 0);
    }
}
