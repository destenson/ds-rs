use super::NetworkConditions;

/// Predefined network profiles for common conditions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkProfile {
    /// Perfect network conditions
    Perfect,
    /// 3G mobile network
    Mobile3G,
    /// 4G LTE network
    Mobile4G,
    /// 5G network
    Mobile5G,
    /// Home WiFi network
    WiFiHome,
    /// Public WiFi (congested)
    WiFiPublic,
    /// Satellite internet
    Satellite,
    /// Cable/DSL broadband
    Broadband,
    /// Poor network conditions
    Poor,
    /// Noisy radio link (high packet loss, variable latency)
    NoisyRadio,
    /// Intermittent satellite (periodic disconnections)
    IntermittentSatellite,
    /// Drone UHF/VHF link through urban environment
    DroneUrban,
    /// Drone in open/mountain terrain (long range, terrain masking)
    DroneMountain,
    /// Custom profile
    Custom,
}

impl NetworkProfile {
    /// Convert profile to network conditions
    pub fn into_conditions(self) -> NetworkConditions {
        match self {
            NetworkProfile::Perfect => NetworkConditions {
                packet_loss: 0.0,
                latency_ms: 0,
                bandwidth_kbps: 0, // unlimited
                connection_dropped: false,
                jitter_ms: 0,
                duplicate_probability: 0.0,
                allow_reordering: false,
                min_delay_ms: 0,
                max_delay_ms: 0,
                delay_probability: 0.0,
            },

            NetworkProfile::Mobile3G => NetworkConditions {
                packet_loss: 2.0,
                latency_ms: 150,
                bandwidth_kbps: 2000, // 2 Mbps
                connection_dropped: false,
                jitter_ms: 30,
                duplicate_probability: 0.5,
                allow_reordering: true,
                min_delay_ms: 120,
                max_delay_ms: 180,
                delay_probability: 100.0,
            },

            NetworkProfile::Mobile4G => NetworkConditions {
                packet_loss: 0.5,
                latency_ms: 50,
                bandwidth_kbps: 10000, // 10 Mbps
                connection_dropped: false,
                jitter_ms: 10,
                duplicate_probability: 0.2,
                allow_reordering: true,
                min_delay_ms: 40,
                max_delay_ms: 60,
                delay_probability: 100.0,
            },

            NetworkProfile::Mobile5G => NetworkConditions {
                packet_loss: 0.1,
                latency_ms: 10,
                bandwidth_kbps: 100000, // 100 Mbps
                connection_dropped: false,
                jitter_ms: 2,
                duplicate_probability: 0.05,
                allow_reordering: false,
                min_delay_ms: 8,
                max_delay_ms: 12,
                delay_probability: 100.0,
            },

            NetworkProfile::WiFiHome => NetworkConditions {
                packet_loss: 0.2,
                latency_ms: 5,
                bandwidth_kbps: 50000, // 50 Mbps
                connection_dropped: false,
                jitter_ms: 2,
                duplicate_probability: 0.1,
                allow_reordering: false,
                min_delay_ms: 3,
                max_delay_ms: 7,
                delay_probability: 100.0,
            },

            NetworkProfile::WiFiPublic => NetworkConditions {
                packet_loss: 3.0,
                latency_ms: 100,
                bandwidth_kbps: 5000, // 5 Mbps
                connection_dropped: false,
                jitter_ms: 50,
                duplicate_probability: 1.0,
                allow_reordering: true,
                min_delay_ms: 50,
                max_delay_ms: 150,
                delay_probability: 100.0,
            },

            NetworkProfile::Satellite => NetworkConditions {
                packet_loss: 1.0,
                latency_ms: 600,
                bandwidth_kbps: 25000, // 25 Mbps
                connection_dropped: false,
                jitter_ms: 100,
                duplicate_probability: 0.2,
                allow_reordering: false, // Satellite links maintain order
                min_delay_ms: 550,
                max_delay_ms: 700,
                delay_probability: 100.0,
            },

            NetworkProfile::Broadband => NetworkConditions {
                packet_loss: 0.1,
                latency_ms: 20,
                bandwidth_kbps: 100000, // 100 Mbps
                connection_dropped: false,
                jitter_ms: 5,
                duplicate_probability: 0.05,
                allow_reordering: false,
                min_delay_ms: 15,
                max_delay_ms: 25,
                delay_probability: 100.0,
            },

            NetworkProfile::Poor => NetworkConditions {
                packet_loss: 10.0,
                latency_ms: 500,
                bandwidth_kbps: 500, // 500 kbps
                connection_dropped: false,
                jitter_ms: 200,
                duplicate_probability: 2.0,
                allow_reordering: true,
                min_delay_ms: 300,
                max_delay_ms: 700,
                delay_probability: 100.0,
            },

            NetworkProfile::NoisyRadio => NetworkConditions {
                packet_loss: 15.0,    // High packet loss due to interference
                latency_ms: 80,       // Moderate latency
                bandwidth_kbps: 1000, // 1 Mbps limited bandwidth
                connection_dropped: false,
                jitter_ms: 150,             // High jitter from signal variations
                duplicate_probability: 3.0, // Radio interference can cause duplicates
                allow_reordering: true,     // Signal reflections cause reordering
                min_delay_ms: 20,
                max_delay_ms: 230,
                delay_probability: 100.0,
            },

            NetworkProfile::IntermittentSatellite => NetworkConditions {
                packet_loss: 3.0,          // Some packet loss
                latency_ms: 750,           // Very high latency
                bandwidth_kbps: 5000,      // 5 Mbps when connected
                connection_dropped: false, // Will be toggled periodically
                jitter_ms: 200,            // High jitter from atmospheric conditions
                duplicate_probability: 0.5,
                allow_reordering: false,
                min_delay_ms: 650,
                max_delay_ms: 950,
                delay_probability: 100.0,
            },

            NetworkProfile::DroneUrban => NetworkConditions {
                packet_loss: 20.0,   // High loss from building obstruction
                latency_ms: 40,      // Low latency when signal gets through
                bandwidth_kbps: 800, // Limited bandwidth on UHF/VHF
                connection_dropped: false,
                jitter_ms: 120,             // Variable due to multipath reflections
                duplicate_probability: 5.0, // Multipath reflections cause duplicates
                allow_reordering: true,     // Building reflections cause severe reordering
                min_delay_ms: 10,
                max_delay_ms: 160,
                delay_probability: 100.0,
            },

            NetworkProfile::DroneMountain => NetworkConditions {
                packet_loss: 5.0,     // Lower loss in open terrain
                latency_ms: 60,       // Slightly higher from distance
                bandwidth_kbps: 1500, // Better bandwidth in clear air
                connection_dropped: false,
                jitter_ms: 30, // More stable than urban
                duplicate_probability: 1.0,
                allow_reordering: true,
                min_delay_ms: 45,
                max_delay_ms: 90,
                delay_probability: 100.0,
            },

            NetworkProfile::Custom => NetworkConditions::default(),
        }
    }

