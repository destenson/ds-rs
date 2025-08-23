use crate::error::Result;
use crate::pipeline::Pipeline;
use gstreamer as gst;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use super::{
    SourceId, SourceManager, SourceState, SourceAddition, SourceRemoval,
    SourceEventHandler, SourceEvent, SourceSynchronizer, events::EosTracker
};

pub struct SourceController {
    manager: Arc<SourceManager>,
    event_handler: Arc<SourceEventHandler>,
    synchronizer: Arc<SourceSynchronizer>,
    eos_tracker: Arc<EosTracker>,
    auto_remove_on_eos: bool,
}

impl SourceController {
    pub fn new(pipeline: Arc<Pipeline>, streammux: gst::Element) -> Self {
        let mut manager = SourceManager::with_defaults();
        manager.set_pipeline(pipeline);
        manager.set_streammux(streammux);
        
        let manager = Arc::new(manager);
        let synchronizer = Arc::new(SourceSynchronizer::new(manager.clone()));
        
        Self {
            manager: manager.clone(),
            event_handler: Arc::new(SourceEventHandler::new()),
            synchronizer,
            eos_tracker: Arc::new(EosTracker::new(super::MAX_NUM_SOURCES)),
            auto_remove_on_eos: false,
        }
    }
    
    pub fn with_max_sources(
        pipeline: Arc<Pipeline>, 
        streammux: gst::Element,
        max_sources: usize
    ) -> Self {
        let mut manager = SourceManager::new(max_sources);
        manager.set_pipeline(pipeline);
        manager.set_streammux(streammux);
        
        let manager = Arc::new(manager);
        let synchronizer = Arc::new(SourceSynchronizer::new(manager.clone()));
        
        Self {
            manager: manager.clone(),
            event_handler: Arc::new(SourceEventHandler::new()),
            synchronizer,
            eos_tracker: Arc::new(EosTracker::new(max_sources)),
            auto_remove_on_eos: false,
        }
    }
    
    pub fn add_source(&self, uri: &str) -> Result<SourceId> {
        let id = self.manager.add_video_source(uri)?;
        
        self.event_handler.emit(SourceEvent::SourceAdded {
            id,
            uri: uri.to_string(),
        })?;
        
        self.synchronizer.sync_source_with_pipeline(id)?;
        
        Ok(id)
    }
    
    pub fn remove_source(&self, id: SourceId) -> Result<()> {
        self.manager.remove_video_source(id)?;
        
        self.event_handler.emit(SourceEvent::SourceRemoved { id })?;
        self.eos_tracker.clear_eos(id)?;
        
        Ok(())
    }
    
    pub fn add_sources_batch(&self, uris: &[String]) -> Result<Vec<SourceId>> {
        let mut ids = Vec::new();
        
        for uri in uris {
            match self.add_source(uri) {
                Ok(id) => ids.push(id),
                Err(e) => {
                    eprintln!("Failed to add source {}: {:?}", uri, e);
                    for &id in &ids {
                        let _ = self.remove_source(id);
                    }
                    return Err(e);
                }
            }
        }
        
        Ok(ids)
    }
    
    pub fn remove_all_sources(&self) -> Result<()> {
        self.manager.remove_all_sources()?;
        Ok(())
    }
    
    pub fn list_active_sources(&self) -> Result<Vec<(SourceId, String, SourceState)>> {
        let source_ids = self.manager.list_sources()?;
        let mut result = Vec::new();
        
        for id in source_ids {
            if let Ok(info) = self.manager.get_source_info(id) {
                result.push((id, info.uri, info.state));
            }
        }
        
        Ok(result)
    }
    
    pub fn get_source_state(&self, id: SourceId) -> Result<SourceState> {
        let info = self.manager.get_source_info(id)?;
        Ok(info.state)
    }
    
    pub fn set_source_state(&self, id: SourceId, state: gst::State) -> Result<()> {
        let source = self.manager.get_source(id)?;
        source.set_state(state)?;
        Ok(())
    }
    
    pub fn pause_source(&self, id: SourceId) -> Result<()> {
        self.set_source_state(id, gst::State::Paused)
    }
    
    pub fn resume_source(&self, id: SourceId) -> Result<()> {
        self.set_source_state(id, gst::State::Playing)
    }
    
    pub fn restart_source(&self, id: SourceId) -> Result<()> {
        let info = self.manager.get_source_info(id)?;
        let uri = info.uri.clone();
        
        self.remove_source(id)?;
        thread::sleep(Duration::from_millis(100));
        self.add_source(&uri)?;
        
        Ok(())
    }
    
