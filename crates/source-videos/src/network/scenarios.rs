use super::{NetworkConditions, NetworkController};
use std::time::{Duration, Instant};
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

/// A network scenario that changes conditions over time
#[derive(Debug, Clone)]
pub struct NetworkScenario {
    pub name: String,
    pub description: String,
    pub duration: Duration,
    pub events: BTreeMap<Duration, NetworkConditions>,
}

impl NetworkScenario {
    /// Create a new network scenario
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            duration: Duration::from_secs(300), // 5 minutes default
            events: BTreeMap::new(),
        }
    }
    
    /// Add an event at a specific time
    pub fn add_event(mut self, time: Duration, conditions: NetworkConditions) -> Self {
        self.events.insert(time, conditions);
        if time > self.duration {
            self.duration = time;
        }
        self
    }
    
    /// Get conditions at a specific time (interpolated if needed)
    pub fn get_conditions_at(&self, elapsed: Duration) -> NetworkConditions {
        // Find the last event before or at this time
        let mut last_conditions = NetworkConditions::perfect();
        let mut next_conditions = None;
        let mut last_time = Duration::ZERO;
        let mut next_time = Duration::ZERO;
        
        for (&time, conditions) in &self.events {
            if time <= elapsed {
                last_conditions = conditions.clone();
                last_time = time;
            } else if next_conditions.is_none() {
                next_conditions = Some(conditions.clone());
                next_time = time;
                break;
            }
        }
        
        // If we have a next event, interpolate between last and next
        if let Some(next) = next_conditions {
            if next_time > last_time {
                let progress = (elapsed - last_time).as_secs_f32() / (next_time - last_time).as_secs_f32();
                return interpolate_conditions(&last_conditions, &next, progress);
            }
        }
        
        last_conditions
    }
    
    /// Create a degrading network scenario
    pub fn degrading() -> Self {
        Self::new("degrading_network", "Network quality degrades over time")
            .add_event(Duration::ZERO, NetworkConditions {
                packet_loss: 0.0,
                latency_ms: 20,
                bandwidth_kbps: 10000,
                connection_dropped: false,
                jitter_ms: 5,
            })
            .add_event(Duration::from_secs(60), NetworkConditions {
                packet_loss: 1.0,
                latency_ms: 50,
                bandwidth_kbps: 5000,
                connection_dropped: false,
                jitter_ms: 10,
            })
            .add_event(Duration::from_secs(180), NetworkConditions {
                packet_loss: 5.0,
                latency_ms: 200,
                bandwidth_kbps: 1000,
                connection_dropped: false,
                jitter_ms: 50,
            })
            .add_event(Duration::from_secs(240), NetworkConditions {
                packet_loss: 10.0,
                latency_ms: 500,
                bandwidth_kbps: 500,
                connection_dropped: false,
                jitter_ms: 100,
            })
    }
    
    /// Create a flaky network scenario
    pub fn flaky() -> Self {
        Self::new("flaky_network", "Network with periodic issues")
            .add_event(Duration::ZERO, NetworkConditions::perfect())
            .add_event(Duration::from_secs(30), NetworkConditions {
                packet_loss: 20.0,
                latency_ms: 300,
                bandwidth_kbps: 500,
                connection_dropped: false,
                jitter_ms: 100,
            })
            .add_event(Duration::from_secs(45), NetworkConditions::perfect())
            .add_event(Duration::from_secs(90), NetworkConditions {
                packet_loss: 0.0,
                latency_ms: 0,
                bandwidth_kbps: 0,
                connection_dropped: true,
                jitter_ms: 0,
            })
            .add_event(Duration::from_secs(95), NetworkConditions::perfect())
            .add_event(Duration::from_secs(150), NetworkConditions {
                packet_loss: 15.0,
                latency_ms: 200,
                bandwidth_kbps: 1000,
                connection_dropped: false,
                jitter_ms: 80,
            })
            .add_event(Duration::from_secs(180), NetworkConditions::perfect())
    }
    
    /// Create an intermittent satellite scenario with periodic disconnections
    pub fn intermittent_satellite() -> Self {
        Self::new("intermittent_satellite", "Satellite link with periodic signal loss")
            .add_event(Duration::ZERO, NetworkConditions {
                packet_loss: 3.0,
                latency_ms: 750,
                bandwidth_kbps: 5000,
                connection_dropped: false,
                jitter_ms: 200,
            })
            // First disconnection at 30s
            .add_event(Duration::from_secs(30), NetworkConditions {
                packet_loss: 100.0,
                latency_ms: 0,
                bandwidth_kbps: 0,
                connection_dropped: true,
                jitter_ms: 0,
            })
            // Reconnect at 35s
            .add_event(Duration::from_secs(35), NetworkConditions {
                packet_loss: 3.0,
                latency_ms: 750,
                bandwidth_kbps: 5000,
                connection_dropped: false,
                jitter_ms: 200,
            })
            // Second disconnection at 90s
            .add_event(Duration::from_secs(90), NetworkConditions {
                packet_loss: 100.0,
                latency_ms: 0,
                bandwidth_kbps: 0,
                connection_dropped: true,
                jitter_ms: 0,
            })
            // Reconnect at 100s
            .add_event(Duration::from_secs(100), NetworkConditions {
                packet_loss: 3.0,
                latency_ms: 750,
                bandwidth_kbps: 5000,
                connection_dropped: false,
                jitter_ms: 200,
            })
            // Signal degradation at 150s
            .add_event(Duration::from_secs(150), NetworkConditions {
                packet_loss: 20.0,
                latency_ms: 900,
                bandwidth_kbps: 1000,
                connection_dropped: false,
                jitter_ms: 300,
            })
            // Recovery at 180s
            .add_event(Duration::from_secs(180), NetworkConditions {
                packet_loss: 3.0,
                latency_ms: 750,
                bandwidth_kbps: 5000,
                connection_dropped: false,
                jitter_ms: 200,
            })
    }
    
    /// Create a noisy radio link scenario with high interference
    pub fn noisy_radio() -> Self {
        Self::new("noisy_radio", "Radio link with varying interference")
            .add_event(Duration::ZERO, NetworkConditions {
                packet_loss: 5.0,
                latency_ms: 50,
                bandwidth_kbps: 2000,
                connection_dropped: false,
                jitter_ms: 30,
            })
            // High interference period
            .add_event(Duration::from_secs(20), NetworkConditions {
                packet_loss: 25.0,
                latency_ms: 150,
                bandwidth_kbps: 500,
                connection_dropped: false,
                jitter_ms: 200,
            })
            // Moderate interference
            .add_event(Duration::from_secs(60), NetworkConditions {
                packet_loss: 15.0,
                latency_ms: 80,
                bandwidth_kbps: 1000,
                connection_dropped: false,
                jitter_ms: 150,
            })
            // Clear signal
            .add_event(Duration::from_secs(120), NetworkConditions {
                packet_loss: 2.0,
                latency_ms: 40,
                bandwidth_kbps: 3000,
                connection_dropped: false,
                jitter_ms: 20,
            })
            // Interference returns
            .add_event(Duration::from_secs(180), NetworkConditions {
                packet_loss: 20.0,
                latency_ms: 100,
                bandwidth_kbps: 800,
                connection_dropped: false,
                jitter_ms: 180,
            })
    }
    
    /// Create a drone urban flight scenario with building obstructions
    pub fn drone_urban_flight() -> Self {
        Self::new("drone_urban_flight", "Drone flying through urban environment with buildings")
            // Clear line of sight at start
            .add_event(Duration::ZERO, NetworkConditions {
                packet_loss: 2.0,
                latency_ms: 20,
                bandwidth_kbps: 2000,
                connection_dropped: false,
                jitter_ms: 10,
            })
            // Entering urban canyon
            .add_event(Duration::from_secs(10), NetworkConditions {
                packet_loss: 15.0,
                latency_ms: 40,
                bandwidth_kbps: 1200,
                connection_dropped: false,
                jitter_ms: 80,
            })
            // Behind building - severe degradation
            .add_event(Duration::from_secs(20), NetworkConditions {
                packet_loss: 50.0,
                latency_ms: 100,
                bandwidth_kbps: 200,
                connection_dropped: false,
                jitter_ms: 200,
            })
            // Complete signal loss behind large building
            .add_event(Duration::from_secs(25), NetworkConditions {
                packet_loss: 100.0,
                latency_ms: 0,
                bandwidth_kbps: 0,
                connection_dropped: true,
                jitter_ms: 0,
            })
            // Emerging from building shadow
            .add_event(Duration::from_secs(30), NetworkConditions {
                packet_loss: 30.0,
                latency_ms: 60,
                bandwidth_kbps: 500,
                connection_dropped: false,
                jitter_ms: 150,
            })
            // Between buildings - multipath interference
            .add_event(Duration::from_secs(45), NetworkConditions {
                packet_loss: 20.0,
                latency_ms: 40,
                bandwidth_kbps: 800,
                connection_dropped: false,
                jitter_ms: 120,
            })
            // Flying low between buildings - reflections
            .add_event(Duration::from_secs(60), NetworkConditions {
                packet_loss: 25.0,
                latency_ms: 50,
                bandwidth_kbps: 600,
                connection_dropped: false,
                jitter_ms: 180,
            })
            // Gaining altitude - improving signal
            .add_event(Duration::from_secs(90), NetworkConditions {
                packet_loss: 8.0,
                latency_ms: 30,
                bandwidth_kbps: 1500,
                connection_dropped: false,
                jitter_ms: 40,
            })
            // Clear line of sight above buildings
            .add_event(Duration::from_secs(120), NetworkConditions {
                packet_loss: 1.0,
                latency_ms: 15,
                bandwidth_kbps: 2500,
                connection_dropped: false,
                jitter_ms: 5,
            })
            // Descending back into urban area
            .add_event(Duration::from_secs(150), NetworkConditions {
                packet_loss: 20.0,
                latency_ms: 40,
                bandwidth_kbps: 800,
                connection_dropped: false,
                jitter_ms: 120,
            })
    }
    
    /// Create a drone mountain flight scenario with terrain masking
    pub fn drone_mountain_flight() -> Self {
        Self::new("drone_mountain_flight", "Drone flying in mountainous/open terrain")
            // Takeoff - good signal
            .add_event(Duration::ZERO, NetworkConditions {
                packet_loss: 1.0,
                latency_ms: 30,
                bandwidth_kbps: 2000,
                connection_dropped: false,
                jitter_ms: 10,
            })
            // Flying away - distance effects
            .add_event(Duration::from_secs(30), NetworkConditions {
                packet_loss: 3.0,
                latency_ms: 50,
                bandwidth_kbps: 1800,
                connection_dropped: false,
                jitter_ms: 20,
            })
            // Behind first ridge - partial obstruction
            .add_event(Duration::from_secs(60), NetworkConditions {
                packet_loss: 15.0,
                latency_ms: 70,
                bandwidth_kbps: 1000,
                connection_dropped: false,
                jitter_ms: 60,
            })
            // Deep valley - terrain masking
            .add_event(Duration::from_secs(90), NetworkConditions {
                packet_loss: 40.0,
                latency_ms: 100,
                bandwidth_kbps: 300,
                connection_dropped: false,
                jitter_ms: 150,
            })
            // Complete terrain masking
            .add_event(Duration::from_secs(105), NetworkConditions {
                packet_loss: 100.0,
                latency_ms: 0,
                bandwidth_kbps: 0,
                connection_dropped: true,
                jitter_ms: 0,
            })
            // Climbing out of valley
            .add_event(Duration::from_secs(120), NetworkConditions {
                packet_loss: 25.0,
                latency_ms: 80,
                bandwidth_kbps: 800,
                connection_dropped: false,
                jitter_ms: 100,
            })
            // High altitude - good signal but distance effects
            .add_event(Duration::from_secs(150), NetworkConditions {
                packet_loss: 5.0,
                latency_ms: 60,
                bandwidth_kbps: 1500,
                connection_dropped: false,
                jitter_ms: 30,
            })
            // Maximum range - weak signal
            .add_event(Duration::from_secs(180), NetworkConditions {
                packet_loss: 12.0,
                latency_ms: 90,
                bandwidth_kbps: 600,
                connection_dropped: false,
                jitter_ms: 80,
            })
            // Returning - signal improving
            .add_event(Duration::from_secs(240), NetworkConditions {
                packet_loss: 4.0,
                latency_ms: 45,
                bandwidth_kbps: 1700,
                connection_dropped: false,
                jitter_ms: 25,
            })
            // Close range - excellent signal
            .add_event(Duration::from_secs(300), NetworkConditions {
                packet_loss: 0.5,
                latency_ms: 25,
                bandwidth_kbps: 2200,
                connection_dropped: false,
                jitter_ms: 5,
            })
    }
    
    /// Create a congestion scenario
    pub fn congestion() -> Self {
        Self::new("congestion", "Network congestion during peak hours")
            .add_event(Duration::ZERO, NetworkConditions::perfect())
            .add_event(Duration::from_secs(60), NetworkConditions {
                packet_loss: 2.0,
                latency_ms: 100,
                bandwidth_kbps: 3000,
                connection_dropped: false,
                jitter_ms: 30,
            })
            .add_event(Duration::from_secs(180), NetworkConditions {
                packet_loss: 5.0,
                latency_ms: 250,
                bandwidth_kbps: 1000,
                connection_dropped: false,
                jitter_ms: 80,
            })
            .add_event(Duration::from_secs(300), NetworkConditions {
                packet_loss: 3.0,
                latency_ms: 150,
                bandwidth_kbps: 2000,
                connection_dropped: false,
                jitter_ms: 50,
            })
            .add_event(Duration::from_secs(420), NetworkConditions::perfect())
    }
}

