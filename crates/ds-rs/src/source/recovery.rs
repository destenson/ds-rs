use std::time::Duration;
use std::sync::{Arc, Mutex};
use rand::{thread_rng, Rng};

/// Configuration for recovery behavior
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier (typically 2.0)
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0 to 1.0)
    pub jitter_factor: f64,
    /// Enable health monitoring
    pub health_monitoring_enabled: bool,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Failure threshold before circuit opens
    pub circuit_breaker_threshold: usize,
    /// Half-open test interval for circuit breaker
    pub half_open_interval: Duration,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter_factor: 0.3,
            health_monitoring_enabled: true,
            health_check_interval: Duration::from_secs(10),
            circuit_breaker_threshold: 5,
            half_open_interval: Duration::from_secs(30),
        }
    }
}

/// Represents the current state of recovery attempts
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryState {
    /// No recovery in progress
    Idle,
    /// Currently attempting recovery
    Retrying {
        attempt: usize,
        next_retry: std::time::Instant,
    },
    /// Recovery failed after all attempts
    Failed {
        attempts: usize,
        last_error: String,
    },
    /// Successfully recovered
    Recovered {
        attempts: usize,
    },
}

/// Tracks recovery statistics
#[derive(Debug, Default)]
pub struct RecoveryStats {
    pub total_attempts: usize,
    pub successful_recoveries: usize,
    pub failed_recoveries: usize,
    pub current_streak: usize,
    pub longest_streak: usize,
    pub last_recovery_time: Option<std::time::Instant>,
    pub total_downtime: Duration,
}

/// Manages recovery state and calculates backoff
pub struct RecoveryManager {
    config: RecoveryConfig,
    state: Arc<Mutex<RecoveryState>>,
    stats: Arc<Mutex<RecoveryStats>>,
}

