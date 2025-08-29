pub mod gstreamer;
pub mod profiles;
pub mod scenarios;
pub mod simulator;

use std::time::Duration;

pub use gstreamer::GStreamerNetworkSimulator;
pub use profiles::{NetworkProfile, StandardProfiles};
pub use scenarios::{NetworkScenario, ScenarioConfig, ScenarioPlayer};
pub use simulator::{NetworkSimulator, SimulationConfig};

/// Network conditions to simulate
#[derive(Debug, Clone)]
pub struct NetworkConditions {
    /// Packet loss percentage (0-100)
    pub packet_loss: f32,
    /// Additional latency in milliseconds
    pub latency_ms: u32,
    /// Bandwidth limit in kbps (0 = unlimited)
    pub bandwidth_kbps: u32,
    /// Whether connection is interrupted
    pub connection_dropped: bool,
    /// Jitter in milliseconds
    pub jitter_ms: u32,
    /// Packet duplication percentage (0-100)
    pub duplicate_probability: f32,
    /// Allow packet reordering when delaying
    pub allow_reordering: bool,
    /// Minimum delay in milliseconds (for delay distribution)
    pub min_delay_ms: u32,
    /// Maximum delay in milliseconds (for delay distribution)
    pub max_delay_ms: u32,
    /// Delay probability (0-100)
    pub delay_probability: f32,
}

impl Default for NetworkConditions {
    fn default() -> Self {
        Self {
            packet_loss: 0.0,
            latency_ms: 0,
            bandwidth_kbps: 0,
            connection_dropped: false,
            jitter_ms: 0,
            duplicate_probability: 0.0,
            allow_reordering: true,
            min_delay_ms: 0,
            max_delay_ms: 0,
            delay_probability: 0.0,
        }
    }
}

impl NetworkConditions {
    /// Create perfect network conditions
    pub fn perfect() -> Self {
        Self::default()
    }

    /// Create conditions that will trigger error recovery
    pub fn problematic() -> Self {
        Self {
            packet_loss: 10.0,
            latency_ms: 500,
            bandwidth_kbps: 1000,
            connection_dropped: false,
            jitter_ms: 100,
            duplicate_probability: 2.0,
            allow_reordering: true,
            min_delay_ms: 200,
            max_delay_ms: 800,
            delay_probability: 5.0,
        }
    }

    /// Create custom network conditions
    pub fn custom(packet_loss: f32, latency_ms: u32, bandwidth_kbps: u32, jitter_ms: u32) -> Self {
        Self {
            packet_loss,
            latency_ms,
            bandwidth_kbps,
            connection_dropped: false,
            jitter_ms,
            duplicate_probability: 0.0,
            allow_reordering: true,
            min_delay_ms: latency_ms,
            max_delay_ms: latency_ms + jitter_ms,
            delay_probability: if latency_ms > 0 { 100.0 } else { 0.0 },
        }
    }

    /// Simulate complete connection loss
    pub fn disconnected() -> Self {
        Self {
            connection_dropped: true,
            ..Default::default()
        }
    }
}

/// Events that can occur during network simulation
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// Connection was dropped
    ConnectionDropped,
    /// Connection was restored
    ConnectionRestored,
    /// Packet loss threshold exceeded
    HighPacketLoss(f32),
    /// Bandwidth throttled
    BandwidthThrottled(u32),
    /// Latency spike detected
    LatencySpike(u32),
}

/// Control interface for network simulation
pub trait NetworkController: Send + Sync {
    /// Apply network conditions
    fn apply_conditions(&self, conditions: NetworkConditions);

    /// Get current conditions
    fn get_conditions(&self) -> NetworkConditions;

    /// Simulate connection drop
    fn drop_connection(&self);

    /// Restore connection
    fn restore_connection(&self);

    /// Apply a predefined profile
    fn apply_profile(&self, profile: NetworkProfile);

    /// Reset to perfect conditions
    fn reset(&self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_conditions() {
        let perfect = NetworkConditions::perfect();
        assert_eq!(perfect.packet_loss, 0.0);
        assert!(!perfect.connection_dropped);

        let problematic = NetworkConditions::problematic();
        assert!(problematic.packet_loss > 0.0);
        assert!(problematic.latency_ms > 0);

        let disconnected = NetworkConditions::disconnected();
        assert!(disconnected.connection_dropped);
    }
}