/// Interpolate between two network conditions
fn interpolate_conditions(from: &NetworkConditions, to: &NetworkConditions, progress: f32) -> NetworkConditions {
    let progress = progress.clamp(0.0, 1.0);
    
    NetworkConditions {
        packet_loss: from.packet_loss + (to.packet_loss - from.packet_loss) * progress,
        latency_ms: (from.latency_ms as f32 + (to.latency_ms as f32 - from.latency_ms as f32) * progress) as u32,
        bandwidth_kbps: (from.bandwidth_kbps as f32 + (to.bandwidth_kbps as f32 - from.bandwidth_kbps as f32) * progress) as u32,
        jitter_ms: (from.jitter_ms as f32 + (to.jitter_ms as f32 - from.jitter_ms as f32) * progress) as u32,
        connection_dropped: if progress > 0.5 { to.connection_dropped } else { from.connection_dropped },
    }
}

/// A scenario player that executes scenarios over time
pub struct ScenarioPlayer {
    scenario: NetworkScenario,
    start_time: Instant,
    controller: Box<dyn NetworkController>,
}

impl ScenarioPlayer {
    /// Create a new scenario player
    pub fn new(scenario: NetworkScenario, controller: Box<dyn NetworkController>) -> Self {
        Self {
            scenario,
            start_time: Instant::now(),
            controller,
        }
    }
    
