use super::{NetworkConditions, NetworkController, NetworkProfile, NetworkSimulator};
use anyhow::{Context, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, RwLock};

/// GStreamer-based network simulator
pub struct GStreamerNetworkSimulator {
    simulator: NetworkSimulator,
    elements: Arc<RwLock<SimulationElements>>,
}

/// GStreamer elements used for network simulation
struct SimulationElements {
    /// netsim element for comprehensive network simulation
    netsim: Option<gst::Element>,
    /// Valve element for connection control (backup if netsim fails)
    valve: Option<gst::Element>,
    /// Using netsim or fallback mode
    using_netsim: bool,
    /// Legacy elements for fallback mode
    queue: Option<gst::Element>,
    identity: Option<gst::Element>,
}

impl GStreamerNetworkSimulator {
    /// Create a new GStreamer network simulator
    pub fn new() -> Self {
        Self {
            simulator: NetworkSimulator::new(),
            elements: Arc::new(RwLock::new(SimulationElements {
                netsim: None,
                valve: None,
                using_netsim: false,
                queue: None,
                identity: None,
            })),
        }
    }

    /// Create simulation elements for a pipeline
    pub fn create_elements(&self, name_prefix: &str) -> Result<gst::Bin> {
        // Create a bin to contain all simulation elements
        let bin = gst::Bin::with_name(&format!("{}_network_sim", name_prefix));

        // Try to create netsim element first (requires gst-plugins-bad)
        let netsim_result = gst::ElementFactory::make("netsim")
            .name(&format!("{}_sim_netsim", name_prefix))
            .property("drop-probability", 0.0f32)
            .property("duplicate-probability", 0.0f32)
            .property("delay-probability", 0.0f32)
            .property("allow-reordering", true)
            .property("min-delay", 0i32)
            .property("max-delay", 0i32)
            .property("max-kbps", -1i32) // unlimited
            .property("max-bucket-size", -1i32) // unlimited
            .build();

        // Create valve for connection control (always needed)
        let valve = gst::ElementFactory::make("valve")
            .name(&format!("{}_sim_valve", name_prefix))
            .property("drop", false)
            .build()
            .context("Failed to create valve element")?;

        let (first_element, using_netsim) = if let Ok(netsim) = netsim_result {
            // Success: Use netsim for comprehensive simulation
            bin.add_many(&[&netsim, &valve])?;
            gst::Element::link_many(&[&netsim, &valve])?;

            // Store netsim reference
            if let Ok(mut elements) = self.elements.write() {
                elements.netsim = Some(netsim.clone());
                elements.valve = Some(valve.clone());
                elements.using_netsim = true;
            }

            (netsim, true)
        } else {
            // Fallback: Use queue + identity + valve (old implementation)
            eprintln!("Warning: netsim element not available, falling back to basic simulation");

            let queue = gst::ElementFactory::make("queue")
                .name(&format!("{}_sim_queue", name_prefix))
                .property("max-size-buffers", 1000u32)
                .property("max-size-bytes", 0u32)
                .property("max-size-time", 0u64)
                .property_from_str("leaky", "downstream")
                .build()
                .context("Failed to create queue element")?;

            let identity = gst::ElementFactory::make("identity")
                .name(&format!("{}_sim_identity", name_prefix))
                .property("drop-probability", 0.0f32)
                .property("sync", true)
                .build()
                .context("Failed to create identity element")?;

            bin.add_many(&[&queue, &identity, &valve])?;
            gst::Element::link_many(&[&queue, &identity, &valve])?;

            // Store fallback element references
            if let Ok(mut elements) = self.elements.write() {
                elements.queue = Some(queue.clone());
                elements.identity = Some(identity.clone());
                elements.valve = Some(valve.clone());
                elements.using_netsim = false;
            }

            (queue, false)
        };

        // Create ghost pads for the bin
        let sink_pad = first_element
            .static_pad("sink")
            .context("Failed to get sink pad")?;
        let ghost_sink = gst::GhostPad::builder_with_target(&sink_pad)?
            .name("sink")
            .build();
        bin.add_pad(&ghost_sink)?;

        let src_pad = valve
            .static_pad("src")
            .context("Failed to get valve src pad")?;
        let ghost_src = gst::GhostPad::builder_with_target(&src_pad)?
            .name("src")
            .build();
        bin.add_pad(&ghost_src)?;

        Ok(bin)
    }

