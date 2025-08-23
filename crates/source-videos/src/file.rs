use crate::config::{VideoSourceConfig, FileContainer};
use crate::error::{Result, SourceVideoError};
use crate::patterns::TestPattern;
use crate::pipeline::builder::{PipelineBuilder, ElementBuilder, CapsBuilder};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct FileGenerator {
    config: VideoSourceConfig,
    output_path: PathBuf,
    pipeline: Option<gst::Pipeline>,
    bus_watch: Option<gst::bus::BusWatchGuard>,
    completion: Arc<Mutex<Option<Result<()>>>>,
}

impl FileGenerator {
    pub fn new(config: VideoSourceConfig, output_path: impl AsRef<Path>) -> Self {
        Self {
            config,
            output_path: output_path.as_ref().to_path_buf(),
            pipeline: None,
            bus_watch: None,
            completion: Arc::new(Mutex::new(None)),
        }
    }
    
    pub fn generate(&mut self) -> Result<()> {
        self.create_pipeline()?;
        self.setup_bus_watch();
        self.start_pipeline()?;
        self.wait_for_completion()
    }
    
    pub fn generate_async(&mut self) -> Result<()> {
        self.create_pipeline()?;
        self.setup_bus_watch();
        self.start_pipeline()
    }
    
    fn create_pipeline(&mut self) -> Result<()> {
        let pipeline_name = format!("file-gen-{}", self.config.name);
        let mut builder = PipelineBuilder::new(pipeline_name);
        
        let src = ElementBuilder::videotestsrc(Some("source"))?;
        
        if let crate::config::VideoSourceType::TestPattern { pattern } = &self.config.source_type {
            let pattern = TestPattern::from_str(pattern)?;
            src.set_property("pattern", pattern.to_gst_pattern());
        } else {
            src.set_property("pattern", 0i32);
        }
        
        if let Some(num_buffers) = self.config.num_buffers {
            src.set_property("num-buffers", num_buffers);
        } else if let Some(duration) = self.config.duration {
            let num_buffers = (duration * self.config.framerate.numerator as u64) 
                / self.config.framerate.denominator as u64;
            src.set_property("num-buffers", num_buffers as i32);
        }
        
        let caps = CapsBuilder::video_raw(
            self.config.resolution.width,
            self.config.resolution.height,
            (self.config.framerate.numerator, self.config.framerate.denominator),
            self.config.format.to_caps_string(),
        );
        
        let capsfilter = ElementBuilder::capsfilter(Some("filter"), &caps)?;
        let videoconvert = ElementBuilder::videoconvert(Some("convert"))?;
        
        let (encoder, muxer) = self.create_encoder_muxer()?;
        
        let filesink = ElementBuilder::filesink(
            Some("sink"),
            &self.output_path.to_string_lossy(),
        )?;
        
        builder = builder
            .add_many(vec![
                src.clone(),
                capsfilter.clone(),
                videoconvert.clone(),
                encoder.clone(),
                muxer.clone(),
                filesink.clone(),
            ])?
            .link_elements(&src, &capsfilter)?
            .link_elements(&capsfilter, &videoconvert)?
            .link_elements(&videoconvert, &encoder)?
            .link_elements(&encoder, &muxer)?
            .link_elements(&muxer, &filesink)?;
        
        self.pipeline = Some(builder.build());
        Ok(())
    }
    
    fn create_encoder_muxer(&self) -> Result<(gst::Element, gst::Element)> {
        let container = if let crate::config::VideoSourceType::File { container, .. } = &self.config.source_type {
            container
        } else {
            &FileContainer::Mp4
        };
        
        let encoder = match container {
            FileContainer::Mp4 | FileContainer::Mkv | FileContainer::Avi => {
                let enc = ElementBuilder::x264enc(Some("encoder"))?;
                enc.set_property("speed-preset", "ultrafast");
                enc.set_property("tune", "zerolatency");
                enc
            }
            FileContainer::WebM => {
                gst::ElementFactory::make("vp8enc")
                    .name("encoder")
                    .build()
                    .map_err(|_| SourceVideoError::element("vp8enc"))?
            }
        };
        
        let muxer = gst::ElementFactory::make(container.muxer_name())
            .name("muxer")
            .build()
            .map_err(|_| SourceVideoError::element(container.muxer_name()))?;
        
        Ok((encoder, muxer))
    }
    
