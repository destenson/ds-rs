use crate::error::{DeepStreamError, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};
use super::{SourceId, SourceState};

pub struct VideoSource {
    source_bin: gst::Element,
    source_id: SourceId,
    uri: String,
    state: Arc<Mutex<SourceState>>,
    pad_added_handler: Option<gstreamer::glib::signal::SignalHandlerId>,
}

impl Clone for VideoSource {
    fn clone(&self) -> Self {
        Self {
            source_bin: self.source_bin.clone(),
            source_id: self.source_id,
            uri: self.uri.clone(),
            state: self.state.clone(),
            pad_added_handler: None,  // Don't clone signal handlers
        }
    }
}

impl VideoSource {
    pub fn new(source_id: SourceId, uri: &str) -> Result<Self> {
        let bin_name = format!("source-bin-{:02}", source_id.0);
        
        // Fix Windows file URI format
        let fixed_uri = if cfg!(target_os = "windows") && uri.starts_with("file://") {
            // Normalize file URIs on Windows
            let normalized = if uri.starts_with("file:///") {
                // Already has three slashes, keep as is
                uri.to_string()
            } else {
                // Has only two slashes, need to add one for absolute paths
                let path = uri.strip_prefix("file://").unwrap_or(uri);
                // Only add extra slash if path starts with / (absolute Unix-style path)
                if path.starts_with('/') {
                    format!("file://{}", path)
                } else {
                    format!("file:///{}", path)
                }
            };
            normalized.replace('\\', "/")
        } else {
            uri.to_string()
        };
        
        let source_bin = gst::ElementFactory::make("uridecodebin")
            .name(&bin_name)
            .property("uri", &fixed_uri)
            .build()
            .map_err(|_| DeepStreamError::ElementCreation {
                element: format!("uridecodebin for source {}", source_id)
            })?;
        
        Ok(Self {
            source_bin,
            source_id,
            uri: fixed_uri,
            state: Arc::new(Mutex::new(SourceState::Idle)),
            pad_added_handler: None,
        })
    }
    
    pub fn connect_pad_added<F>(&mut self, streammux: &gst::Element, callback: F) -> Result<()>
    where
        F: Fn(&gst::Element, &gst::Pad, SourceId, &gst::Element) + Send + Sync + 'static,
    {
        let source_id = self.source_id;
        let streammux_weak = streammux.downgrade();
        
        let handler_id = self.source_bin.connect_pad_added(move |decodebin, pad| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let timestamp = format!("{:.3}", now.as_secs_f64());
            println!("[{}] pad-added callback triggered for source {} (pad: {})", 
                timestamp, source_id, pad.name());
            
            if let Some(streammux) = streammux_weak.upgrade() {
                callback(decodebin, pad, source_id, &streammux);
            } else {
                eprintln!("[{}] Failed to upgrade streammux weak reference for source {}", 
                    timestamp, source_id);
            }
        });
        
