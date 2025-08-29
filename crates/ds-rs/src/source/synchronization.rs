use super::{SourceId, SourceManager, SourceState};
use crate::error::{DeepStreamError, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct SourceSynchronizer {
    manager: Arc<SourceManager>,
}

impl SourceSynchronizer {
    pub fn new(manager: Arc<SourceManager>) -> Self {
        Self { manager }
    }

    pub fn sync_source_with_pipeline(&self, source_id: SourceId) -> Result<()> {
        let _pipeline = self
            .manager
            .get_pipeline()
            .ok_or_else(|| DeepStreamError::NotInitialized("Pipeline not set".to_string()))?;

        let source = self.manager.get_source(source_id)?;

        // Use sync_state_with_parent() to properly synchronize with pipeline
        // This ensures the element inherits the pipeline's clock and base time
        let element = source.element();
        element.sync_state_with_parent()?;

        Ok(())
    }

    pub fn wait_for_state(
        &self,
        source_id: SourceId,
        target_state: SourceState,
        timeout: Duration,
    ) -> Result<()> {
        let start = std::time::Instant::now();

        loop {
            let source = self.manager.get_source(source_id)?;
            let current_state = source.current_state();

            if current_state == target_state {
                return Ok(());
            }

            if start.elapsed() >= timeout {
                return Err(DeepStreamError::Timeout(format!(
                    "Timeout waiting for source {} to reach state {:?}",
                    source_id, target_state
                )));
            }

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn ensure_all_sources_state(&self, target_state: gst::State) -> Result<()> {
        let source_ids = self.manager.list_sources()?;

        for id in source_ids {
            let source = self.manager.get_source(id)?;
            source.set_state(target_state)?;
        }

        Ok(())
    }
}

pub struct PadProbe {
    pad: gst::Pad,
    probe_id: Option<gst::PadProbeId>,
}

impl PadProbe {
    pub fn new(pad: gst::Pad) -> Self {
        Self {
            pad,
            probe_id: None,
        }
    }

    pub fn add_blocking_probe<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(&gst::Pad, &mut gst::PadProbeInfo) -> gst::PadProbeReturn + Send + Sync + 'static,
    {
        let probe_id = self.pad.add_probe(
            gst::PadProbeType::BLOCK | gst::PadProbeType::DATA_DOWNSTREAM,
            callback,
        );

        self.probe_id = probe_id;
        Ok(())
    }

    pub fn add_idle_probe<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(&gst::Pad, &mut gst::PadProbeInfo) -> gst::PadProbeReturn + Send + Sync + 'static,
    {
        let probe_id = self.pad.add_probe(gst::PadProbeType::IDLE, callback);

        self.probe_id = probe_id;
        Ok(())
    }

    pub fn remove_probe(&mut self) {
        if let Some(probe_id) = self.probe_id.take() {
            self.pad.remove_probe(probe_id);
        }
    }
}

impl Drop for PadProbe {
    fn drop(&mut self) {
        self.remove_probe();
    }
}

pub fn block_source_pad(source_id: SourceId, pad: &gst::Pad) -> Result<PadProbe> {
    let mut probe = PadProbe::new(pad.clone());

    probe.add_blocking_probe(move |_pad, _info| {
        println!("Blocking data flow for source {}", source_id);
        gst::PadProbeReturn::Ok
    })?;

    Ok(probe)
}

pub fn add_eos_probe(source_id: SourceId, pad: &gst::Pad) -> Result<PadProbe> {
    let mut probe = PadProbe::new(pad.clone());

    probe.add_idle_probe(move |pad, _info| {
        println!("Sending EOS for source {}", source_id);
        pad.send_event(gst::event::Eos::new());
        gst::PadProbeReturn::Remove
    })?;

    Ok(probe)
}

#[derive(Clone)]
pub struct StateTransition {
    pub from: SourceState,
    pub to: SourceState,
    pub duration: Duration,
}

impl StateTransition {
    pub fn new(from: SourceState, to: SourceState) -> Self {
        Self {
            from,
            to,
            duration: Duration::from_millis(0),
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

pub struct TransitionManager {
    transitions: Arc<Mutex<Vec<StateTransition>>>,
}

impl TransitionManager {
    pub fn new() -> Self {
        Self {
            transitions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn record_transition(&self, transition: StateTransition) -> Result<()> {
        let mut transitions = self
            .transitions
            .lock()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock transitions".to_string()))?;

        transitions.push(transition);

        const MAX_HISTORY: usize = 100;
        if transitions.len() > MAX_HISTORY {
            let drain_count = transitions.len() - MAX_HISTORY;
            transitions.drain(0..drain_count);
        }

        Ok(())
    }

    pub fn get_transitions(&self) -> Result<Vec<StateTransition>> {
        let transitions = self
            .transitions
            .lock()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock transitions".to_string()))?;

        Ok(transitions.clone())
    }

    pub fn clear_history(&self) -> Result<()> {
        let mut transitions = self
            .transitions
            .lock()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock transitions".to_string()))?;

        transitions.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transition() {
        let transition = StateTransition::new(SourceState::Idle, SourceState::Playing)
            .with_duration(Duration::from_secs(1));

        assert_eq!(transition.from, SourceState::Idle);
        assert_eq!(transition.to, SourceState::Playing);
        assert_eq!(transition.duration, Duration::from_secs(1));
    }

    #[test]
    fn test_transition_manager() {
        let manager = TransitionManager::new();

        let transition1 = StateTransition::new(SourceState::Idle, SourceState::Playing);
        let transition2 = StateTransition::new(SourceState::Playing, SourceState::Paused);

        manager.record_transition(transition1).unwrap();
        manager.record_transition(transition2).unwrap();

        let transitions = manager.get_transitions().unwrap();
        assert_eq!(transitions.len(), 2);

        manager.clear_history().unwrap();
        let transitions = manager.get_transitions().unwrap();
        assert_eq!(transitions.len(), 0);
    }
}
