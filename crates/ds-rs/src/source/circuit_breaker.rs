use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// State of the circuit breaker
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed - normal operation
    Closed,
    /// Circuit is open - blocking all requests
    Open { opened_at: Instant, reason: String },
    /// Circuit is half-open - testing if service recovered
    HalfOpen {
        started_at: Instant,
        test_count: usize,
    },
}

/// Configuration for circuit breaker behavior
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: usize,
    /// Success threshold to close circuit from half-open
    pub success_threshold: usize,
    /// Time window for counting failures
    pub window_duration: Duration,
    /// How long to keep circuit open before testing
    pub open_duration: Duration,
    /// Maximum test requests in half-open state
    pub half_open_max_requests: usize,
    /// Timeout for requests
    pub request_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            window_duration: Duration::from_secs(60),
            open_duration: Duration::from_secs(30),
            half_open_max_requests: 3,
            request_timeout: Duration::from_secs(10),
        }
    }
}

/// Circuit breaker metrics
#[derive(Debug, Default)]
pub struct CircuitMetrics {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub rejected_requests: usize,
    pub circuit_opens: usize,
    pub last_failure_time: Option<Instant>,
    pub last_success_time: Option<Instant>,
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitState>>,
    failure_times: Arc<Mutex<VecDeque<Instant>>>,
    success_count: Arc<Mutex<usize>>,
    metrics: Arc<Mutex<CircuitMetrics>>,
}

impl CircuitBreaker {
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            name,
            config,
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_times: Arc::new(Mutex::new(VecDeque::new())),
            success_count: Arc::new(Mutex::new(0)),
            metrics: Arc::new(Mutex::new(CircuitMetrics::default())),
        }
    }

    /// Check if a request should be allowed
    pub fn should_allow_request(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();

        match &*state {
            CircuitState::Closed => true,
            CircuitState::Open { opened_at, .. } => {
                // Check if it's time to transition to half-open
                if now.duration_since(*opened_at) >= self.config.open_duration {
                    *state = CircuitState::HalfOpen {
                        started_at: now,
                        test_count: 0,
                    };
                    true
                } else {
                    // Still open, reject request
                    let mut metrics = self.metrics.lock().unwrap();
                    metrics.rejected_requests += 1;
                    false
                }
            }
            CircuitState::HalfOpen { test_count, .. } => {
                // Allow limited requests in half-open state
                if *test_count < self.config.half_open_max_requests {
                    true
                } else {
                    let mut metrics = self.metrics.lock().unwrap();
                    metrics.rejected_requests += 1;
                    false
                }
            }
        }
    }

    /// Record a successful request
    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap();
        let mut success_count = self.success_count.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        metrics.total_requests += 1;
        metrics.successful_requests += 1;
        metrics.last_success_time = Some(Instant::now());

        match &*state {
            CircuitState::HalfOpen {
                started_at,
                test_count,
            } => {
                *success_count += 1;

                // Update test count
                let new_test_count = test_count + 1;
                *state = CircuitState::HalfOpen {
                    started_at: *started_at,
                    test_count: new_test_count,
                };

                // Check if we should close the circuit
                if *success_count >= self.config.success_threshold {
                    *state = CircuitState::Closed;
                    *success_count = 0;

                    // Clear failure history
                    let mut failures = self.failure_times.lock().unwrap();
                    failures.clear();

                    log::info!(
                        "Circuit breaker '{}' closed after successful recovery",
                        self.name
                    );
                }
            }
            CircuitState::Closed => {
                // Normal operation, reset success count
                *success_count = 0;
            }
            CircuitState::Open { .. } => {
                // Shouldn't happen, but handle gracefully
                log::warn!(
                    "Success recorded while circuit breaker '{}' is open",
                    self.name
                );
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&self, reason: String) {
        let mut state = self.state.lock().unwrap();
        let mut failures = self.failure_times.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        let now = Instant::now();
        metrics.total_requests += 1;
        metrics.failed_requests += 1;
        metrics.last_failure_time = Some(now);

        // Add failure to history
        failures.push_back(now);

        // Remove old failures outside the window
        let cutoff = now - self.config.window_duration;
        while let Some(front) = failures.front() {
            if *front < cutoff {
                failures.pop_front();
            } else {
                break;
            }
        }

        match &*state {
            CircuitState::Closed => {
                // Check if we should open the circuit
                if failures.len() >= self.config.failure_threshold {
                    *state = CircuitState::Open {
                        opened_at: now,
                        reason: reason.clone(),
                    };
                    metrics.circuit_opens += 1;

                    log::warn!(
                        "Circuit breaker '{}' opened due to {} failures: {}",
                        self.name,
                        failures.len(),
                        reason
                    );
                }
            }
            CircuitState::HalfOpen { .. } => {
                // Failure in half-open state, reopen circuit
                *state = CircuitState::Open {
                    opened_at: now,
                    reason: reason.clone(),
                };
                metrics.circuit_opens += 1;

                // Reset success count
                let mut success_count = self.success_count.lock().unwrap();
                *success_count = 0;

                log::warn!(
                    "Circuit breaker '{}' reopened from half-open due to failure: {}",
                    self.name,
                    reason
                );
            }
            CircuitState::Open { .. } => {
                // Already open, update reason if needed
                *state = CircuitState::Open {
                    opened_at: now,
                    reason,
                };
            }
        }
    }

    /// Get current circuit state
    pub fn get_state(&self) -> CircuitState {
        self.state.lock().unwrap().clone()
    }

    /// Get circuit metrics
    pub fn get_metrics(&self) -> CircuitMetrics {
        let metrics = self.metrics.lock().unwrap();
        CircuitMetrics {
            total_requests: metrics.total_requests,
            successful_requests: metrics.successful_requests,
            failed_requests: metrics.failed_requests,
            rejected_requests: metrics.rejected_requests,
            circuit_opens: metrics.circuit_opens,
            last_failure_time: metrics.last_failure_time,
            last_success_time: metrics.last_success_time,
        }
    }

    /// Reset the circuit breaker
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        *state = CircuitState::Closed;

        let mut failures = self.failure_times.lock().unwrap();
        failures.clear();

        let mut success_count = self.success_count.lock().unwrap();
        *success_count = 0;

        let mut metrics = self.metrics.lock().unwrap();
        *metrics = CircuitMetrics::default();

        log::info!("Circuit breaker '{}' reset", self.name);
    }

    /// Force the circuit to a specific state (for testing/management)
    pub fn force_state(&self, new_state: CircuitState) {
        let mut state = self.state.lock().unwrap();
        *state = new_state;

        log::info!(
            "Circuit breaker '{}' forced to state: {:?}",
            self.name,
            state
        );
    }
}