    /// Get a description of the profile
    pub fn description(&self) -> &'static str {
        match self {
            NetworkProfile::Perfect => "Perfect network with no issues",
            NetworkProfile::Mobile3G => "3G mobile network (2 Mbps, 150ms latency)",
            NetworkProfile::Mobile4G => "4G LTE network (10 Mbps, 50ms latency)",
            NetworkProfile::Mobile5G => "5G network (100 Mbps, 10ms latency)",
            NetworkProfile::WiFiHome => "Home WiFi network (50 Mbps, 5ms latency)",
            NetworkProfile::WiFiPublic => "Congested public WiFi (5 Mbps, 100ms latency)",
            NetworkProfile::Satellite => "Satellite internet (25 Mbps, 600ms latency)",
            NetworkProfile::Broadband => "Cable/DSL broadband (100 Mbps, 20ms latency)",
            NetworkProfile::Poor => "Poor network conditions (500 kbps, 500ms latency)",
            NetworkProfile::NoisyRadio => "Noisy radio link (15% loss, high jitter, 1 Mbps)",
            NetworkProfile::IntermittentSatellite => {
                "Intermittent satellite (750ms latency, periodic drops)"
            }
            NetworkProfile::DroneUrban => {
                "Drone UHF/VHF through buildings (20% loss, multipath, 800 kbps)"
            }
            NetworkProfile::DroneMountain => {
                "Drone in mountain terrain (5% loss, distance effects, 1.5 Mbps)"
            }
            NetworkProfile::Custom => "Custom network profile",
        }
    }

    /// Get all available profiles
    pub fn all() -> Vec<NetworkProfile> {
        vec![
            NetworkProfile::Perfect,
            NetworkProfile::Mobile3G,
            NetworkProfile::Mobile4G,
            NetworkProfile::Mobile5G,
            NetworkProfile::WiFiHome,
            NetworkProfile::WiFiPublic,
            NetworkProfile::Satellite,
            NetworkProfile::Broadband,
            NetworkProfile::Poor,
            NetworkProfile::NoisyRadio,
            NetworkProfile::IntermittentSatellite,
            NetworkProfile::DroneUrban,
            NetworkProfile::DroneMountain,
        ]
    }
}

