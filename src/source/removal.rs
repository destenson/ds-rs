use crate::error::{DeepStreamError, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::time::Duration;
use std::thread;
use super::{SourceId, SourceManager, SourceState};

pub trait SourceRemoval {
    fn remove_video_source(&self, id: SourceId) -> Result<()>;
    fn remove_all_sources(&self) -> Result<()>;
    fn stop_and_remove_source(&self, id: SourceId) -> Result<()>;
}

impl SourceRemoval for SourceManager {
    fn remove_video_source(&self, id: SourceId) -> Result<()> {
        self.stop_and_remove_source(id)
    }
    
    fn remove_all_sources(&self) -> Result<()> {
        let source_ids = self.list_sources()?;
        
        for id in source_ids {
            if let Err(e) = self.remove_video_source(id) {
                eprintln!("Failed to remove source {}: {:?}", id, e);
            }
        }
        
        Ok(())
    }
    
    fn stop_and_remove_source(&self, id: SourceId) -> Result<()> {
        let pipeline = self.get_pipeline()
            .ok_or_else(|| DeepStreamError::NotInitialized("Pipeline not set".to_string()))?;
        
        let streammux = self.get_streammux()
            .ok_or_else(|| DeepStreamError::NotInitialized("Streammux not set".to_string()))?;
        
        println!("Stopping and removing source {}", id);
        
        self.update_source_state(id, SourceState::Stopping)?;
        
        let source_info = self.get_source_info(id)?;
        let source = &source_info.source;
        
        let state_result = source.set_state(gst::State::Null)?;
        
        match state_result {
            gst::StateChangeSuccess::Success => {
                println!("Source {} state change to NULL: SUCCESS", id);
                perform_source_cleanup(&pipeline, streammux, id, source)?;
            }
            gst::StateChangeSuccess::Async => {
                println!("Source {} state change to NULL: ASYNC", id);
                thread::sleep(Duration::from_millis(100));
                perform_source_cleanup(&pipeline, streammux, id, source)?;
            }
            gst::StateChangeSuccess::NoPreroll => {
                println!("Source {} state change to NULL: NO PREROLL", id);
                perform_source_cleanup(&pipeline, streammux, id, source)?;
            }
        }
        
        self.remove_source(id)?;
        
        println!("Successfully removed source {} - Remaining sources: {}", 
                 id, self.num_sources()?);
        
        Ok(())
    }
}

fn perform_source_cleanup(
    pipeline: &crate::pipeline::Pipeline,
    streammux: &gst::Element,
    source_id: SourceId,
    source: &super::VideoSource,
) -> Result<()> {
    let pad_name = format!("sink_{}", source_id.0);
    
    if let Some(sinkpad) = streammux.static_pad(&pad_name) {
        sinkpad.send_event(gst::event::FlushStop::builder(false).build());
        
        if let Some(peer) = sinkpad.peer() {
            peer.unlink(&sinkpad)?;
        }
        
        println!("Released pad {} from streammux", pad_name);
    } else if let Some(sinkpad) = streammux.request_pad_simple(&pad_name) {
        sinkpad.send_event(gst::event::FlushStop::builder(false).build());
        
        if let Some(peer) = sinkpad.peer() {
            peer.unlink(&sinkpad)?;
        }
        
        streammux.release_request_pad(&sinkpad);
        println!("Released request pad {} from streammux", pad_name);
    }
    
    pipeline.remove_element(source.element())?;
    
    source.update_state(SourceState::Stopped)?;
    
    Ok(())
}

pub struct RemovalConfig {
    pub force: bool,
    pub timeout: Duration,
    pub send_eos: bool,
}

impl Default for RemovalConfig {
    fn default() -> Self {
        Self {
            force: false,
            timeout: Duration::from_secs(5),
            send_eos: true,
        }
    }
}

impl RemovalConfig {
    pub fn force() -> Self {
        Self {
            force: true,
            timeout: Duration::from_millis(100),
            send_eos: false,
        }
    }
    
    pub fn graceful() -> Self {
        Self {
            force: false,
            timeout: Duration::from_secs(10),
            send_eos: true,
        }
    }
}

pub fn remove_source_with_config(
    manager: &SourceManager,
    source_id: SourceId,
    config: &RemovalConfig,
) -> Result<()> {
    if !manager.is_source_enabled(source_id)? {
        return Err(DeepStreamError::InvalidInput(
            format!("Source {} is not enabled", source_id)
        ));
    }
    
    let source = manager.get_source(source_id)?;
    
    if config.send_eos && !config.force {
        source.send_eos()?;
        thread::sleep(Duration::from_millis(500));
    }
    
    if config.force {
        let _ = source.set_state(gst::State::Null);
        thread::sleep(Duration::from_millis(100));
    } else {
        let start = std::time::Instant::now();
        loop {
            match source.set_state(gst::State::Null) {
                Ok(_) => break,
                Err(_) if start.elapsed() >= config.timeout => {
                    if config.force {
                        break;
                    } else {
                        return Err(DeepStreamError::Timeout(
                            format!("Timeout removing source {}", source_id)
                        ));
                    }
                }
                _ => thread::sleep(Duration::from_millis(100)),
            }
        }
    }
    
    manager.stop_and_remove_source(source_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_removal_config() {
        let default_config = RemovalConfig::default();
        assert!(!default_config.force);
        assert!(default_config.send_eos);
        assert_eq!(default_config.timeout, Duration::from_secs(5));
        
        let force_config = RemovalConfig::force();
        assert!(force_config.force);
        assert!(!force_config.send_eos);
        assert_eq!(force_config.timeout, Duration::from_millis(100));
        
        let graceful_config = RemovalConfig::graceful();
        assert!(!graceful_config.force);
        assert!(graceful_config.send_eos);
        assert_eq!(graceful_config.timeout, Duration::from_secs(10));
    }
}