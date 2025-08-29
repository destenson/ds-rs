use crate::error::{DeepStreamError, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::time::{Duration, Instant};

/// Pipeline state representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineState {
    Null,
    Ready,
    Paused,
    Playing,
}

impl From<gst::State> for PipelineState {
    fn from(state: gst::State) -> Self {
        match state {
            gst::State::Null => PipelineState::Null,
            gst::State::Ready => PipelineState::Ready,
            gst::State::Paused => PipelineState::Paused,
            gst::State::Playing => PipelineState::Playing,
            _ => PipelineState::Null,
        }
    }
}

impl From<PipelineState> for gst::State {
    fn from(state: PipelineState) -> Self {
        match state {
            PipelineState::Null => gst::State::Null,
            PipelineState::Ready => gst::State::Ready,
            PipelineState::Paused => gst::State::Paused,
            PipelineState::Playing => gst::State::Playing,
        }
    }
}

/// State transition information
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from: PipelineState,
    pub to: PipelineState,
    pub timestamp: Instant,
    pub success: bool,
    pub message: Option<String>,
}

/// Manages pipeline state transitions with validation and recovery
pub struct StateManager {
    current_state: PipelineState,
    pending_state: Option<PipelineState>,
    transition_history: Vec<StateTransition>,
    max_history_size: usize,
    state_change_timeout: Duration,
    allow_async: bool,
}

impl StateManager {
    /// Create a new state manager
    pub fn new() -> Self {
        Self {
            current_state: PipelineState::Null,
            pending_state: None,
            transition_history: Vec::new(),
            max_history_size: 100,
            state_change_timeout: Duration::from_secs(5),
            allow_async: true,
        }
    }