/// Standard profiles for quick access
pub struct StandardProfiles;

impl StandardProfiles {
    /// Get profile for testing error recovery
    pub fn for_error_recovery() -> NetworkProfile {
        NetworkProfile::Poor
    }

    /// Get profile for testing reconnection
    pub fn for_reconnection_test() -> NetworkConditions {
        let mut conditions = NetworkProfile::Mobile4G.into_conditions();
        conditions.connection_dropped = true;
        conditions
    }

    /// Get profile for testing buffering
    pub fn for_buffer_test() -> NetworkProfile {
        NetworkProfile::Mobile3G
    }

    /// Get profile for testing high latency
    pub fn for_latency_test() -> NetworkProfile {
        NetworkProfile::Satellite
    }

    /// Get profile for testing noisy/unreliable connections
    pub fn for_reliability_test() -> NetworkProfile {
        NetworkProfile::NoisyRadio
    }

    /// Get profile for testing intermittent connections
    pub fn for_intermittent_test() -> NetworkProfile {
        NetworkProfile::IntermittentSatellite
    }

    /// Get profile for testing urban drone/UAV communications
    pub fn for_drone_test() -> NetworkProfile {
        NetworkProfile::DroneUrban
    }

    /// Get profile for testing multipath and obstruction effects
    pub fn for_obstruction_test() -> NetworkProfile {
        NetworkProfile::DroneUrban
    }
}

impl std::fmt::Display for NetworkProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for NetworkProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "perfect" => Ok(NetworkProfile::Perfect),
            "3g" | "mobile3g" => Ok(NetworkProfile::Mobile3G),
            "4g" | "mobile4g" | "lte" => Ok(NetworkProfile::Mobile4G),
            "5g" | "mobile5g" => Ok(NetworkProfile::Mobile5G),
            "wifi" | "wifihome" | "home" => Ok(NetworkProfile::WiFiHome),
            "public" | "wifipublic" => Ok(NetworkProfile::WiFiPublic),
            "satellite" | "sat" => Ok(NetworkProfile::Satellite),
            "broadband" | "cable" | "dsl" => Ok(NetworkProfile::Broadband),
            "poor" | "bad" => Ok(NetworkProfile::Poor),
            "noisy" | "noisyradio" | "radio" => Ok(NetworkProfile::NoisyRadio),
            "intermittent" | "intermittentsatellite" | "intermittent-satellite" => {
                Ok(NetworkProfile::IntermittentSatellite)
            }
            "drone" | "droneurban" | "drone-urban" | "uhf" | "vhf" => {
                Ok(NetworkProfile::DroneUrban)
            }
            "mountain" | "dronemountain" | "drone-mountain" | "open-terrain" => {
                Ok(NetworkProfile::DroneMountain)
            }
            "custom" => Ok(NetworkProfile::Custom),
            _ => Err(format!("Unknown network profile: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_conditions() {
        let perfect = NetworkProfile::Perfect.into_conditions();
        assert_eq!(perfect.packet_loss, 0.0);
        assert_eq!(perfect.latency_ms, 0);

        let mobile3g = NetworkProfile::Mobile3G.into_conditions();
        assert!(mobile3g.packet_loss > 0.0);
        assert!(mobile3g.latency_ms > 100);
        assert!(mobile3g.bandwidth_kbps > 0);

        let satellite = NetworkProfile::Satellite.into_conditions();
        assert!(satellite.latency_ms > 500);
    }

    #[test]
    fn test_profile_parsing() {
        assert_eq!(
            "perfect".parse::<NetworkProfile>().unwrap(),
            NetworkProfile::Perfect
        );
        assert_eq!(
            "3g".parse::<NetworkProfile>().unwrap(),
            NetworkProfile::Mobile3G
        );
        assert_eq!(
            "lte".parse::<NetworkProfile>().unwrap(),
            NetworkProfile::Mobile4G
        );
        assert_eq!(
            "satellite".parse::<NetworkProfile>().unwrap(),
            NetworkProfile::Satellite
        );

        assert!("invalid".parse::<NetworkProfile>().is_err());
    }

    #[test]
    fn test_standard_profiles() {
        let recovery = StandardProfiles::for_error_recovery();
        assert_eq!(recovery, NetworkProfile::Poor);

        let reconnect = StandardProfiles::for_reconnection_test();
        assert!(reconnect.connection_dropped);

        let buffer = StandardProfiles::for_buffer_test();
        assert_eq!(buffer, NetworkProfile::Mobile3G);
    }
}
