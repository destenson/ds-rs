use crate::error::{Result, SourceVideoError};
use crate::source::{VideoSource, SourceState};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::glib;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct LoopingVideoSource {
    inner_source: Box<dyn VideoSource>,
    loop_config: LoopConfig,
    loop_count: Arc<Mutex<u32>>,
    is_looping: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone)]
pub struct LoopConfig {
    pub max_loops: Option<u32>,
    pub loop_duration: Option<Duration>,
    pub seamless: bool,
    pub gap_duration: Duration,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_loops: None,
            loop_duration: None,
            seamless: true,
            gap_duration: Duration::from_millis(100),
        }
    }
}

impl LoopingVideoSource {
    pub fn new(inner_source: Box<dyn VideoSource>) -> Self {
        Self {
            inner_source,
            loop_config: LoopConfig::default(),
            loop_count: Arc::new(Mutex::new(0)),
            is_looping: Arc::new(Mutex::new(false)),
        }
    }
    
    pub fn with_config(mut self, config: LoopConfig) -> Self {
        self.loop_config = config;
        self
    }
    
    pub fn with_max_loops(mut self, max_loops: u32) -> Self {
        self.loop_config.max_loops = Some(max_loops);
        self
    }
    
    pub fn with_seamless_loop(mut self, seamless: bool) -> Self {
        self.loop_config.seamless = seamless;
        self
    }
    
    pub fn with_gap_duration(mut self, duration: Duration) -> Self {
        self.loop_config.gap_duration = duration;
        self
    }
    
    pub fn get_loop_count(&self) -> u32 {
        self.loop_count.lock().unwrap_or_else(|_| panic!("Lock poisoned")).clone()
    }
    
    pub fn reset_loop_count(&self) {
        if let Ok(mut count) = self.loop_count.lock() {
            *count = 0;
        }
    }
    
    pub fn is_looping_active(&self) -> bool {
        self.is_looping.lock().unwrap_or_else(|_| panic!("Lock poisoned")).clone()
    }
    
    fn setup_loop_handling(&mut self) -> Result<()> {
        if let Some(pipeline) = self.inner_source.get_pipeline() {
            let bus = pipeline.bus().ok_or_else(|| {
                SourceVideoError::pipeline("Failed to get pipeline bus for loop setup")
            })?;
            
            let loop_count = Arc::clone(&self.loop_count);
            let is_looping = Arc::clone(&self.is_looping);
            let max_loops = self.loop_config.max_loops;
            let seamless = self.loop_config.seamless;
            let gap_duration = self.loop_config.gap_duration;
            let pipeline_weak = pipeline.downgrade();
            
            bus.add_watch_local(move |_, msg| {
                use gst::MessageView;
                
                match msg.view() {
                    MessageView::Eos(_) => {
                        if let Some(pipeline) = pipeline_weak.upgrade() {
                            let mut should_continue = true;
                            
                            // Check loop count
                            if let Ok(mut count) = loop_count.lock() {
                                *count += 1;
                                
                                if let Some(max) = max_loops {
                                    if *count >= max {
                                        should_continue = false;
                                        log::info!("Reached maximum loop count: {}", max);
                                    }
                                }
                            }
                            
                            if should_continue {
                                if let Ok(mut looping) = is_looping.lock() {
                                    *looping = true;
                                }
                                
                                if seamless {
                                    // Seamless loop using segment seek
                                    if let Err(e) = pipeline.seek_simple(
                                        gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT | gst::SeekFlags::SEGMENT,
                                        gst::ClockTime::ZERO,
                                    ) {
                                        log::error!("Failed to seek for seamless loop: {:?}", e);
                                    } else {
                                        log::debug!("Seamless loop restart");
                                    }
                                } else {
                                    // Non-seamless loop with gap
                                    let gap_duration_clone = gap_duration;
                                    let pipeline_clone = pipeline.clone();
                                    
                                    glib::timeout_add_local(gap_duration_clone, move || {
                                        if let Err(e) = pipeline_clone.seek_simple(
                                            gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                                            gst::ClockTime::ZERO,
                                        ) {
                                            log::error!("Failed to seek for gapped loop: {:?}", e);
                                        } else {
                                            log::debug!("Gapped loop restart");
                                        }
                                        
                                        glib::ControlFlow::Break
                                    });
                                }
                            } else {
                                if let Ok(mut looping) = is_looping.lock() {
                                    *looping = false;
                                }
                                log::info!("Loop playback completed");
                            }
                        }
                    }
                    MessageView::Error(err) => {
                        log::error!("Pipeline error in looping source: {:?}", err.error());
                        if let Ok(mut looping) = is_looping.lock() {
                            *looping = false;
                        }
                    }
                    _ => {}
                }
                
                glib::ControlFlow::Continue
            }).map_err(|_| SourceVideoError::pipeline("Failed to add bus watch for looping"))?;
        }
        
        Ok(())
    }
}