impl RecoveryManager {
    pub fn new(config: RecoveryConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(RecoveryState::Idle)),
            stats: Arc::new(Mutex::new(RecoveryStats::default())),
        }
    }

    /// Calculate the next backoff duration with jitter
    pub fn calculate_backoff(&self, attempt: usize) -> Duration {
        // Calculate exponential backoff
        let base_backoff = self.config.initial_backoff.as_secs_f64()
            * self.config.backoff_multiplier.powi(attempt as i32);
        
        // Cap at maximum backoff
        let capped_backoff = base_backoff.min(self.config.max_backoff.as_secs_f64());
        
        // Apply jitter
        let mut rng = thread_rng();
        let jitter_range = capped_backoff * self.config.jitter_factor;
        let jitter = rng.gen_range(-jitter_range..=jitter_range);
        let final_backoff = (capped_backoff + jitter).max(0.0);
        
        Duration::from_secs_f64(final_backoff)
    }

    /// Start a recovery attempt
    pub fn start_recovery(&self) -> Option<Duration> {
        let mut state = self.state.lock().unwrap();
        
        match &*state {
            RecoveryState::Failed { attempts, .. } if *attempts >= self.config.max_retries => {
                // Already exceeded max retries
                None
            }
            RecoveryState::Retrying { attempt, .. } if *attempt >= self.config.max_retries => {
                // Already at max retries
                None
            }
            _ => {
                let attempt = match &*state {
                    RecoveryState::Retrying { attempt, .. } => attempt + 1,
                    RecoveryState::Failed { attempts, .. } => *attempts,
                    _ => 0,
                };
                
                if attempt >= self.config.max_retries {
                    *state = RecoveryState::Failed {
                        attempts: attempt,
                        last_error: "Max retries exceeded".to_string(),
                    };
                    None
                } else {
                    let backoff = self.calculate_backoff(attempt);
                    let next_retry = std::time::Instant::now() + backoff;
                    
                    *state = RecoveryState::Retrying {
                        attempt,
                        next_retry,
                    };
                    
                    // Update stats
                    let mut stats = self.stats.lock().unwrap();
                    stats.total_attempts += 1;
                    
                    Some(backoff)
                }
            }
        }
    }

    /// Mark a recovery attempt as successful
    pub fn mark_recovered(&self) {
        let mut state = self.state.lock().unwrap();
        let attempts = match &*state {
            RecoveryState::Retrying { attempt, .. } => *attempt + 1,
            _ => 1,
        };
        
        *state = RecoveryState::Recovered { attempts };
        
        // Update stats
        let mut stats = self.stats.lock().unwrap();
        stats.successful_recoveries += 1;
        stats.current_streak += 1;
        if stats.current_streak > stats.longest_streak {
            stats.longest_streak = stats.current_streak;
        }
        stats.last_recovery_time = Some(std::time::Instant::now());
    }

    /// Mark a recovery attempt as failed
    pub fn mark_failed(&self, error: String) {
        let mut state = self.state.lock().unwrap();
        let attempts = match &*state {
            RecoveryState::Retrying { attempt, .. } => *attempt + 1,
            RecoveryState::Failed { attempts, .. } => *attempts,
            _ => 1,
        };
        
        *state = RecoveryState::Failed {
            attempts,
            last_error: error,
        };
        
        // Update stats
        let mut stats = self.stats.lock().unwrap();
        stats.failed_recoveries += 1;
        stats.current_streak = 0;
    }

    /// Reset recovery state
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        *state = RecoveryState::Idle;
    }

    /// Get current recovery state
    pub fn get_state(&self) -> RecoveryState {
        self.state.lock().unwrap().clone()
    }

    /// Get recovery statistics
    pub fn get_stats(&self) -> RecoveryStats {
        let stats = self.stats.lock().unwrap();
        RecoveryStats {
            total_attempts: stats.total_attempts,
            successful_recoveries: stats.successful_recoveries,
            failed_recoveries: stats.failed_recoveries,
            current_streak: stats.current_streak,
            longest_streak: stats.longest_streak,
            last_recovery_time: stats.last_recovery_time,
            total_downtime: stats.total_downtime,
        }
    }

    /// Check if recovery should be attempted
    pub fn should_retry(&self) -> bool {
        let state = self.state.lock().unwrap();
        match &*state {
            RecoveryState::Failed { attempts, .. } => *attempts < self.config.max_retries,
            RecoveryState::Retrying { attempt, next_retry } => {
                *attempt < self.config.max_retries && std::time::Instant::now() >= *next_retry
            }
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let config = RecoveryConfig {
            initial_backoff: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_secs(16),
            jitter_factor: 0.0, // No jitter for predictable testing
            ..Default::default()
        };
        
        let manager = RecoveryManager::new(config);
        
        // Test exponential growth
        assert_eq!(manager.calculate_backoff(0), Duration::from_secs(1));
        assert_eq!(manager.calculate_backoff(1), Duration::from_secs(2));
        assert_eq!(manager.calculate_backoff(2), Duration::from_secs(4));
        assert_eq!(manager.calculate_backoff(3), Duration::from_secs(8));
        assert_eq!(manager.calculate_backoff(4), Duration::from_secs(16));
        assert_eq!(manager.calculate_backoff(5), Duration::from_secs(16)); // Capped at max
    }

    #[test]
    fn test_jitter_application() {
        let config = RecoveryConfig {
            initial_backoff: Duration::from_secs(10),
            jitter_factor: 0.3,
            ..Default::default()
        };
        
        let manager = RecoveryManager::new(config);
        
        // Test that jitter produces different values
        let backoff1 = manager.calculate_backoff(0);
        let backoff2 = manager.calculate_backoff(0);
        
        // With jitter, values should be within expected range
        let min_expected = Duration::from_secs_f64(7.0); // 10 - 30%
        let max_expected = Duration::from_secs_f64(13.0); // 10 + 30%
        
        assert!(backoff1 >= min_expected && backoff1 <= max_expected);
        assert!(backoff2 >= min_expected && backoff2 <= max_expected);
    }

    #[test]
    fn test_recovery_state_transitions() {
        let manager = RecoveryManager::new(RecoveryConfig::default());
        
        // Initial state should be Idle
        assert_eq!(manager.get_state(), RecoveryState::Idle);
        
        // Start recovery
        let backoff = manager.start_recovery();
        assert!(backoff.is_some());
        assert!(matches!(manager.get_state(), RecoveryState::Retrying { .. }));
        
        // Mark as recovered
        manager.mark_recovered();
        assert!(matches!(manager.get_state(), RecoveryState::Recovered { .. }));
        
        // Reset and mark as failed
        manager.reset();
        manager.mark_failed("Test error".to_string());
        assert!(matches!(manager.get_state(), RecoveryState::Failed { .. }));
    }

    #[test]
    fn test_max_retries_enforcement() {
        let config = RecoveryConfig {
            max_retries: 2,
            ..Default::default()
        };
        
        let manager = RecoveryManager::new(config);
        
        // First retry
        assert!(manager.start_recovery().is_some());
        manager.mark_failed("Error 1".to_string());
        
        // Second retry (last allowed)
        assert!(manager.start_recovery().is_some());
        manager.mark_failed("Error 2".to_string());
        
        // Third retry should fail (exceeded max)
        assert!(manager.start_recovery().is_none());
        assert!(!manager.should_retry());
    }

    #[test]
    fn test_recovery_statistics() {
        let manager = RecoveryManager::new(RecoveryConfig::default());
        
        // Successful recovery
        manager.start_recovery();
        manager.mark_recovered();
        
        // Failed recovery
        manager.start_recovery();
        manager.mark_failed("Test failure".to_string());
        
        let stats = manager.get_stats();
        assert_eq!(stats.total_attempts, 2);
        assert_eq!(stats.successful_recoveries, 1);
        assert_eq!(stats.failed_recoveries, 1);
        assert_eq!(stats.current_streak, 0);
    }
}
