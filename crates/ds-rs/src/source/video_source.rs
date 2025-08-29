use super::{SourceId, SourceState};
use crate::error::{DeepStreamError, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};

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
            pad_added_handler: None, // Don't clone signal handlers
        }
    }
}

impl VideoSource {
    pub fn new(source_id: SourceId, uri: &str) -> Result<Self> {
        let bin_name = format!("source-bin-{:02}", source_id.0);

        // Handle special test source URI
        let (source_bin, final_uri) = if uri == "videotestsrc://" {
            // Create a bin with videotestsrc for testing
            let bin = gst::Bin::builder().name(&bin_name).build();

            let src = gst::ElementFactory::make("videotestsrc")
                .name(&format!("testsrc-{}", source_id.0))
                .property_from_str("pattern", "ball") // Ball pattern
                .property("is-live", true)
                .build()
                .map_err(|_| DeepStreamError::ElementCreation {
                    element: format!("videotestsrc for source {}", source_id),
                })?;

            let capsfilter = gst::ElementFactory::make("capsfilter")
                .name(&format!("testcaps-{}", source_id.0))
                .build()
                .map_err(|_| DeepStreamError::ElementCreation {
                    element: format!("capsfilter for source {}", source_id),
                })?;

            let caps = gst::Caps::builder("video/x-raw")
                .field("width", 640i32)
                .field("height", 480i32)
                .field("framerate", gst::Fraction::new(30, 1))
                .build();
            capsfilter.set_property("caps", &caps);

            bin.add_many([&src, &capsfilter])?;
            src.link(&capsfilter)?;

            // Create ghost pad
            let src_pad = capsfilter.static_pad("src").unwrap();
            let ghost_pad = gst::GhostPad::with_target(&src_pad)?;
            ghost_pad.set_active(true)?;
            bin.add_pad(&ghost_pad)?;

            (bin.upcast(), uri.to_string())
        } else {
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
                    element: format!("uridecodebin for source {}", source_id),
                })?;