    /// Insert simulation elements into an existing pipeline
    pub fn insert_into_pipeline(
        &self,
        pipeline: &gst::Pipeline,
        before_element: &gst::Element,
        after_element: &gst::Element,
        name_prefix: &str,
    ) -> Result<()> {
        // Create simulation bin
        let sim_bin = self.create_elements(name_prefix)?;

        // Add to pipeline
        pipeline.add(&sim_bin)?;

        // Unlink original connection
        before_element.unlink(after_element);

        // Insert simulation bin
        before_element.link(&sim_bin)?;
        sim_bin.link(after_element)?;

        // Sync state with parent
        sim_bin.sync_state_with_parent()?;

        Ok(())
    }

    /// Apply current conditions to GStreamer elements
    pub fn apply_to_elements(&self) {
        let conditions = self.simulator.get_conditions();
        let elements = match self.elements.read() {
            Ok(e) => e,
            Err(_) => return,
        };

        if elements.using_netsim {
            // Apply conditions to netsim element
            if let Some(ref netsim) = elements.netsim {
                // Convert percentage to 0.0-1.0 probability for netsim
                let drop_prob = (conditions.packet_loss / 100.0).clamp(0.0, 1.0);
                netsim.set_property("drop-probability", drop_prob);

                // Packet duplication
                let dup_prob = (conditions.duplicate_probability / 100.0).clamp(0.0, 1.0);
                netsim.set_property("duplicate-probability", dup_prob);

                // Delay configuration
                if conditions.delay_probability > 0.0 {
                    let delay_prob = (conditions.delay_probability / 100.0).clamp(0.0, 1.0);
                    netsim.set_property("delay-probability", delay_prob);

                    // Set delay range
                    let min_delay = if conditions.min_delay_ms > 0 {
                        conditions.min_delay_ms as i32
                    } else {
                        conditions.latency_ms as i32
                    };
                    let max_delay = if conditions.max_delay_ms > 0 {
                        conditions.max_delay_ms as i32
                    } else {
                        (conditions.latency_ms + conditions.jitter_ms) as i32
                    };

                    netsim.set_property("min-delay", min_delay);
                    netsim.set_property("max-delay", max_delay);
                } else if conditions.latency_ms > 0 {
                    // Use uniform delay for latency simulation
                    netsim.set_property("delay-probability", 1.0f32);
                    netsim.set_property("min-delay", conditions.latency_ms as i32);
                    let max_delay = (conditions.latency_ms + conditions.jitter_ms) as i32;
                    netsim.set_property("max-delay", max_delay);
                } else {
                    netsim.set_property("delay-probability", 0.0f32);
                }

                // Packet reordering control
                netsim.set_property("allow-reordering", conditions.allow_reordering);

                // Bandwidth limiting
                if conditions.bandwidth_kbps > 0 {
                    netsim.set_property("max-kbps", conditions.bandwidth_kbps as i32);
                    // Set bucket size for burst tolerance (1 second worth of data)
                    let bucket_size = conditions.bandwidth_kbps as i32;
                    netsim.set_property("max-bucket-size", bucket_size);
                } else {
                    netsim.set_property("max-kbps", -1i32); // unlimited
                    netsim.set_property("max-bucket-size", -1i32);
                }
            }
        } else {
            // Fallback mode: Use old implementation with queue + identity
            if let Some(ref identity) = elements.identity {
                let drop_prob = (conditions.packet_loss / 100.0) as f32;
                identity.set_property("drop-probability", drop_prob);

                // Apply latency
                if conditions.latency_ms > 0 {
                    let latency_ns = conditions.latency_ms as u64 * 1_000_000;
                    identity.set_property("datarate", latency_ns as i32);
                }
            }

            // Apply bandwidth limits to queue
            if let Some(ref queue) = elements.queue {
                if conditions.bandwidth_kbps > 0 {
                    let buffer_bytes = (conditions.bandwidth_kbps * 1000 / 8) as u32;
                    queue.set_property("max-size-bytes", buffer_bytes);
                    queue.set_property("max-size-buffers", 0u32);
                    queue.set_property("max-size-time", 1_000_000_000u64);
                } else {
                    queue.set_property("max-size-bytes", 0u32);
                    queue.set_property("max-size-buffers", 1000u32);
                    queue.set_property("max-size-time", 0u64);
                }
            }
        }

        // Apply connection drops to valve (both modes)
        if let Some(ref valve) = elements.valve {
            valve.set_property("drop", conditions.connection_dropped);
        }
    }