        self.pad_added_handler = Some(handler_id);
        Ok(())
    }
    
    pub fn connect_pad_added_default(&mut self, streammux: &gst::Element) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp = format!("{:.3}", now.as_secs_f64());
        println!("[{}] Connecting pad-added callback for source {}", timestamp, self.source_id);
        
        self.connect_pad_added(streammux, |_decodebin, pad, source_id, mux| {
            let caps = pad.current_caps().unwrap_or_else(|| pad.query_caps(None));
            
            let Some(structure) = caps.structure(0) else {
                eprintln!("Failed to get caps structure for source {}", source_id);
                return;
            };
            
            let name = structure.name().as_str();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let timestamp = format!("{:.3}", now.as_secs_f64());
            println!("[{}] New pad {} from source {}", timestamp, name, source_id);
            
            if !name.starts_with("video/") && !name.starts_with("image/") {
                return;
            }
            
            let pad_name = format!("sink_{}", source_id.0);
            
            // For compositor (Standard backend), we need to configure the pad properly
            let is_compositor = mux.factory()
                .map(|f| f.name() == "compositor")
                .unwrap_or(false);
            
            // Check if the pad already exists or request a new one
            let sinkpad = if is_compositor {
                // Compositor uses request pads without specific names
                match mux.request_pad_simple("sink_%u") {
                    Some(pad) => {
                        // Set position for this video on the compositor
                        // For now, just place videos side by side
                        let x_pos = (source_id.0 % 2) * 640;
                        let y_pos = (source_id.0 / 2) * 480;
                        pad.set_property("xpos", x_pos as i32);
                        pad.set_property("ypos", y_pos as i32);
                        pad
                    }
                    None => {
                        eprintln!("Failed to get compositor pad for source {}", source_id);
                        return;
                    }
                }
            } else {
                // For nvstreammux or other muxers
                match mux.static_pad(&pad_name)
                    .filter(|p| !p.is_linked())
                    .or_else(|| mux.request_pad_simple(&pad_name)) {
                    Some(pad) => pad,
                    None => {
                        eprintln!("Failed to get pad {} from mux", pad_name);
                        return;
                    }
                }
            };
            
            let now2 = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let timestamp2 = format!("{:.3}", now2.as_secs_f64());
            
            pad.link(&sinkpad)
                .map(|_| println!("[{}] Linked source {} to mux", timestamp2, source_id))
                .inspect_err(|e| eprintln!("[{}] Failed to link source {} to mux: {:?}", timestamp2, source_id, e))
                .ok();
        })
    }
    
    pub fn disconnect_pad_added(&mut self) {
        if let Some(handler_id) = self.pad_added_handler.take() {
            self.source_bin.disconnect(handler_id);
        }
    }
    
    pub fn element(&self) -> &gst::Element {
        &self.source_bin
    }
    
    pub fn set_state(&self, state: gst::State) -> Result<gst::StateChangeSuccess> {
        let result = self.source_bin.set_state(state);
        
        match result {
            Ok(success) => {
                let new_state = match state {
                    gst::State::Null => SourceState::Stopped,
                    gst::State::Ready => SourceState::Idle,
                    gst::State::Paused => SourceState::Paused,
                    gst::State::Playing => SourceState::Playing,
                    _ => SourceState::Idle,
                };
                
                if let Ok(mut state_guard) = self.state.lock() {
                    *state_guard = new_state;
                }
                
                Ok(success)
            }
            Err(e) => {
                if let Ok(mut state_guard) = self.state.lock() {
                    *state_guard = SourceState::Error(format!("State change failed: {:?}", e));
                }
                Err(DeepStreamError::StateChange(
                    format!("Failed to set state for source {}: {:?}", self.source_id, e)
                ))
            }
        }
    }
    
    pub fn get_state(&self, timeout: gst::ClockTime) -> Result<(gst::StateChangeSuccess, gst::State, gst::State)> {
        let (result, current, pending) = self.source_bin.state(timeout);
        result.map_err(|e| DeepStreamError::StateChange(
            format!("Failed to get state for source {}: {:?}", self.source_id, e)
        ))?;
        Ok((gst::StateChangeSuccess::Success, current, pending))
    }
    
    pub fn send_eos(&self) -> Result<()> {
        self.source_bin.send_event(gst::event::Eos::new());
        Ok(())
    }
    
    pub fn id(&self) -> SourceId {
        self.source_id
    }
    
    pub fn uri(&self) -> &str {
        &self.uri
    }
    
    pub fn current_state(&self) -> SourceState {
        self.state.lock()
            .map(|s| s.clone())
            .unwrap_or(SourceState::Error("Failed to lock state".to_string()))
    }
    
    pub fn update_state(&self, state: SourceState) -> Result<()> {
        let mut state_guard = self.state.lock()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock state".to_string()))?;
        *state_guard = state;
        Ok(())
    }
}

impl Drop for VideoSource {
    fn drop(&mut self) {
        self.disconnect_pad_added();
        let _ = self.set_state(gst::State::Null);
    }
}

pub fn create_uridecode_bin(source_id: SourceId, uri: &str) -> Result<VideoSource> {
    VideoSource::new(source_id, uri)
}

pub fn handle_pad_added(
    _decodebin: &gst::Element,
    pad: &gst::Pad,
    source_id: SourceId,
    streammux: &gst::Element,
) -> Result<()> {
    let caps = pad.current_caps().unwrap_or_else(|| pad.query_caps(None));
    let structure = caps.structure(0)
        .ok_or_else(|| DeepStreamError::Unknown("No caps structure".to_string()))?;
    let name = structure.name().as_str();
    
    println!("decodebin new pad {} for source {}", name, source_id);
    
    if name.starts_with("video/") || name.starts_with("image/") {
        let pad_name = format!("sink_{}", source_id.0);
        let sinkpad = streammux.request_pad_simple(&pad_name)
            .ok_or_else(|| DeepStreamError::PadLinking(
                format!("Failed to get request pad {} from streammux", pad_name)
            ))?;
        
        pad.link(&sinkpad)
            .map_err(|_| DeepStreamError::PadLinking(
                format!("Failed to link decodebin to streammux for source {}", source_id)
            ))?;
        
        println!("Decodebin linked to pipeline for source {}", source_id);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_video_source_creation() {
        gst::init().unwrap();
        
        let source_id = SourceId(0);
        let uri = "file:///tmp/test.mp4";
        
        let source = VideoSource::new(source_id, uri).unwrap();
        assert_eq!(source.id(), source_id);
        assert_eq!(source.uri(), uri);
        assert_eq!(source.current_state(), SourceState::Idle);
    }
    
    #[test]
    fn test_video_source_state_transitions() {
        gst::init().unwrap();
        
        let source_id = SourceId(1);
        let uri = "file:///tmp/test.mp4";
        
        let source = VideoSource::new(source_id, uri).unwrap();
        
        source.update_state(SourceState::Initializing).unwrap();
        assert_eq!(source.current_state(), SourceState::Initializing);
        
        source.update_state(SourceState::Playing).unwrap();
        assert_eq!(source.current_state(), SourceState::Playing);
    }
}