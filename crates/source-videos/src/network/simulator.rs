use super::{NetworkConditions, NetworkController, NetworkProfile};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use rand::Rng;

/// Configuration for network simulation
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Enable/disable simulation
    pub enabled: bool,
    /// Current network conditions
    pub conditions: NetworkConditions,
    /// Random seed for reproducible tests
    pub seed: Option<u64>,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            conditions: NetworkConditions::default(),
            seed: None,
        }
    }
}

/// Network simulator for testing error recovery
pub struct NetworkSimulator {
    config: Arc<RwLock<SimulationConfig>>,
    last_drop_time: Arc<RwLock<Option<Instant>>>,
}

impl NetworkSimulator {
    /// Create a new network simulator
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(SimulationConfig::default())),
            last_drop_time: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Create with specific configuration
    pub fn with_config(config: SimulationConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            last_drop_time: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Enable simulation
    pub fn enable(&self) {
        if let Ok(mut config) = self.config.write() {
            config.enabled = true;
        }
    }
    
    /// Disable simulation
    pub fn disable(&self) {
        if let Ok(mut config) = self.config.write() {
            config.enabled = false;
        }
    }
    
    /// Check if simulation is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.read().map(|c| c.enabled).unwrap_or(false)
    }
    
    /// Should drop packet based on loss rate
    pub fn should_drop_packet(&self) -> bool {
        let config = match self.config.read() {
            Ok(c) => c,
            Err(_) => return false,
        };
        
        if !config.enabled || config.conditions.packet_loss <= 0.0 {
            return false;
        }
        
        let mut rng = rand::thread_rng();
        rng.random::<f32>() * 100.0 < config.conditions.packet_loss
    }
    
    /// Get delay to add for latency simulation
    pub fn get_latency_delay(&self) -> Duration {
        let config = match self.config.read() {
            Ok(c) => c,
            Err(_) => return Duration::ZERO,
        };
        
        if !config.enabled {
            return Duration::ZERO;
        }
        
        let base_latency = config.conditions.latency_ms;
        let jitter = config.conditions.jitter_ms;
        
        if jitter > 0 {
            let mut rng = rand::thread_rng();
            let variation = rng.random_range(0..=jitter);
            Duration::from_millis((base_latency + variation) as u64)
        } else {
            Duration::from_millis(base_latency as u64)
        }
    }
    
    /// Check if connection should be dropped
    pub fn is_connection_dropped(&self) -> bool {
        self.config.read()
            .map(|c| c.enabled && c.conditions.connection_dropped)
            .unwrap_or(false)
    }
    
    /// Simulate periodic connection drops
    pub fn simulate_periodic_drops(&self, period: Duration, drop_duration: Duration) {
        let now = Instant::now();
        let mut last_drop = self.last_drop_time.write().unwrap();
        
        if let Some(last) = *last_drop {
            if now.duration_since(last) >= period {
                self.drop_connection();
                *last_drop = Some(now);
                
                // Schedule restoration
                let simulator = self.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(drop_duration);
                    simulator.restore_connection();
                });
            }
        } else {
            *last_drop = Some(now);
        }
    }
}

impl NetworkController for NetworkSimulator {
    fn apply_conditions(&self, conditions: NetworkConditions) {
        if let Ok(mut config) = self.config.write() {
            config.conditions = conditions;
            config.enabled = true;
        }
    }
    
    fn get_conditions(&self) -> NetworkConditions {
        self.config.read()
            .map(|c| c.conditions.clone())
            .unwrap_or_default()
    }
    
    fn drop_connection(&self) {
        if let Ok(mut config) = self.config.write() {
            config.conditions.connection_dropped = true;
        }
    }
    
    fn restore_connection(&self) {
        if let Ok(mut config) = self.config.write() {
            config.conditions.connection_dropped = false;
        }
    }
    
    fn apply_profile(&self, profile: NetworkProfile) {
        self.apply_conditions(profile.into_conditions());
    }
    
    fn reset(&self) {
        self.apply_conditions(NetworkConditions::perfect());
        self.disable();
    }
}

impl Clone for NetworkSimulator {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            last_drop_time: Arc::clone(&self.last_drop_time),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simulator_creation() {
        let sim = NetworkSimulator::new();
        assert!(!sim.is_enabled());
        
        sim.enable();
        assert!(sim.is_enabled());
        
        sim.disable();
        assert!(!sim.is_enabled());
    }
    
    #[test]
    fn test_packet_loss() {
        let mut config = SimulationConfig::default();
        config.enabled = true;
        config.conditions.packet_loss = 100.0; // 100% loss
        
        let sim = NetworkSimulator::with_config(config);
        assert!(sim.should_drop_packet());
    }
    
    #[test]
    fn test_latency_simulation() {
        let mut config = SimulationConfig::default();
        config.enabled = true;
        config.conditions.latency_ms = 100;
        
        let sim = NetworkSimulator::with_config(config);
        let delay = sim.get_latency_delay();
        assert_eq!(delay, Duration::from_millis(100));
    }
    
    #[test]
    fn test_connection_control() {
        let sim = NetworkSimulator::new();
        sim.enable();
        
        assert!(!sim.is_connection_dropped());
        
        sim.drop_connection();
        assert!(sim.is_connection_dropped());
        
        sim.restore_connection();
        assert!(!sim.is_connection_dropped());
    }
}
