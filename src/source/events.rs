use crate::error::Result;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use super::{SourceId, SourceState};

#[derive(Debug, Clone)]
pub enum SourceEvent {
    SourceAdded {
        id: SourceId,
        uri: String,
    },
    SourceRemoved {
        id: SourceId,
    },
    StateChanged {
        id: SourceId,
        old_state: SourceState,
        new_state: SourceState,
    },
    PadAdded {
        id: SourceId,
        pad_name: String,
    },
    PadRemoved {
        id: SourceId,
        pad_name: String,
    },
    Eos {
        id: SourceId,
    },
    Error {
        id: SourceId,
        error: String,
    },
    Warning {
        id: SourceId,
        warning: String,
    },
}

pub struct SourceEventHandler {
    sender: Sender<SourceEvent>,
    receiver: Arc<Mutex<Receiver<SourceEvent>>>,
    callbacks: Arc<Mutex<Vec<Box<dyn Fn(&SourceEvent) + Send + 'static>>>>,
}

impl SourceEventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn sender(&self) -> Sender<SourceEvent> {
        self.sender.clone()
    }
    
    pub fn emit(&self, event: SourceEvent) -> Result<()> {
        println!("Emitting event: {:?}", event);
        
        if let Ok(callbacks) = self.callbacks.lock() {
            for callback in callbacks.iter() {
                callback(&event);
            }
        }
        
        self.sender.send(event)
            .map_err(|e| crate::error::DeepStreamError::Unknown(
                format!("Failed to send event: {}", e)
            ))
    }
    
    pub fn register_callback<F>(&self, callback: F)
    where
        F: Fn(&SourceEvent) + Send + 'static,
    {
        if let Ok(mut callbacks) = self.callbacks.lock() {
            callbacks.push(Box::new(callback));
        }
    }
    
    pub fn poll_event(&self) -> Option<SourceEvent> {
        if let Ok(receiver) = self.receiver.lock() {
            receiver.try_recv().ok()
        } else {
            None
        }
    }
    
    pub fn wait_for_event(&self) -> Result<SourceEvent> {
        let receiver = self.receiver.lock()
            .map_err(|_| crate::error::DeepStreamError::Unknown(
                "Failed to lock receiver".to_string()
            ))?;
        
        receiver.recv()
            .map_err(|e| crate::error::DeepStreamError::Unknown(
                format!("Failed to receive event: {}", e)
            ))
    }
}

impl Default for SourceEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

pub fn handle_bus_message(
    msg: &gst::Message,
    source_id: Option<SourceId>,
    event_handler: &SourceEventHandler,
) -> Result<()> {
    use gst::MessageView;
    
    match msg.view() {
        MessageView::Eos(_) => {
            if let Some(id) = source_id {
                event_handler.emit(SourceEvent::Eos { id })?;
            }
        }
        MessageView::Error(err) => {
            if let Some(id) = source_id {
                let error_msg = format!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                event_handler.emit(SourceEvent::Error {
                    id,
                    error: error_msg,
                })?;
            }
        }
        MessageView::Warning(warn) => {
            if let Some(id) = source_id {
                let warning_msg = format!(
                    "Warning from {:?}: {} ({:?})",
                    warn.src().map(|s| s.path_string()),
                    warn.error(),
                    warn.debug()
                );
                event_handler.emit(SourceEvent::Warning {
                    id,
                    warning: warning_msg,
                })?;
            }
        }
        MessageView::StateChanged(state_changed) => {
            if let Some(id) = source_id {
                let old = match state_changed.old() {
                    gst::State::Null => SourceState::Stopped,
                    gst::State::Ready => SourceState::Idle,
                    gst::State::Paused => SourceState::Paused,
                    gst::State::Playing => SourceState::Playing,
                    _ => SourceState::Idle,
                };
                
                let new = match state_changed.current() {
                    gst::State::Null => SourceState::Stopped,
                    gst::State::Ready => SourceState::Idle,
                    gst::State::Paused => SourceState::Paused,
                    gst::State::Playing => SourceState::Playing,
                    _ => SourceState::Idle,
                };
                
                if old != new {
                    event_handler.emit(SourceEvent::StateChanged {
                        id,
                        old_state: old,
                        new_state: new,
                    })?;
                }
            }
        }
        _ => {}
    }
    
    Ok(())
}

pub struct EosTracker {
    eos_list: Arc<Mutex<Vec<bool>>>,
}

impl EosTracker {
    pub fn new(max_sources: usize) -> Self {
        let mut eos_list = Vec::with_capacity(max_sources);
        eos_list.resize(max_sources, false);
        
        Self {
            eos_list: Arc::new(Mutex::new(eos_list)),
        }
    }
    
    pub fn mark_eos(&self, source_id: SourceId) -> Result<()> {
        let mut eos_list = self.eos_list.lock()
            .map_err(|_| crate::error::DeepStreamError::Unknown(
                "Failed to lock EOS list".to_string()
            ))?;
        
        if source_id.0 < eos_list.len() {
            eos_list[source_id.0] = true;
        }
        
        Ok(())
    }
    
    pub fn clear_eos(&self, source_id: SourceId) -> Result<()> {
        let mut eos_list = self.eos_list.lock()
            .map_err(|_| crate::error::DeepStreamError::Unknown(
                "Failed to lock EOS list".to_string()
            ))?;
        
        if source_id.0 < eos_list.len() {
            eos_list[source_id.0] = false;
        }
        
        Ok(())
    }
    
    pub fn is_eos(&self, source_id: SourceId) -> Result<bool> {
        let eos_list = self.eos_list.lock()
            .map_err(|_| crate::error::DeepStreamError::Unknown(
                "Failed to lock EOS list".to_string()
            ))?;
        
        if source_id.0 < eos_list.len() {
            Ok(eos_list[source_id.0])
        } else {
            Ok(false)
        }
    }
    
    pub fn get_eos_sources(&self) -> Result<Vec<SourceId>> {
        let eos_list = self.eos_list.lock()
            .map_err(|_| crate::error::DeepStreamError::Unknown(
                "Failed to lock EOS list".to_string()
            ))?;
        
        let mut eos_sources = Vec::new();
        for (i, &is_eos) in eos_list.iter().enumerate() {
            if is_eos {
                eos_sources.push(SourceId(i));
            }
        }
        
        Ok(eos_sources)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_handler() {
        let handler = SourceEventHandler::new();
        
        let event = SourceEvent::SourceAdded {
            id: SourceId(1),
            uri: "file:///test.mp4".to_string(),
        };
        
        handler.emit(event.clone()).unwrap();
        
        if let Some(received) = handler.poll_event() {
            match received {
                SourceEvent::SourceAdded { id, uri } => {
                    assert_eq!(id.0, 1);
                    assert_eq!(uri, "file:///test.mp4");
                }
                _ => panic!("Unexpected event type"),
            }
        } else {
            panic!("No event received");
        }
    }
    
    #[test]
    fn test_eos_tracker() {
        let tracker = EosTracker::new(5);
        
        let source1 = SourceId(1);
        let source2 = SourceId(2);
        
        tracker.mark_eos(source1).unwrap();
        assert!(tracker.is_eos(source1).unwrap());
        assert!(!tracker.is_eos(source2).unwrap());
        
        tracker.mark_eos(source2).unwrap();
        let eos_sources = tracker.get_eos_sources().unwrap();
        assert_eq!(eos_sources.len(), 2);
        assert!(eos_sources.contains(&source1));
        assert!(eos_sources.contains(&source2));
        
        tracker.clear_eos(source1).unwrap();
        assert!(!tracker.is_eos(source1).unwrap());
    }
}