    /// Get the simulator instance
    pub fn simulator(&self) -> &NetworkSimulator {
        &self.simulator
    }

    /// Enable simulation and apply conditions
    pub fn enable_with_conditions(&self, conditions: NetworkConditions) {
        self.simulator.apply_conditions(conditions);
        self.apply_to_elements();
    }

    /// Enable simulation with a profile
    pub fn enable_with_profile(&self, profile: NetworkProfile) {
        self.simulator.apply_profile(profile);
        self.apply_to_elements();
    }

    /// Simulate a temporary connection drop
    pub fn simulate_connection_drop(&self, duration: std::time::Duration) {
        self.simulator.drop_connection();
        self.apply_to_elements();

        let sim = self.simulator.clone();
        let elements = Arc::clone(&self.elements);
        std::thread::spawn(move || {
            std::thread::sleep(duration);
            sim.restore_connection();

            // Reapply to elements
            if let Ok(elems) = elements.read() {
                if let Some(ref valve) = elems.valve {
                    valve.set_property("drop", false);
                }
            }
        });
    }
}

impl NetworkController for GStreamerNetworkSimulator {
    fn apply_conditions(&self, conditions: NetworkConditions) {
        self.simulator.apply_conditions(conditions);
        self.apply_to_elements();
    }

    fn get_conditions(&self) -> NetworkConditions {
        self.simulator.get_conditions()
    }

    fn drop_connection(&self) {
        self.simulator.drop_connection();
        self.apply_to_elements();
    }

    fn restore_connection(&self) {
        self.simulator.restore_connection();
        self.apply_to_elements();
    }

    fn apply_profile(&self, profile: NetworkProfile) {
        self.simulator.apply_profile(profile);
        self.apply_to_elements();
    }

    fn reset(&self) {
        self.simulator.reset();
        self.apply_to_elements();
    }
}

/// Helper to add network simulation to a pipeline builder
pub fn add_network_simulation(
    pipeline: &gst::Pipeline,
    source: &gst::Element,
    sink: &gst::Element,
    profile: NetworkProfile,
) -> Result<GStreamerNetworkSimulator> {
    let simulator = GStreamerNetworkSimulator::new();

    // Insert simulation elements
    simulator.insert_into_pipeline(pipeline, source, sink, "network_sim")?;

    // Apply profile
    simulator.enable_with_profile(profile);

    Ok(simulator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        gst::init().unwrap();

        let sim = GStreamerNetworkSimulator::new();
        let bin = sim.create_elements("test").unwrap();

        assert!(bin.static_pad("sink").is_some());
        assert!(bin.static_pad("src").is_some());
    }

    #[test]
    fn test_condition_application() {
        gst::init().unwrap();

        let sim = GStreamerNetworkSimulator::new();
        let _bin = sim.create_elements("test").unwrap();

        let conditions = NetworkConditions {
            packet_loss: 10.0,
            latency_ms: 100,
            bandwidth_kbps: 1000,
            connection_dropped: false,
            jitter_ms: 20,
            duplicate_probability: 0.5,
            allow_reordering: true,
            min_delay_ms: 80,
            max_delay_ms: 120,
            delay_probability: 100.0,
        };

        sim.enable_with_conditions(conditions.clone());

        let current = sim.get_conditions();
        assert_eq!(current.packet_loss, conditions.packet_loss);
        assert_eq!(current.latency_ms, conditions.latency_ms);
    }
}