    fn setup_bus_watch(&mut self) {
        if let Some(pipeline) = &self.pipeline {
            let bus = pipeline.bus().expect("Pipeline should have a bus");
            let completion = Arc::clone(&self.completion);
            
            let watch = bus.add_watch(move |_bus, msg| {
                use gst::MessageView;
                
                match msg.view() {
                    MessageView::Eos(_) => {
                        log::info!("File generation completed");
                        if let Ok(mut comp) = completion.lock() {
                            *comp = Some(Ok(()));
                        }
                        gst::glib::ControlFlow::Break
                    }
                    MessageView::Error(err) => {
                        let error_msg = format!(
                            "File generation error from {:?}: {} ({:?})",
                            err.src().map(|s| s.path_string()),
                            err.error(),
                            err.debug()
                        );
                        log::error!("{}", error_msg);
                        if let Ok(mut comp) = completion.lock() {
                            *comp = Some(Err(SourceVideoError::pipeline(error_msg)));
                        }
                        gst::glib::ControlFlow::Break
                    }
                    MessageView::Warning(warn) => {
                        log::warn!(
                            "Warning from {:?}: {} ({:?})",
                            warn.src().map(|s| s.path_string()),
                            warn.error(),
                            warn.debug()
                        );
                        gst::glib::ControlFlow::Continue
                    }
                    _ => gst::glib::ControlFlow::Continue,
                }
            })
            .expect("Failed to add bus watch");
            
            self.bus_watch = Some(watch);
        }
    }
    
    fn start_pipeline(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Playing)
                .map_err(|_| SourceVideoError::StateChange("Failed to start file generation".to_string()))?;
            Ok(())
        } else {
            Err(SourceVideoError::pipeline("Pipeline not created"))
        }
    }
    
    fn wait_for_completion(&self) -> Result<()> {
        let timeout = Duration::from_secs(self.config.duration.unwrap_or(60) + 10);
        let start = std::time::Instant::now();
        
        loop {
            if let Ok(comp) = self.completion.lock() {
                if let Some(result) = &*comp {
                    match result {
                        Ok(_) => return Ok(()),
                        Err(e) => return Err(SourceVideoError::pipeline(e.to_string())),
                    }
                }
            }
            
            if start.elapsed() > timeout {
                return Err(SourceVideoError::Timeout(timeout.as_secs()));
            }
            
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    
    pub fn stop(&mut self) -> Result<()> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Null)
                .map_err(|_| SourceVideoError::StateChange("Failed to stop pipeline".to_string()))?;
        }
        
        self.bus_watch = None;
        self.pipeline = None;
        Ok(())
    }
}

pub struct BatchFileGenerator {
    configs: Vec<(VideoSourceConfig, PathBuf)>,
}

impl BatchFileGenerator {
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
        }
    }
    
    pub fn add(&mut self, config: VideoSourceConfig, output_path: impl AsRef<Path>) {
        self.configs.push((config, output_path.as_ref().to_path_buf()));
    }
    
    pub fn generate_all(&self) -> Result<Vec<PathBuf>> {
        let mut generated_files = Vec::new();
        
        for (config, path) in &self.configs {
            log::info!("Generating file: {}", path.display());
            
            let mut generator = FileGenerator::new(config.clone(), path);
            generator.generate()?;
            
            generated_files.push(path.clone());
            log::info!("Successfully generated: {}", path.display());
        }
        
        Ok(generated_files)
    }
    
    pub fn generate_parallel(&self, max_parallel: usize) -> Result<Vec<PathBuf>> {
        use std::sync::mpsc;
        use std::thread;
        
        let (tx, rx) = mpsc::channel();
        let configs = Arc::new(self.configs.clone());
        let mut handles = Vec::new();
        
        let chunks: Vec<_> = configs.chunks(max_parallel)
            .map(|chunk| chunk.to_vec())
            .collect();
        
        for chunk in chunks {
            let tx = tx.clone();
            let handle = thread::spawn(move || {
                for (config, path) in chunk {
                    let result = FileGenerator::new(config, &path).generate();
                    tx.send((path, result)).unwrap();
                }
            });
            handles.push(handle);
        }
        
        drop(tx);
        
        let mut generated_files = Vec::new();
        for (path, result) in rx {
            result?;
            generated_files.push(path);
        }
        
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        
        Ok(generated_files)
    }
}

pub fn generate_test_file(
    pattern: &str,
    duration: u64,
    output_path: impl AsRef<Path>,
) -> Result<()> {
    let mut config = VideoSourceConfig::test_pattern("test-gen", pattern);
    config.duration = Some(duration);
    config.source_type = crate::config::VideoSourceType::File {
        path: output_path.as_ref().to_string_lossy().to_string(),
        container: FileContainer::Mp4,
    };
    
    let mut generator = FileGenerator::new(config, output_path);
    generator.generate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_file_generator_creation() {
        gst::init().unwrap();
        
        let config = VideoSourceConfig::test_pattern("test", "smpte");
        let generator = FileGenerator::new(config, "/tmp/test.mp4");
        
        assert_eq!(generator.output_path, PathBuf::from("/tmp/test.mp4"));
    }
    
    #[test]
    fn test_batch_generator() {
        let mut batch = BatchFileGenerator::new();
        
        let config1 = VideoSourceConfig::test_pattern("test1", "smpte");
        let config2 = VideoSourceConfig::test_pattern("test2", "ball");
        
        batch.add(config1, "/tmp/test1.mp4");
        batch.add(config2, "/tmp/test2.mp4");
        
        assert_eq!(batch.configs.len(), 2);
    }
}