/// Manages multiple circuit breakers
pub struct CircuitBreakerManager {
    breakers: Arc<Mutex<std::collections::HashMap<String, Arc<CircuitBreaker>>>>,
}

impl CircuitBreakerManager {
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Create or get a circuit breaker
    pub fn get_or_create(&self, name: String, config: CircuitBreakerConfig) -> Arc<CircuitBreaker> {
        let mut breakers = self.breakers.lock().unwrap();

        breakers
            .entry(name.clone())
            .or_insert_with(|| Arc::new(CircuitBreaker::new(name, config)))
            .clone()
    }

    /// Get all circuit breakers
    pub fn get_all(&self) -> Vec<Arc<CircuitBreaker>> {
        let breakers = self.breakers.lock().unwrap();
        breakers.values().cloned().collect()
    }

    /// Reset all circuit breakers
    pub fn reset_all(&self) {
        let breakers = self.breakers.lock().unwrap();
        for breaker in breakers.values() {
            breaker.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_state_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            open_duration: Duration::from_millis(100),
            ..Default::default()
        };

        let breaker = CircuitBreaker::new("test".to_string(), config);

        // Initially closed
        assert_eq!(breaker.get_state(), CircuitState::Closed);
        assert!(breaker.should_allow_request());

        // Record failures to open circuit
        breaker.record_failure("Error 1".to_string());
        assert_eq!(breaker.get_state(), CircuitState::Closed);

        breaker.record_failure("Error 2".to_string());
        assert!(matches!(breaker.get_state(), CircuitState::Open { .. }));
        assert!(!breaker.should_allow_request());

        // Wait for open duration
        std::thread::sleep(Duration::from_millis(150));

        // Should transition to half-open
        assert!(breaker.should_allow_request());
        assert!(matches!(breaker.get_state(), CircuitState::HalfOpen { .. }));

        // Record successes to close circuit
        breaker.record_success();
        breaker.record_success();
        assert_eq!(breaker.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_failure_window() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            window_duration: Duration::from_millis(100),
            ..Default::default()
        };

        let breaker = CircuitBreaker::new("test".to_string(), config);

        // Record old failures
        breaker.record_failure("Old error".to_string());
        breaker.record_failure("Old error".to_string());

        // Wait for window to expire
        std::thread::sleep(Duration::from_millis(150));

        // These failures should not trigger opening
        breaker.record_failure("New error".to_string());
        assert_eq!(breaker.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            open_duration: Duration::from_millis(50),
            ..Default::default()
        };

        let breaker = CircuitBreaker::new("test".to_string(), config);

        // Open the circuit
        breaker.record_failure("Error".to_string());
        assert!(matches!(breaker.get_state(), CircuitState::Open { .. }));

        // Wait and transition to half-open
        std::thread::sleep(Duration::from_millis(100));
        assert!(breaker.should_allow_request());

        // Failure in half-open should reopen
        breaker.record_failure("Error in half-open".to_string());
        assert!(matches!(breaker.get_state(), CircuitState::Open { .. }));
    }

    #[test]
    fn test_metrics_tracking() {
        let breaker = CircuitBreaker::new("test".to_string(), CircuitBreakerConfig::default());

        breaker.record_success();
        breaker.record_success();
        breaker.record_failure("Error".to_string());

        let metrics = breaker.get_metrics();
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 2);
        assert_eq!(metrics.failed_requests, 1);
    }

    #[test]
    fn test_circuit_breaker_manager() {
        let manager = CircuitBreakerManager::new();

        let breaker1 =
            manager.get_or_create("source1".to_string(), CircuitBreakerConfig::default());

        let breaker2 =
            manager.get_or_create("source1".to_string(), CircuitBreakerConfig::default());

        // Should return the same instance
        assert!(Arc::ptr_eq(&breaker1, &breaker2));

        // Create different breaker
        let breaker3 =
            manager.get_or_create("source2".to_string(), CircuitBreakerConfig::default());

        assert!(!Arc::ptr_eq(&breaker1, &breaker3));

        let all = manager.get_all();
        assert_eq!(all.len(), 2);
    }
}
