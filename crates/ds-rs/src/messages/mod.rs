#![allow(unused)]
//! DeepStream-specific message handling

use gstreamer as gst;
use gstreamer::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Errors that can occur during message processing
#[derive(Debug, Error)]
pub enum MessageError {
    #[error("Invalid message format")]
    InvalidFormat,
    
    #[error("Message parsing failed: {0}")]
    ParseFailed(String),
    
    #[error("Unknown stream ID: {0}")]
    UnknownStream(u32),
}

pub type Result<T> = std::result::Result<T, MessageError>;

/// DeepStream message types
#[derive(Debug, Clone, PartialEq)]
pub enum DSMessageType {
    /// Stream-specific EOS
    StreamEos(u32),
    
    /// Stream added
    StreamAdded(u32),
    
    /// Stream removed
    StreamRemoved(u32),
    
    /// Inference done
    InferenceDone(u32),
    
    /// Custom application message
    Custom(String),
}

/// Stream EOS tracker
#[derive(Clone)]
pub struct StreamEosTracker {
    /// Map of stream IDs to EOS status
    eos_status: Arc<Mutex<HashMap<u32, bool>>>,
    
    /// Callback for when stream receives EOS
    eos_callbacks: Arc<Mutex<Vec<Box<dyn Fn(u32) + Send + Sync>>>>,
}

impl std::fmt::Debug for StreamEosTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamEosTracker")
            .field("eos_status", &self.eos_status)
            .field("eos_callbacks", &format!("{} callbacks", self.eos_callbacks.lock().map(|c| c.len()).unwrap_or(0)))
            .finish()
    }
}