            (source_bin, fixed_uri)
        };

        Ok(Self {
            source_bin,
            source_id,
            uri: final_uri,
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
            println!(
                "[{}] pad-added callback triggered for source {} (pad: {})",
                timestamp,
                source_id,
                pad.name()
            );

            if let Some(streammux) = streammux_weak.upgrade() {
                callback(decodebin, pad, source_id, &streammux);
            } else {
                eprintln!(
                    "[{}] Failed to upgrade streammux weak reference for source {}",
                    timestamp, source_id
                );
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
        println!(
            "[{}] Connecting pad-added callback for source {}",
            timestamp, self.source_id
        );

        // For test sources (videotestsrc://), we don't need pad-added callback
        // We'll handle the connection after the element is added to the pipeline
        if self.uri == "videotestsrc://" {
            // Don't set up callback for test sources
            return Ok(());
        }

        self.connect_pad_added(streammux, |decodebin, pad, source_id, mux| {
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
            let is_compositor = mux
                .factory()
                .map(|f| f.name() == "compositor")
                .unwrap_or(false);

            // For compositor, we need to add videorate and capsfilter to normalize framerate
            // This fixes the H264 parser warning about excessive framerate
            if is_compositor {
                // Get the parent pipeline
                let pipeline = mux
                    .parent()
                    .and_then(|p| p.downcast::<gst::Pipeline>().ok())
                    .or_else(|| {
                        // Try to get pipeline from decodebin parent
                        decodebin
                            .parent()
                            .and_then(|p| p.downcast::<gst::Pipeline>().ok())
                    });

                let Some(pipeline) = pipeline else {
                    eprintln!("Failed to get pipeline for source {}", source_id);
                    return;
                };

                // Create elements to normalize framerate
                let videorate = match gst::ElementFactory::make("videorate")
                    .name(&format!("videorate-{}", source_id.0))
                    .build()
                {
                    Ok(e) => e,
                    Err(_) => {
                        eprintln!("Failed to create videorate for source {}", source_id);
                        return;
                    }
                };

                let capsfilter = match gst::ElementFactory::make("capsfilter")
                    .name(&format!("capsfilter-{}", source_id.0))
                    .build()
                {
                    Ok(e) => e,
                    Err(_) => {
                        eprintln!("Failed to create capsfilter for source {}", source_id);
                        return;
                    }
                };

                // Set caps to normalize framerate to 30fps
                let filtercaps = gst::Caps::builder("video/x-raw")
                    .field("framerate", gst::Fraction::new(30, 1))
                    .build();
                capsfilter.set_property("caps", &filtercaps);

                // Add elements to pipeline
                if let Err(e) = pipeline.add_many([&videorate, &capsfilter]) {
                    eprintln!("Failed to add framerate elements to pipeline: {:?}", e);
                    return;
                }

                // Link videorate -> capsfilter
                if let Err(e) = videorate.link(&capsfilter) {
                    eprintln!("Failed to link videorate to capsfilter: {:?}", e);
                    return;
                }

                // Sync state with parent
                videorate.sync_state_with_parent().ok();
                capsfilter.sync_state_with_parent().ok();

                // Link decoder pad to videorate
                let videorate_sink = match videorate.static_pad("sink") {
                    Some(pad) => pad,
                    None => {
                        eprintln!("Failed to get videorate sink pad for source {}", source_id);
                        return;
                    }
                };
                if let Err(e) = pad.link(&videorate_sink) {
                    eprintln!("Failed to link decoder to videorate: {:?}", e);
                    return;
                }

                // Get compositor sink pad
                let sinkpad = match mux.request_pad_simple("sink_%u") {
                    Some(pad) => {
                        // Set position for this video on the compositor
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
                };

                // Link capsfilter to compositor
                let capsfilter_src = match capsfilter.static_pad("src") {
                    Some(pad) => pad,
                    None => {
                        eprintln!("Failed to get capsfilter src pad for source {}", source_id);
                        return;
                    }
                };
                if let Err(e) = capsfilter_src.link(&sinkpad) {
                    eprintln!("Failed to link capsfilter to compositor: {:?}", e);
                    return;
                }

                let now2 = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                let timestamp2 = format!("{:.3}", now2.as_secs_f64());
                println!(
                    "[{}] Linked source {} through framerate normalizer to compositor",
                    timestamp2, source_id
                );

                return;
            }

            // Check if the pad already exists or request a new one
            let sinkpad = if is_compositor {
                // This should not be reached anymore due to early return above
                eprintln!("Unexpected: compositor path reached after framerate handling");
                return;
            } else {
                // For nvstreammux or other muxers
                match mux
                    .static_pad(&pad_name)
                    .filter(|p| !p.is_linked())
                    .or_else(|| mux.request_pad_simple(&pad_name))
                {
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

            let link_result = pad.link(&sinkpad);
            match link_result {
                Ok(_) => {
                    println!(
                        "[{}] Linked source {} to mux successfully",
                        timestamp2, source_id
                    );

                    // Check if data is flowing by adding buffer and event probes
                    let source_id_probe = source_id;
                    pad.add_probe(gst::PadProbeType::BUFFER, move |pad, _info| {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default();
                        let timestamp = format!("{:.3}", now.as_secs_f64());
                        println!(
                            "[{}] Buffer flowing through source {} pad {}",
                            timestamp,
                            source_id_probe,
                            pad.name()
                        );
                        gst::PadProbeReturn::Ok
                    });

                    // Also add caps probe to see negotiation issues
                    let source_id_caps = source_id;
                    pad.add_probe(gst::PadProbeType::EVENT_DOWNSTREAM, move |_pad, info| {
                        if let Some(gst::PadProbeData::Event(ref event)) = info.data {
                            if let gst::EventView::Caps(caps_event) = event.view() {
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default();
                                let timestamp = format!("{:.3}", now.as_secs_f64());
                                println!(
                                    "[{}] Caps negotiated for source {}: {}",
                                    timestamp,
                                    source_id_caps,
                                    caps_event.caps()
                                );
                            }
                        }
                        gst::PadProbeReturn::Ok
                    });
                }
                Err(e) => {
                    eprintln!(
                        "[{}] Failed to link source {} to mux: {:?}",
                        timestamp2, source_id, e
                    );
                }
            }
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
                Err(DeepStreamError::StateChange(format!(
                    "Failed to set state for source {}: {:?}",
                    self.source_id, e
                )))
            }
        }
    }

    pub fn get_state(
        &self,
        timeout: gst::ClockTime,
    ) -> Result<(gst::StateChangeSuccess, gst::State, gst::State)> {
        let (result, current, pending) = self.source_bin.state(timeout);
        result.map_err(|e| {
            DeepStreamError::StateChange(format!(
                "Failed to get state for source {}: {:?}",
                self.source_id, e
            ))
        })?;
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
        self.state
            .lock()
            .map(|s| s.clone())
            .unwrap_or(SourceState::Error("Failed to lock state".to_string()))
    }

    pub fn update_state(&self, state: SourceState) -> Result<()> {
        let mut state_guard = self
            .state
            .lock()
            .map_err(|_| DeepStreamError::Unknown("Failed to lock state".to_string()))?;
        *state_guard = state;
        Ok(())
    }

    /// Connect test sources to the muxer after being added to pipeline
    pub fn connect_test_source(&self, streammux: &gst::Element) -> Result<()> {
        if self.uri != "videotestsrc://" {
            return Ok(()); // Not a test source
        }

        let Some(src_pad) = self.source_bin.static_pad("src") else {
            return Err(DeepStreamError::Pipeline(
                "Test source has no src pad".to_string(),
            ));
        };

        // For compositor (Standard backend), request a pad and configure position
        let is_compositor = streammux
            .factory()
            .map(|f| f.name() == "compositor")
            .unwrap_or(false);

        if is_compositor {
            if let Some(sinkpad) = streammux.request_pad_simple("sink_%u") {
                // Set position for this video on the compositor
                let x_pos = (self.source_id.0 % 2) * 640;
                let y_pos = (self.source_id.0 / 2) * 480;
                sinkpad.set_property("xpos", x_pos as i32);
                sinkpad.set_property("ypos", y_pos as i32);

                if let Err(e) = src_pad.link(&sinkpad) {
                    return Err(DeepStreamError::Pipeline(format!(
                        "Failed to link test source to compositor: {:?}",
                        e
                    )));
                }
                println!(
                    "[{:.3}] Linked test source {} to compositor",
                    crate::timestamp(),
                    self.source_id
                );
            }
        } else {
            // For other muxers, use normal pad naming
            let pad_name = format!("sink_{}", self.source_id.0);
            if let Some(sinkpad) = streammux.request_pad_simple(&pad_name) {
                if let Err(e) = src_pad.link(&sinkpad) {
                    return Err(DeepStreamError::Pipeline(format!(
                        "Failed to link test source to mux: {:?}",
                        e
                    )));
                }
            }
        }

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
    let structure = caps
        .structure(0)
        .ok_or_else(|| DeepStreamError::Unknown("No caps structure".to_string()))?;
    let name = structure.name().as_str();

    println!("decodebin new pad {} for source {}", name, source_id);

    if name.starts_with("video/") || name.starts_with("image/") {
        let pad_name = format!("sink_{}", source_id.0);
        let sinkpad = streammux.request_pad_simple(&pad_name).ok_or_else(|| {
            DeepStreamError::PadLinking(format!(
                "Failed to get request pad {} from streammux",
                pad_name
            ))
        })?;

        pad.link(&sinkpad).map_err(|_| {
            DeepStreamError::PadLinking(format!(
                "Failed to link decodebin to streammux for source {}",
                source_id
            ))
        })?;

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