    pub fn enable_auto_remove_on_eos(&mut self, enable: bool) {
        self.auto_remove_on_eos = enable;
        
        if enable {
            let manager = self.manager.clone();
            let eos_tracker = self.eos_tracker.clone();
            let _event_handler = self.event_handler.clone();
            
            self.event_handler.register_callback(move |event| {
                if let SourceEvent::Eos { id } = event {
                    let _ = eos_tracker.mark_eos(*id);
                    if let Err(e) = manager.remove_video_source(*id) {
                        eprintln!("Failed to auto-remove source {} on EOS: {:?}", id, e);
                    }
                }
            });
        }
    }
    
    pub fn handle_eos_sources(&self) -> Result<Vec<SourceId>> {
        let eos_sources = self.eos_tracker.get_eos_sources()?;
        let mut removed = Vec::new();
        
        for id in eos_sources {
            if self.manager.is_source_enabled(id)? {
                self.remove_source(id)?;
                removed.push(id);
            }
        }
        
        Ok(removed)
    }
    
    pub fn wait_for_sources_ready(&self, timeout: Duration) -> Result<()> {
        let source_ids = self.manager.list_sources()?;
        
        for id in source_ids {
            self.synchronizer.wait_for_state(
                id,
                SourceState::Playing,
                timeout
            )?;
        }
        
        Ok(())
    }
    
    pub fn get_event_handler(&self) -> Arc<SourceEventHandler> {
        self.event_handler.clone()
    }
    
    pub fn get_manager(&self) -> Arc<SourceManager> {
        self.manager.clone()
    }
    
    pub fn num_active_sources(&self) -> Result<usize> {
        self.manager.num_sources()
    }
    
    pub fn has_capacity(&self) -> Result<bool> {
        let num_sources = self.manager.num_sources()?;
        Ok(num_sources < super::MAX_NUM_SOURCES)
    }
}

pub struct DynamicSourceScheduler {
    controller: Arc<SourceController>,
    add_interval: Duration,
    remove_interval: Duration,
    running: Arc<Mutex<bool>>,
}

impl DynamicSourceScheduler {
    pub fn new(controller: Arc<SourceController>) -> Self {
        Self {
            controller,
            add_interval: Duration::from_secs(10),
            remove_interval: Duration::from_secs(10),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    pub fn with_intervals(
        controller: Arc<SourceController>,
        add_interval: Duration,
        remove_interval: Duration,
    ) -> Self {
        Self {
            controller,
            add_interval,
            remove_interval,
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    pub fn start_auto_add(&self, uris: Vec<String>) {
        let controller = self.controller.clone();
        let interval = self.add_interval;
        let running = self.running.clone();
        
        thread::spawn(move || {
            let mut uri_index = 0;
            
            loop {
                if let Ok(guard) = running.lock() {
                    if !*guard {
                        break;
                    }
                }
                
                if controller.has_capacity().unwrap_or(false) {
                    let uri = &uris[uri_index % uris.len()];
                    if let Ok(id) = controller.add_source(uri) {
                        println!("Auto-added source {} with URI: {}", id, uri);
                        uri_index += 1;
                    }
                }
                
                thread::sleep(interval);
            }
        });
    }
    
    pub fn start_auto_remove(&self) {
        let controller = self.controller.clone();
        let interval = self.remove_interval;
        let running = self.running.clone();
        
        thread::spawn(move || {
            loop {
                if let Ok(guard) = running.lock() {
                    if !*guard {
                        break;
                    }
                }
                
                if let Ok(sources) = controller.list_active_sources() {
                    if !sources.is_empty() {
                        use rand::Rng;
                        let mut rng = rand::thread_rng();
                        let random_index = rng.gen_range(0..sources.len());
                        let (id, _, _) = sources[random_index];
                        
                        if let Ok(()) = controller.remove_source(id) {
                            println!("Auto-removed source {}", id);
                        }
                    }
                }
                
                thread::sleep(interval);
            }
        });
    }
    
    pub fn start(&self) {
        if let Ok(mut running) = self.running.lock() {
            *running = true;
        }
    }
    
    pub fn stop(&self) {
        if let Ok(mut running) = self.running.lock() {
            *running = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dynamic_scheduler_creation() {
        gst::init().unwrap();
        
        let pipeline = Pipeline::new("test").unwrap();
        let streammux = gst::ElementFactory::make("identity")
            .name("test-mux")
            .build()
            .unwrap();
        
        let controller = Arc::new(SourceController::new(
            Arc::new(pipeline),
            streammux,
        ));
        
        let scheduler = DynamicSourceScheduler::new(controller);
        assert_eq!(scheduler.add_interval, Duration::from_secs(10));
        assert_eq!(scheduler.remove_interval, Duration::from_secs(10));
    }
}