    /// Update conditions based on elapsed time
    pub fn update(&self) {
        let elapsed = self.start_time.elapsed();
        if elapsed <= self.scenario.duration {
            let conditions = self.scenario.get_conditions_at(elapsed);
            self.controller.apply_conditions(conditions);
        }
    }
    
    /// Check if scenario is complete
    pub fn is_complete(&self) -> bool {
        self.start_time.elapsed() > self.scenario.duration
    }
    
    /// Reset the scenario
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.controller.reset();
    }
    
    /// Get progress percentage
    pub fn progress(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let total = self.scenario.duration.as_secs_f32();
        (elapsed / total * 100.0).min(100.0)
    }
}

/// Scenario configuration for YAML/JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioConfig {
    pub name: String,
    pub description: String,
    pub duration: String,
    pub events: Vec<ScenarioEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioEvent {
    pub time: String,
    pub conditions: ScenarioConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioConditions {
    #[serde(default)]
    pub packet_loss: f32,
    #[serde(default)]
    pub latency_ms: u32,
    #[serde(default)]
    pub bandwidth_kbps: u32,
    #[serde(default)]
    pub jitter_ms: u32,
    #[serde(default)]
    pub connection_dropped: bool,
}

impl ScenarioConfig {
    /// Convert to NetworkScenario
    pub fn into_scenario(self) -> Result<NetworkScenario, String> {
        let duration = parse_duration(&self.duration)?;
        let mut scenario = NetworkScenario::new(self.name, self.description);
        scenario.duration = duration;
        
        for event in self.events {
            let time = parse_duration(&event.time)?;
            let conditions = NetworkConditions {
                packet_loss: event.conditions.packet_loss,
                latency_ms: event.conditions.latency_ms,
                bandwidth_kbps: event.conditions.bandwidth_kbps,
                jitter_ms: event.conditions.jitter_ms,
                connection_dropped: event.conditions.connection_dropped,
            };
            scenario = scenario.add_event(time, conditions);
        }
        
        Ok(scenario)
    }
}

/// Parse duration string (e.g., "60s", "5m", "1h")
fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty duration string".to_string());
    }
    
    let (num_str, unit) = if s.ends_with("ms") {
        (&s[..s.len()-2], "ms")
    } else if s.ends_with('s') {
        (&s[..s.len()-1], "s")
    } else if s.ends_with('m') {
        (&s[..s.len()-1], "m")
    } else if s.ends_with('h') {
        (&s[..s.len()-1], "h")
    } else {
        return Err(format!("Invalid duration format: {}", s));
    };
    
    let num: u64 = num_str.parse()
        .map_err(|_| format!("Invalid number in duration: {}", num_str))?;
    
    Ok(match unit {
        "ms" => Duration::from_millis(num),
        "s" => Duration::from_secs(num),
        "m" => Duration::from_secs(num * 60),
        "h" => Duration::from_secs(num * 3600),
        _ => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scenario_creation() {
        let scenario = NetworkScenario::degrading();
        assert_eq!(scenario.events.len(), 4);
        
        // Check conditions at various times
        let conditions_start = scenario.get_conditions_at(Duration::ZERO);
        assert_eq!(conditions_start.packet_loss, 0.0);
        
        let conditions_end = scenario.get_conditions_at(Duration::from_secs(240));
        assert_eq!(conditions_end.packet_loss, 10.0);
    }
    
    #[test]
    fn test_interpolation() {
        let from = NetworkConditions {
            packet_loss: 0.0,
            latency_ms: 0,
            bandwidth_kbps: 10000,
            jitter_ms: 0,
            connection_dropped: false,
        };
        
        let to = NetworkConditions {
            packet_loss: 10.0,
            latency_ms: 100,
            bandwidth_kbps: 1000,
            jitter_ms: 50,
            connection_dropped: false,
        };
        
        let mid = interpolate_conditions(&from, &to, 0.5);
        assert_eq!(mid.packet_loss, 5.0);
        assert_eq!(mid.latency_ms, 50);
        assert_eq!(mid.bandwidth_kbps, 5500);
    }
    
    #[test]
    fn test_duration_parsing() {
        assert_eq!(parse_duration("60s").unwrap(), Duration::from_secs(60));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));
    }
}