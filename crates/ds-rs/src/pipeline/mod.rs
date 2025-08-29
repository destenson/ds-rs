pub mod builder;
pub mod bus;
pub mod state;

use crate::backend::BackendManager;
use crate::error::{DeepStreamError, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub use builder::PipelineBuilder;
pub use bus::{BusWatcher, MessageHandler};
pub use state::{PipelineState, StateManager};

/// Main pipeline struct that wraps GStreamer pipeline with additional management
pub struct Pipeline {
    /// The underlying GStreamer pipeline
    gst_pipeline: gst::Pipeline,

    /// State manager for handling state transitions
    state_manager: Arc<Mutex<StateManager>>,

    /// Bus watcher for handling messages
    bus_watcher: Option<BusWatcher>,

    /// Backend manager for element creation
    backend_manager: Arc<BackendManager>,

    /// Pipeline name
    name: String,
}

impl Pipeline {
    /// Create a new pipeline builder
    pub fn builder(name: impl Into<String>) -> PipelineBuilder {
        PipelineBuilder::new(name)
    }

    /// Create a new pipeline with default settings
    pub fn new(name: impl Into<String>) -> Result<Self> {
        Self::builder(name).build()
    }

    /// Get the underlying GStreamer pipeline
    pub fn gst_pipeline(&self) -> &gst::Pipeline {
        &self.gst_pipeline
    }

    /// Get the pipeline name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add an element to the pipeline
    pub fn add_element(&self, element: &gst::Element) -> Result<()> {
        self.gst_pipeline.add(element).map_err(|_| {
            DeepStreamError::Pipeline(format!("Failed to add element to pipeline {}", self.name))
        })
    }

    /// Add multiple elements to the pipeline
    pub fn add_many(&self, elements: &[&gst::Element]) -> Result<()> {
        self.gst_pipeline.add_many(elements).map_err(|_| {
            DeepStreamError::Pipeline(format!("Failed to add elements to pipeline {}", self.name))
        })
    }

    /// Remove an element from the pipeline
    pub fn remove_element(&self, element: &gst::Element) -> Result<()> {
        self.gst_pipeline.remove(element).map_err(|_| {
            DeepStreamError::Pipeline(format!(
                "Failed to remove element from pipeline {}",
                self.name
            ))
        })
    }

    /// Link two elements in the pipeline
    pub fn link_elements(&self, src: &gst::Element, dest: &gst::Element) -> Result<()> {
        src.link(dest).map_err(|_| {
            DeepStreamError::PadLinking(format!(
                "Failed to link elements in pipeline {}",
                self.name
            ))
        })
    }

    /// Link multiple elements in sequence
    pub fn link_many(&self, elements: &[&gst::Element]) -> Result<()> {
        gst::Element::link_many(elements).map_err(|_| {
            DeepStreamError::PadLinking(format!(
                "Failed to link element chain in pipeline {}",
                self.name
            ))
        })
    }

    /// Set the pipeline state
    pub fn set_state(&self, state: gst::State) -> Result<gst::StateChangeSuccess> {
        let mut state_manager = self
            .state_manager
            .lock()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock state manager".to_string()))?;

        state_manager.set_state(&self.gst_pipeline, state)
    }

    /// Get the current pipeline state
    pub fn get_state(
        &self,
        timeout: Option<Duration>,
    ) -> Result<(gst::StateChangeSuccess, gst::State, gst::State)> {
        let timeout = timeout.map(|d| gst::ClockTime::from_nseconds(d.as_nanos() as u64));

        let (result, current, pending) = self.gst_pipeline.state(timeout);

        result
            .map(|success| (success, current, pending))
            .map_err(|_| {
                DeepStreamError::StateChange(format!(
                    "Failed to get state for pipeline {}",
                    self.name
                ))
            })
    }

    /// Get the current state without pending state
    pub fn current_state(&self) -> Result<gst::State> {
        let (_, current, _) = self.get_state(Some(Duration::from_millis(100)))?;
        Ok(current)
    }

    /// Play the pipeline
    pub fn play(&self) -> Result<()> {
        self.set_state(gst::State::Playing)?;
        Ok(())
    }

    /// Pause the pipeline
    pub fn pause(&self) -> Result<()> {
        self.set_state(gst::State::Paused)?;
        Ok(())
    }

    /// Stop the pipeline
    pub fn stop(&self) -> Result<()> {
        self.set_state(gst::State::Null)?;
        Ok(())
    }

    /// Check if the pipeline is playing
    pub fn is_playing(&self) -> bool {
        matches!(self.current_state(), Ok(gst::State::Playing))
    }

    /// Check if the pipeline is paused
    pub fn is_paused(&self) -> bool {
        matches!(self.current_state(), Ok(gst::State::Paused))
    }

    /// Send an EOS event to the pipeline
    pub fn send_eos(&self) -> Result<()> {
        self.gst_pipeline.send_event(gst::event::Eos::new());
        Ok(())
    }

    /// Get a reference to the backend manager
    pub fn backend_manager(&self) -> &Arc<BackendManager> {
        &self.backend_manager
    }

    /// Get an element by name from the pipeline
    pub fn get_by_name(&self, name: &str) -> Option<gst::Element> {
        self.gst_pipeline.by_name(name)
    }

    /// Set the pipeline to use a specific clock
    pub fn use_clock(&self, clock: Option<&gst::Clock>) {
        self.gst_pipeline.use_clock(clock);
    }

    /// Get the pipeline's clock
    pub fn clock(&self) -> Option<gst::Clock> {
        self.gst_pipeline.clock()
    }

    /// Set whether to automatically flush the bus on NULL state
    pub fn set_auto_flush_bus(&self, auto_flush: bool) {
        self.gst_pipeline.set_auto_flush_bus(auto_flush);
    }

    /// Get the pipeline's bus
    pub fn bus(&self) -> Option<gst::Bus> {
        self.gst_pipeline.bus()
    }

    /// Start watching the bus with a message handler
    pub fn start_bus_watch<F>(&mut self, handler: F) -> Result<()>
    where
        F: Fn(&gst::Bus, &gst::Message) -> gst::BusSyncReply + Send + Sync + 'static,
    {
        if let Some(bus) = self.bus() {
            let watcher = BusWatcher::new(bus, handler)?;
            self.bus_watcher = Some(watcher);
            Ok(())
        } else {
            Err(DeepStreamError::Pipeline(format!(
                "No bus available for pipeline {}",
                self.name
            )))
        }
    }

    /// Stop watching the bus
    pub fn stop_bus_watch(&mut self) {
        self.bus_watcher = None;
    }

    /// Wait for EOS or error with timeout
    pub fn wait_for_eos(&self, timeout: Option<Duration>) -> Result<()> {
        if let Some(bus) = self.bus() {
            let timeout = timeout.map(|d| gst::ClockTime::from_nseconds(d.as_nanos() as u64));

            let msg =
                bus.timed_pop_filtered(timeout, &[gst::MessageType::Eos, gst::MessageType::Error]);

            match msg {
                Some(msg) => match msg.view() {
                    gst::MessageView::Eos(_) => Ok(()),
                    gst::MessageView::Error(err) => Err(DeepStreamError::Pipeline(format!(
                        "Pipeline error: {:?}",
                        err.error()
                    ))),
                    _ => Ok(()),
                },
                None => {
                    if timeout.is_some() {
                        Err(DeepStreamError::Pipeline(
                            "Timeout waiting for EOS".to_string(),
                        ))
                    } else {
                        Ok(())
                    }
                }
            }
        } else {
            Err(DeepStreamError::Pipeline("No bus available".to_string()))
        }
    }

    /// Seek to a specific position in the pipeline
    pub fn seek(&self, position: Duration) -> Result<()> {
        let position = gst::ClockTime::from_nseconds(position.as_nanos() as u64);

        self.gst_pipeline
            .seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT, position)
            .map_err(|_| {
                DeepStreamError::Pipeline(format!("Failed to seek in pipeline {}", self.name))
            })
    }

    /// Get the current position in the pipeline
    pub fn position(&self) -> Result<Duration> {
        self.gst_pipeline
            .query_position::<gst::ClockTime>()
            .map(|pos| Duration::from_nanos(pos.nseconds()))
            .ok_or_else(|| {
                DeepStreamError::Pipeline(format!(
                    "Failed to query position for pipeline {}",
                    self.name
                ))
            })
    }

    /// Get the duration of the pipeline
    pub fn duration(&self) -> Result<Duration> {
        self.gst_pipeline
            .query_duration::<gst::ClockTime>()
            .map(|dur| Duration::from_nanos(dur.nseconds()))
            .ok_or_else(|| {
                DeepStreamError::Pipeline(format!(
                    "Failed to query duration for pipeline {}",
                    self.name
                ))
            })
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        // Ensure pipeline is stopped when dropped
        let _ = self.stop();
        self.stop_bus_watch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let _ = gst::init();
        let pipeline = Pipeline::new("test-pipeline");
        assert!(pipeline.is_ok());
    }

    #[test]
    fn test_pipeline_state_changes() {
        let _ = gst::init();
        let pipeline = Pipeline::new("test-pipeline").unwrap();

        // Initial state should be NULL
        assert_eq!(pipeline.current_state().unwrap(), gst::State::Null);

        // Test state transitions
        assert!(pipeline.set_state(gst::State::Ready).is_ok());
        assert!(pipeline.set_state(gst::State::Paused).is_ok());
        assert!(pipeline.play().is_ok());
        assert!(pipeline.is_playing());

        assert!(pipeline.pause().is_ok());
        assert!(pipeline.is_paused());

        assert!(pipeline.stop().is_ok());
        assert_eq!(pipeline.current_state().unwrap(), gst::State::Null);
    }

    #[test]
    fn test_element_management() {
        let _ = gst::init();
        let pipeline = Pipeline::new("test-pipeline").unwrap();

        // Create test elements
        let source = gst::ElementFactory::make("fakesrc")
            .name("test-source")
            .build()
            .unwrap();
        let sink = gst::ElementFactory::make("fakesink")
            .name("test-sink")
            .build()
            .unwrap();

        // Add elements
        assert!(pipeline.add_many(&[&source, &sink]).is_ok());

        // Link elements
        assert!(pipeline.link_elements(&source, &sink).is_ok());

        // Get element by name
        assert!(pipeline.get_by_name("test-source").is_some());
        assert!(pipeline.get_by_name("test-sink").is_some());

        // Remove element
        assert!(pipeline.remove_element(&source).is_ok());
    }
}