impl VideoSource for LoopingVideoSource {
    fn get_id(&self) -> &str {
        self.inner_source.get_id()
    }
    
    fn get_name(&self) -> &str {
        self.inner_source.get_name()
    }
    
    fn get_uri(&self) -> String {
        self.inner_source.get_uri()
    }
    
    fn get_state(&self) -> SourceState {
        self.inner_source.get_state()
    }
    
    fn start(&mut self) -> Result<()> {
        self.inner_source.start()?;
        self.setup_loop_handling()?;
        
        if let Ok(mut looping) = self.is_looping.lock() {
            *looping = true;
        }
        
        log::info!("Started looping video source: {}", self.get_name());
        Ok(())
    }
    
    fn stop(&mut self) -> Result<()> {
        if let Ok(mut looping) = self.is_looping.lock() {
            *looping = false;
        }
        
        self.inner_source.stop()?;
        log::info!("Stopped looping video source: {}", self.get_name());
        Ok(())
    }
    
    fn pause(&mut self) -> Result<()> {
        self.inner_source.pause()
    }
    
    fn resume(&mut self) -> Result<()> {
        self.inner_source.resume()
    }
    
    fn get_pipeline(&self) -> Option<&gst::Pipeline> {
        self.inner_source.get_pipeline()
    }
}

pub struct GaplessLooper {
    segment_start: gst::ClockTime,
    segment_stop: Option<gst::ClockTime>,
    rate: f64,
}

impl GaplessLooper {
    pub fn new() -> Self {
        Self {
            segment_start: gst::ClockTime::ZERO,
            segment_stop: None,
            rate: 1.0,
        }
    }
    
    pub fn with_segment(mut self, start: gst::ClockTime, stop: Option<gst::ClockTime>) -> Self {
        self.segment_start = start;
        self.segment_stop = stop;
        self
    }
    
    pub fn with_rate(mut self, rate: f64) -> Self {
        self.rate = rate;
        self
    }
    
    pub fn apply_to_pipeline(&self, pipeline: &gst::Pipeline) -> Result<()> {
        let stop_time = self.segment_stop.unwrap_or(gst::ClockTime::from_seconds(u64::MAX));
        
        let seek_result = pipeline.seek(
            self.rate,
            gst::SeekFlags::FLUSH | gst::SeekFlags::SEGMENT,
            gst::SeekType::Set,
            self.segment_start,
            gst::SeekType::Set,
            stop_time,
        );
        
        if seek_result.is_ok() {
            log::debug!(
                "Applied gapless loop segment: {} to {}",
                self.segment_start,
                stop_time
            );
            Ok(())
        } else {
            Err(SourceVideoError::StateChange(
                "Failed to apply segment seek for gapless loop".to_string()
            ))
        }
    }
}

pub struct AutoRepeatManager {
    sources: Vec<Box<dyn VideoSource>>,
    global_config: LoopConfig,
}

impl AutoRepeatManager {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            global_config: LoopConfig::default(),
        }
    }
    
    pub fn with_global_config(mut self, config: LoopConfig) -> Self {
        self.global_config = config;
        self
    }
    
    pub fn add_source(&mut self, source: Box<dyn VideoSource>) {
        let looping_source = LoopingVideoSource::new(source).with_config(self.global_config.clone());
        self.sources.push(Box::new(looping_source));
    }
    
    pub fn add_looping_source(&mut self, source: LoopingVideoSource) {
        self.sources.push(Box::new(source));
    }
    
    pub async fn start_all(&mut self) -> Result<()> {
        for source in &mut self.sources {
            source.start()?;
        }
        
        log::info!("Started {} auto-repeat sources", self.sources.len());
        Ok(())
    }
    
    pub async fn stop_all(&mut self) -> Result<()> {
        for source in &mut self.sources {
            source.stop()?;
        }
        
        log::info!("Stopped {} auto-repeat sources", self.sources.len());
        Ok(())
    }
    
    pub fn get_sources(&self) -> &[Box<dyn VideoSource>] {
        &self.sources
    }
    
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }
    
    pub fn get_total_loop_count(&self) -> u32 {
        self.sources
            .iter()
            .filter_map(|source| {
                // Try to downcast to LoopingVideoSource
                // This is a simplified approach - in practice you'd need proper trait design
                None::<u32>
            })
            .sum()
    }
}

impl Default for AutoRepeatManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_looping_source(
    inner: Box<dyn VideoSource>,
    max_loops: Option<u32>,
    seamless: bool,
) -> LoopingVideoSource {
    let config = LoopConfig {
        max_loops,
        seamless,
        ..Default::default()
    };
    
    LoopingVideoSource::new(inner).with_config(config)
}

