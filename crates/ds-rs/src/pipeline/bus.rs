use crate::error::{DeepStreamError, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Bus message handler trait
pub trait MessageHandler: Send + Sync {
    /// Handle a bus message
    fn handle_message(&self, bus: &gst::Bus, msg: &gst::Message) -> gst::BusSyncReply;
}

/// Default message handler implementation
pub struct DefaultMessageHandler {
    log_errors: bool,
    log_warnings: bool,
    log_info: bool,
    handle_eos: bool,
}

impl DefaultMessageHandler {
    pub fn new() -> Self {
        Self {
            log_errors: true,
            log_warnings: true,
            log_info: false,
            handle_eos: true,
        }
    }

    pub fn with_logging(mut self, errors: bool, warnings: bool, info: bool) -> Self {
        self.log_errors = errors;
        self.log_warnings = warnings;
        self.log_info = info;
        self
    }

    pub fn with_eos_handling(mut self, handle: bool) -> Self {
        self.handle_eos = handle;
        self
    }
}

impl MessageHandler for DefaultMessageHandler {
    fn handle_message(&self, _bus: &gst::Bus, msg: &gst::Message) -> gst::BusSyncReply {
        match msg.view() {
            gst::MessageView::Error(err) => {
                if self.log_errors {
                    log::error!(
                        "Pipeline error from {:?}: {} ({:?})",
                        msg.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                }
            }
            gst::MessageView::Warning(warn) => {
                if self.log_warnings {
                    log::warn!(
                        "Pipeline warning from {:?}: {} ({:?})",
                        msg.src().map(|s| s.path_string()),
                        warn.error(),
                        warn.debug()
                    );
                }
            }
            gst::MessageView::Info(info) => {
                if self.log_info {
                    log::info!(
                        "Pipeline info from {:?}: {} ({:?})",
                        msg.src().map(|s| s.path_string()),
                        info.error(),
                        info.debug()
                    );
                }
            }
            gst::MessageView::Eos(_) => {
                if self.handle_eos {
                    log::info!("End of stream reached");
                }
            }
            gst::MessageView::StateChanged(state_changed) => {
                if self.log_info {
                    if let Some(src) = msg.src() {
                        log::debug!(
                            "State changed for {:?}: {:?} -> {:?} (pending: {:?})",
                            src.path_string(),
                            state_changed.old(),
                            state_changed.current(),
                            state_changed.pending()
                        );
                    }
                }
            }
            _ => {}
        }

        gst::BusSyncReply::Pass
    }
}

impl Default for DefaultMessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Bus watcher that handles messages in a separate thread
pub struct BusWatcher {
    bus: gst::Bus,
    thread_handle: Option<JoinHandle<()>>,
    stop_flag: Arc<Mutex<bool>>,
}

impl BusWatcher {
    /// Create a new bus watcher with a custom handler
    pub fn new<F>(bus: gst::Bus, handler: F) -> Result<Self>
    where
        F: Fn(&gst::Bus, &gst::Message) -> gst::BusSyncReply + Send + Sync + 'static,
    {
        let stop_flag = Arc::new(Mutex::new(false));
        let stop_flag_clone = stop_flag.clone();
        let bus_clone = bus.clone();

        let thread_handle = thread::spawn(move || {
            let handler = Arc::new(handler);

            loop {
                // Check stop flag
                if let Ok(stop) = stop_flag_clone.lock() {
                    if *stop {
                        break;
                    }
                }

                // Poll for messages with timeout
                if let Some(msg) = bus_clone.timed_pop(gst::ClockTime::from_mseconds(100)) {
                    handler(&bus_clone, &msg);
                }
            }
        });

        Ok(Self {
            bus,
            thread_handle: Some(thread_handle),
            stop_flag,
        })
    }

    /// Create a bus watcher with the default handler
    pub fn with_default_handler(bus: gst::Bus) -> Result<Self> {
        let handler = DefaultMessageHandler::new();
        Self::new(bus, move |bus, msg| handler.handle_message(bus, msg))
    }

    /// Stop the bus watcher
    pub fn stop(&mut self) {
        if let Ok(mut stop) = self.stop_flag.lock() {
            *stop = true;
        }

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    /// Get the bus
    pub fn bus(&self) -> &gst::Bus {
        &self.bus
    }
}

impl Drop for BusWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Stream-specific message handler for DeepStream
pub struct StreamMessageHandler {
    stream_handlers: Arc<Mutex<Vec<Box<dyn Fn(u32, &gst::Message) + Send + Sync>>>>,
}

impl StreamMessageHandler {
    pub fn new() -> Self {
        Self {
            stream_handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a stream-specific handler
    pub fn add_stream_handler<F>(&self, handler: F)
    where
        F: Fn(u32, &gst::Message) + Send + Sync + 'static,
    {
        if let Ok(mut handlers) = self.stream_handlers.lock() {
            handlers.push(Box::new(handler));
        }
    }

    /// Handle stream-specific EOS
    fn handle_stream_eos(&self, stream_id: u32, msg: &gst::Message) {
        log::info!("Stream {} received EOS", stream_id);

        if let Ok(handlers) = self.stream_handlers.lock() {
            for handler in handlers.iter() {
                handler(stream_id, msg);
            }
        }
    }

    /// Check if a message is a stream-specific EOS
    fn is_stream_eos(&self, _msg: &gst::Message) -> Option<u32> {
        // Check for custom stream EOS messages (DeepStream specific)
        // This would need FFI bindings for gst_nvmessage_is_stream_eos
        // For now, return None
        None
    }
}

impl MessageHandler for StreamMessageHandler {
    fn handle_message(&self, _bus: &gst::Bus, msg: &gst::Message) -> gst::BusSyncReply {
        // Check for stream-specific messages
        if let Some(stream_id) = self.is_stream_eos(msg) {
            self.handle_stream_eos(stream_id, msg);
        }

        // Let the default handler also process
        DefaultMessageHandler::new().handle_message(_bus, msg)
    }
}

/// Message callback manager for custom message handling
pub struct MessageCallbackManager {
    callbacks: Arc<Mutex<Vec<Box<dyn Fn(&gst::Message) -> bool + Send + Sync>>>>,
}

impl MessageCallbackManager {
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a callback for messages
    /// Return true from callback to stop propagation
    pub fn register_callback<F>(&self, callback: F)
    where
        F: Fn(&gst::Message) -> bool + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.callbacks.lock() {
            callbacks.push(Box::new(callback));
        }
    }

    /// Process a message through all callbacks
    pub fn process_message(&self, msg: &gst::Message) -> bool {
        if let Ok(callbacks) = self.callbacks.lock() {
            for callback in callbacks.iter() {
                if callback(msg) {
                    return true; // Stop propagation
                }
            }
        }
        false
    }
}

/// Utility functions for common bus operations
pub struct BusUtils;

impl BusUtils {
    /// Wait for a specific message type with timeout
    pub fn wait_for_message(
        bus: &gst::Bus,
        message_types: &[gst::MessageType],
        timeout: Option<Duration>,
    ) -> Result<gst::Message> {
        let timeout = timeout.map(|d| gst::ClockTime::from_nseconds(d.as_nanos() as u64));

        bus.timed_pop_filtered(timeout, message_types)
            .ok_or_else(|| DeepStreamError::Pipeline("Timeout waiting for message".to_string()))
    }

    /// Poll bus for messages without blocking
    pub fn poll_messages(bus: &gst::Bus) -> Vec<gst::Message> {
        let mut messages = Vec::new();

        while let Some(msg) = bus.pop() {
            messages.push(msg);
        }

        messages
    }

    /// Clear all pending messages from the bus
    pub fn clear_bus(bus: &gst::Bus) {
        while bus.pop().is_some() {
            // Discard messages
        }
    }

    /// Wait for EOS or error
    pub fn wait_for_eos_or_error(bus: &gst::Bus, timeout: Option<Duration>) -> Result<()> {
        let msg = Self::wait_for_message(
            bus,
            &[gst::MessageType::Eos, gst::MessageType::Error],
            timeout,
        )?;

        match msg.view() {
            gst::MessageView::Eos(_) => Ok(()),
            gst::MessageView::Error(err) => Err(DeepStreamError::Pipeline(format!(
                "Pipeline error: {}",
                err.error()
            ))),
            _ => Ok(()),
        }
    }

    /// Create a simple logging handler
    pub fn create_logging_handler() -> impl Fn(&gst::Bus, &gst::Message) -> gst::BusSyncReply {
        move |_bus, msg| {
            match msg.view() {
                gst::MessageView::Error(err) => {
                    log::error!("Error: {} ({:?})", err.error(), err.debug());
                }
                gst::MessageView::Warning(warn) => {
                    log::warn!("Warning: {} ({:?})", warn.error(), warn.debug());
                }
                gst::MessageView::Info(info) => {
                    log::info!("Info: {} ({:?})", info.error(), info.debug());
                }
                gst::MessageView::Eos(_) => {
                    log::info!("End of stream");
                }
                _ => {}
            }
            gst::BusSyncReply::Pass
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_message_handler() {
        let handler = DefaultMessageHandler::new()
            .with_logging(true, true, false)
            .with_eos_handling(true);

        // Create a test pipeline to get a bus
        let _ = gst::init();
        let pipeline = gst::Pipeline::new();
        let bus = pipeline.bus().unwrap();

        // Create a test message
        let msg = gst::message::Error::builder(gst::CoreError::Failed, "Test error")
            .src(&pipeline)
            .build();

        // Handle the message
        let reply = handler.handle_message(&bus, &msg);
        assert_eq!(reply, gst::BusSyncReply::Pass);
    }

    #[test]
    fn test_message_callback_manager() {
        let manager = MessageCallbackManager::new();

        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        manager.register_callback(move |_msg| {
            if let Ok(mut count) = counter_clone.lock() {
                *count += 1;
            }
            false // Don't stop propagation
        });

        // Create a test message
        let _ = gst::init();
        let pipeline = gst::Pipeline::new();
        let msg = gst::message::Eos::builder().src(&pipeline).build();

        // Process the message
        manager.process_message(&msg);

        // Check that callback was called
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_bus_utils() {
        let _ = gst::init();
        let pipeline = gst::Pipeline::new();
        let bus = pipeline.bus().unwrap();

        // Test clearing bus
        BusUtils::clear_bus(&bus);

        // Test polling (should be empty)
        let messages = BusUtils::poll_messages(&bus);
        assert!(messages.is_empty());
    }
}