    /// Set the state change timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.state_change_timeout = timeout;
    }

    /// Set whether async state changes are allowed
    pub fn set_allow_async(&mut self, allow: bool) {
        self.allow_async = allow;
    }

    /// Get the current state
    pub fn current_state(&self) -> PipelineState {
        self.current_state
    }

    /// Get the pending state if any
    pub fn pending_state(&self) -> Option<PipelineState> {
        self.pending_state
    }

    /// Check if a state transition is valid
    pub fn is_valid_transition(&self, from: PipelineState, to: PipelineState) -> bool {
        match (from, to) {
            // Can always go to NULL from any state
            (_, PipelineState::Null) => true,

            // NULL can only go to READY
            (PipelineState::Null, PipelineState::Ready) => true,

            // READY can go to PAUSED
            (PipelineState::Ready, PipelineState::Paused) => true,

            // PAUSED can go to PLAYING or back to READY
            (PipelineState::Paused, PipelineState::Playing) => true,
            (PipelineState::Paused, PipelineState::Ready) => true,

            // PLAYING can only go back to PAUSED
            (PipelineState::Playing, PipelineState::Paused) => true,

            // All other transitions are invalid
            _ => false,
        }
    }

    /// Set the pipeline state with validation
    pub fn set_state(
        &mut self,
        pipeline: &gst::Pipeline,
        target_state: gst::State,
    ) -> Result<gst::StateChangeSuccess> {
        let target = PipelineState::from(target_state);
        let current = self.current_state;

        // Check if this is a valid direct transition
        if !self.is_valid_transition(current, target) {
            // Try to find intermediate states
            let intermediate_states = self.get_intermediate_states(current, target);

            if intermediate_states.is_empty() {
                return Err(DeepStreamError::StateChange(format!(
                    "Invalid state transition from {:?} to {:?}",
                    current, target
                )));
            }

            // Perform intermediate transitions
            for intermediate in intermediate_states {
                self.perform_transition(pipeline, intermediate)?;
            }
        }

        // Perform the final transition
        self.perform_transition(pipeline, target)
    }

    /// Get intermediate states for a transition
    fn get_intermediate_states(
        &self,
        from: PipelineState,
        to: PipelineState,
    ) -> Vec<PipelineState> {
        match (from, to) {
            // NULL to PLAYING needs READY and PAUSED
            (PipelineState::Null, PipelineState::Playing) => {
                vec![PipelineState::Ready, PipelineState::Paused]
            }

            // NULL to PAUSED needs READY
            (PipelineState::Null, PipelineState::Paused) => {
                vec![PipelineState::Ready]
            }

            // READY to PLAYING needs PAUSED
            (PipelineState::Ready, PipelineState::Playing) => {
                vec![PipelineState::Paused]
            }

            // PLAYING to NULL can go directly or through PAUSED and READY
            (PipelineState::Playing, PipelineState::Null) => {
                vec![PipelineState::Paused, PipelineState::Ready]
            }

            // PLAYING to READY needs PAUSED
            (PipelineState::Playing, PipelineState::Ready) => {
                vec![PipelineState::Paused]
            }

            // PAUSED to NULL can go through READY
            (PipelineState::Paused, PipelineState::Null) => {
                vec![PipelineState::Ready]
            }

            _ => vec![],
        }
    }

    /// Perform a state transition
    fn perform_transition(
        &mut self,
        pipeline: &gst::Pipeline,
        target: PipelineState,
    ) -> Result<gst::StateChangeSuccess> {
        let start_time = Instant::now();
        self.pending_state = Some(target);

        // Set the state on the pipeline
        let state_change_result = pipeline.set_state(target.into());

        let (success, message) = match state_change_result {
            Ok(gst::StateChangeSuccess::Success) => {
                self.current_state = target;
                self.pending_state = None;
                (true, None)
            }
            Ok(gst::StateChangeSuccess::Async) => {
                if self.allow_async {
                    // Wait for state change with timeout
                    let timeout =
                        gst::ClockTime::from_nseconds(self.state_change_timeout.as_nanos() as u64);
                    let (result, current, _pending) = pipeline.state(Some(timeout));
                    match result {
                        Ok(success) => {
                            self.current_state = PipelineState::from(current);
                            self.pending_state = None;
                            (
                                true,
                                Some(format!("Async state change completed: {:?}", success)),
                            )
                        }
                        Err(_) => {
                            self.pending_state = None;
                            (false, Some("Async state change timed out".to_string()))
                        }
                    }
                } else {
                    self.pending_state = None;
                    (false, Some("Async state changes not allowed".to_string()))
                }
            }
            Ok(gst::StateChangeSuccess::NoPreroll) => {
                // Live sources return NoPreroll
                self.current_state = target;
                self.pending_state = None;
                (true, Some("Live source detected (NoPreroll)".to_string()))
            }
            Err(err) => {
                self.pending_state = None;
                (false, Some(format!("State change failed: {:?}", err)))
            }
        };

        // Record the transition
        self.record_transition(StateTransition {
            from: self.current_state,
            to: target,
            timestamp: start_time,
            success,
            message: message.clone(),
        });

        if success {
            Ok(gst::StateChangeSuccess::Success)
        } else {
            Err(DeepStreamError::StateChange(message.unwrap_or_else(|| {
                format!("Failed to change state to {:?}", target)
            })))
        }
    }

    /// Record a state transition in history
    fn record_transition(&mut self, transition: StateTransition) {
        self.transition_history.push(transition);

        // Trim history if it exceeds max size
        if self.transition_history.len() > self.max_history_size {
            self.transition_history.remove(0);
        }
    }

    /// Get the state transition history
    pub fn transition_history(&self) -> &[StateTransition] {
        &self.transition_history
    }

    /// Clear the transition history
    pub fn clear_history(&mut self) {
        self.transition_history.clear();
    }

    /// Reset the state manager
    pub fn reset(&mut self) {
        self.current_state = PipelineState::Null;
        self.pending_state = None;
        self.clear_history();
    }

    /// Wait for a specific state
    pub fn wait_for_state(
        &mut self,
        pipeline: &gst::Pipeline,
        target: PipelineState,
        timeout: Duration,
    ) -> Result<()> {
        let start_time = Instant::now();
        let timeout_ns = timeout.as_nanos() as u64;

        loop {
            let elapsed = start_time.elapsed();
            if elapsed >= timeout {
                return Err(DeepStreamError::StateChange(format!(
                    "Timeout waiting for state {:?}",
                    target
                )));
            }

            let remaining = timeout - elapsed;
            let gst_timeout =
                gst::ClockTime::from_nseconds(remaining.as_nanos().min(timeout_ns as u128) as u64);

            let (result, current, _pending) = pipeline.state(Some(gst_timeout));
            if let Ok(_success) = result {
                let current_state = PipelineState::from(current);
                if current_state == target {
                    self.current_state = current_state;
                    return Ok(());
                }
            }

            // Small sleep to avoid busy waiting
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Recover from an error state by resetting to NULL
    pub fn recover(&mut self, pipeline: &gst::Pipeline) -> Result<()> {
        log::warn!("Attempting to recover pipeline from error state");

        // Force to NULL state
        pipeline.set_state(gst::State::Null).map_err(|e| {
            DeepStreamError::StateChange(format!("Failed to recover pipeline: {:?}", e))
        })?;

        self.current_state = PipelineState::Null;
        self.pending_state = None;

        // Record recovery
        self.record_transition(StateTransition {
            from: self.current_state,
            to: PipelineState::Null,
            timestamp: Instant::now(),
            success: true,
            message: Some("Recovery: forced to NULL state".to_string()),
        });

        Ok(())
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        let manager = StateManager::new();

        // Test valid transitions
        assert!(manager.is_valid_transition(PipelineState::Null, PipelineState::Ready));
        assert!(manager.is_valid_transition(PipelineState::Ready, PipelineState::Paused));
        assert!(manager.is_valid_transition(PipelineState::Paused, PipelineState::Playing));
        assert!(manager.is_valid_transition(PipelineState::Playing, PipelineState::Paused));

        // Test invalid direct transitions
        assert!(!manager.is_valid_transition(PipelineState::Null, PipelineState::Playing));
        assert!(!manager.is_valid_transition(PipelineState::Ready, PipelineState::Playing));
    }

    #[test]
    fn test_intermediate_states() {
        let manager = StateManager::new();

        // NULL to PLAYING should go through READY and PAUSED
        let intermediates =
            manager.get_intermediate_states(PipelineState::Null, PipelineState::Playing);
        assert_eq!(
            intermediates,
            vec![PipelineState::Ready, PipelineState::Paused]
        );

        // PLAYING to NULL should go through PAUSED and READY
        let intermediates =
            manager.get_intermediate_states(PipelineState::Playing, PipelineState::Null);
        assert_eq!(
            intermediates,
            vec![PipelineState::Paused, PipelineState::Ready]
        );
    }

    #[test]
    fn test_state_conversion() {
        assert_eq!(PipelineState::from(gst::State::Null), PipelineState::Null);
        assert_eq!(PipelineState::from(gst::State::Ready), PipelineState::Ready);
        assert_eq!(
            PipelineState::from(gst::State::Paused),
            PipelineState::Paused
        );
        assert_eq!(
            PipelineState::from(gst::State::Playing),
            PipelineState::Playing
        );

        assert_eq!(gst::State::from(PipelineState::Null), gst::State::Null);
        assert_eq!(gst::State::from(PipelineState::Ready), gst::State::Ready);
        assert_eq!(gst::State::from(PipelineState::Paused), gst::State::Paused);
        assert_eq!(
            gst::State::from(PipelineState::Playing),
            gst::State::Playing
        );
    }
}