pub fn enable_auto_repeat_for_source(
    source: &mut Box<dyn VideoSource>,
    config: Option<LoopConfig>,
) -> Result<()> {
    if let Some(pipeline) = source.get_pipeline() {
        let loop_config = config.unwrap_or_default();
        
        let bus = pipeline.bus().ok_or_else(|| {
            SourceVideoError::pipeline("No bus available for auto-repeat setup")
        })?;
        
        let seamless = loop_config.seamless;
        let max_loops = loop_config.max_loops;
        let pipeline_weak = pipeline.downgrade();
        let loop_count = Arc::new(Mutex::new(0u32));
        
        bus.add_watch_local(move |_, msg| {
            use gst::MessageView;
            
            if let MessageView::Eos(_) = msg.view() {
                if let Some(pipeline) = pipeline_weak.upgrade() {
                    let should_continue = if let Some(max) = max_loops {
                        if let Ok(mut count) = loop_count.lock() {
                            *count += 1;
                            *count < max
                        } else {
                            false
                        }
                    } else {
                        true
                    };
                    
                    if should_continue {
                        let seek_flags = if seamless {
                            gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT | gst::SeekFlags::SEGMENT
                        } else {
                            gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT
                        };
                        
                        if let Err(e) = pipeline.seek_simple(seek_flags, gst::ClockTime::ZERO) {
                            log::error!("Failed to restart playback for auto-repeat: {:?}", e);
                        }
                    }
                }
            }
            
            glib::ControlFlow::Continue
        }).map_err(|_| SourceVideoError::pipeline("Failed to add auto-repeat watch"))?;
        
        log::info!("Enabled auto-repeat for source: {}", source.get_name());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::SourceState;
    use gstreamer as gst;
    
    struct MockVideoSource {
        id: String,
        name: String,
        state: SourceState,
    }
    
    impl MockVideoSource {
        fn new(id: String, name: String) -> Self {
            Self {
                id,
                name,
                state: SourceState::Created,
            }
        }
    }
    
    impl VideoSource for MockVideoSource {
        fn get_id(&self) -> &str {
            &self.id
        }
        
        fn get_name(&self) -> &str {
            &self.name
        }
        
        fn get_uri(&self) -> String {
            format!("mock:///{}", self.name)
        }
        
        fn get_state(&self) -> SourceState {
            self.state.clone()
        }
        
        fn start(&mut self) -> Result<()> {
            self.state = SourceState::Playing;
            Ok(())
        }
        
        fn stop(&mut self) -> Result<()> {
            self.state = SourceState::Stopped;
            Ok(())
        }
        
        fn pause(&mut self) -> Result<()> {
            self.state = SourceState::Paused;
            Ok(())
        }
        
        fn resume(&mut self) -> Result<()> {
            self.state = SourceState::Playing;
            Ok(())
        }
        
        fn get_pipeline(&self) -> Option<&gst::Pipeline> {
            None
        }
    }
    
    #[test]
    fn test_looping_video_source_creation() {
        let mock_source = Box::new(MockVideoSource::new("test-id".to_string(), "test-source".to_string()));
        let looping_source = LoopingVideoSource::new(mock_source);
        
        assert_eq!(looping_source.get_id(), "test-id");
        assert_eq!(looping_source.get_name(), "test-source");
        assert_eq!(looping_source.get_loop_count(), 0);
        assert!(!looping_source.is_looping_active());
    }
    
    #[test]
    fn test_loop_config() {
        let config = LoopConfig {
            max_loops: Some(5),
            seamless: false,
            gap_duration: Duration::from_millis(200),
            ..Default::default()
        };
        
        let mock_source = Box::new(MockVideoSource::new("test-id".to_string(), "test-source".to_string()));
        let looping_source = LoopingVideoSource::new(mock_source).with_config(config.clone());
        
        assert_eq!(looping_source.loop_config.max_loops, Some(5));
        assert!(!looping_source.loop_config.seamless);
        assert_eq!(looping_source.loop_config.gap_duration, Duration::from_millis(200));
    }
    
    #[test]
    fn test_auto_repeat_manager() {
        let mut manager = AutoRepeatManager::new();
        
        let mock_source = Box::new(MockVideoSource::new("test-1".to_string(), "source-1".to_string()));
        manager.add_source(mock_source);
        
        assert_eq!(manager.source_count(), 1);
    }
    
    #[test]
    fn test_gapless_looper() {
        let looper = GaplessLooper::new()
            .with_segment(gst::ClockTime::from_seconds(5), Some(gst::ClockTime::from_seconds(30)))
            .with_rate(1.5);
        
        assert_eq!(looper.segment_start, gst::ClockTime::from_seconds(5));
        assert_eq!(looper.segment_stop, Some(gst::ClockTime::from_seconds(30)));
        assert_eq!(looper.rate, 1.5);
    }
}