impl StreamEosTracker {
    /// Create new EOS tracker
    pub fn new() -> Self {
        Self {
            eos_status: Arc::new(Mutex::new(HashMap::new())),
            eos_callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Register a stream
    pub fn register_stream(&self, stream_id: u32) {
        if let Ok(mut status) = self.eos_status.lock() {
            status.insert(stream_id, false);
        }
    }
    
    /// Unregister a stream
    pub fn unregister_stream(&self, stream_id: u32) {
        if let Ok(mut status) = self.eos_status.lock() {
            status.remove(&stream_id);
        }
    }
    
    /// Mark stream as EOS
    pub fn mark_eos(&self, stream_id: u32) {
        if let Ok(mut status) = self.eos_status.lock() {
            status.insert(stream_id, true);
        }
        
        // Call callbacks
        if let Ok(callbacks) = self.eos_callbacks.lock() {
            for callback in callbacks.iter() {
                callback(stream_id);
            }
        }
    }
    
    /// Check if stream has received EOS
    pub fn is_eos(&self, stream_id: u32) -> bool {
        self.eos_status.lock()
            .ok()
            .and_then(|status| status.get(&stream_id).copied())
            .unwrap_or(false)
    }
    
    /// Get all streams that have received EOS
    pub fn get_eos_streams(&self) -> Vec<u32> {
        self.eos_status.lock()
            .ok()
            .map(|status| {
                status.iter()
                    .filter(|&(_, &eos)| eos)
                    .map(|(&id, _)| id)
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Reset EOS status for a stream
    pub fn reset_eos(&self, stream_id: u32) {
        if let Ok(mut status) = self.eos_status.lock() {
            status.insert(stream_id, false);
        }
    }
    
    /// Add EOS callback
    pub fn add_eos_callback<F>(&self, callback: F)
    where
        F: Fn(u32) + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.eos_callbacks.lock() {
            callbacks.push(Box::new(callback));
        }
    }
    
    /// Clear all EOS statuses
    pub fn clear(&self) {
        if let Ok(mut status) = self.eos_status.lock() {
            for value in status.values_mut() {
                *value = false;
            }
        }
    }
}

impl Default for StreamEosTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// DeepStream message handler
pub struct DSMessageHandler {
    /// EOS tracker
    eos_tracker: StreamEosTracker,
    
    /// Message callbacks
    callbacks: Arc<Mutex<HashMap<String, Vec<Box<dyn Fn(DSMessageType) + Send + Sync>>>>>,
}

impl DSMessageHandler {
    /// Create new message handler
    pub fn new() -> Self {
        Self {
            eos_tracker: StreamEosTracker::new(),
            callbacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Check if message is stream EOS
    pub fn is_stream_eos(msg: &gst::Message) -> bool {
        // In real DeepStream, this would call gst_nvmessage_is_stream_eos
        // For now, check if it's an element message with specific structure
        matches!(msg.view(), gst::MessageView::Element(_))
    }
    
    /// Parse stream EOS from message
    pub fn parse_stream_eos(msg: &gst::Message) -> Result<u32> {
        // In real DeepStream, this would call gst_nvmessage_parse_stream_eos
        // For now, return mock stream ID
        if Self::is_stream_eos(msg) {
            Ok(0) // Mock stream ID
        } else {
            Err(MessageError::InvalidFormat)
        }
    }
    
    /// Handle GStreamer message
    pub fn handle_message(&self, msg: &gst::Message) -> Result<()> {
        match msg.view() {
            gst::MessageView::Element(_element_msg) => {
                // Check for stream EOS
                if Self::is_stream_eos(msg) {
                    if let Ok(stream_id) = Self::parse_stream_eos(msg) {
                        self.eos_tracker.mark_eos(stream_id);
                        self.emit_message(DSMessageType::StreamEos(stream_id));
                    }
                }
            }
            gst::MessageView::Eos(_) => {
                // Global EOS - mark all streams as EOS
                for stream_id in 0..10 {
                    if self.eos_tracker.is_eos(stream_id) {
                        continue;
                    }
                    self.eos_tracker.mark_eos(stream_id);
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Register callback for message type
    pub fn register_callback<F>(&self, msg_type: &str, callback: F)
    where
        F: Fn(DSMessageType) + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.callbacks.lock() {
            callbacks.entry(msg_type.to_string())
                .or_insert_with(Vec::new)
                .push(Box::new(callback));
        }
    }
    
    /// Emit message to callbacks
    fn emit_message(&self, msg: DSMessageType) {
        let msg_type = match &msg {
            DSMessageType::StreamEos(_) => "stream_eos",
            DSMessageType::StreamAdded(_) => "stream_added",
            DSMessageType::StreamRemoved(_) => "stream_removed",
            DSMessageType::InferenceDone(_) => "inference_done",
            DSMessageType::Custom(_) => "custom",
        };
        
        if let Ok(callbacks) = self.callbacks.lock() {
            if let Some(cbs) = callbacks.get(msg_type) {
                for callback in cbs {
                    callback(msg.clone());
                }
            }
        }
    }
    
    /// Get EOS tracker
    pub fn eos_tracker(&self) -> &StreamEosTracker {
        &self.eos_tracker
    }
}

impl Default for DSMessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

use gst::glib::ControlFlow;

/// Helper trait for bus message handling
pub trait DSMessageBusExt {
    /// Add DeepStream message handler to bus
    fn add_ds_watch<F>(&self, handler: Arc<DSMessageHandler>, callback: F) -> Option<gst::bus::BusWatchGuard>
    where
        F: Fn(&gst::Bus, &gst::Message) -> ControlFlow + Send + Sync + 'static;
}

impl DSMessageBusExt for gst::Bus {
    fn add_ds_watch<F>(&self, handler: Arc<DSMessageHandler>, callback: F) -> Option<gst::bus::BusWatchGuard>
    where
        F: Fn(&gst::Bus, &gst::Message) -> ControlFlow + Send + Sync + 'static,
    {
        self.add_watch(move |bus, msg| {
            // Handle DeepStream-specific messages
            handler.handle_message(msg).ok();
            
            // Call user callback
            callback(bus, msg)
        })
        .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_eos_tracker() {
        let tracker = StreamEosTracker::new();
        
        tracker.register_stream(0);
        tracker.register_stream(1);
        
        assert!(!tracker.is_eos(0));
        
        tracker.mark_eos(0);
        assert!(tracker.is_eos(0));
        assert!(!tracker.is_eos(1));
        
        let eos_streams = tracker.get_eos_streams();
        assert_eq!(eos_streams.len(), 1);
        assert!(eos_streams.contains(&0));
        
        tracker.reset_eos(0);
        assert!(!tracker.is_eos(0));
    }
    
    #[test]
    fn test_eos_callbacks() {
        let tracker = StreamEosTracker::new();
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();
        
        tracker.add_eos_callback(move |_stream_id| {
            if let Ok(mut c) = called_clone.lock() {
                *c = true;
            }
        });
        
        tracker.register_stream(0);
        tracker.mark_eos(0);
        
        assert!(*called.lock().unwrap());
    }
    
    #[test]
    fn test_message_handler() {
        let handler = DSMessageHandler::new();
        let received = Arc::new(Mutex::new(false));
        let received_clone = received.clone();
        
        handler.register_callback("stream_eos", move |msg| {
            if matches!(msg, DSMessageType::StreamEos(_)) {
                if let Ok(mut r) = received_clone.lock() {
                    *r = true;
                }
            }
        });
        
        // Simulate stream EOS
        handler.eos_tracker().register_stream(0);
        handler.eos_tracker().mark_eos(0);
        handler.emit_message(DSMessageType::StreamEos(0));
        
        assert!(*received.lock().unwrap());
    }
}
