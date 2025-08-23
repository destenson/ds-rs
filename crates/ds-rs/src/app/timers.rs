use crate::source::{SourceController, SourceId};
use super::config;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use tokio::time::{interval, Duration};
use rand::Rng;

pub async fn source_addition_timer(
    source_controller: Arc<Mutex<SourceController>>,
    running: Arc<Mutex<bool>>,
    initial_uri: String,
) {
    let mut interval = interval(Duration::from_secs(config::SOURCE_ADD_INTERVAL_SECS));
    let mut source_count = 1; // Start with 1 because initial source is already added
    let mut enabled_sources: HashSet<SourceId> = HashSet::new();
    
    // Add the initial source to tracking
    if let Ok(controller) = source_controller.lock() {
        if let Ok(sources) = controller.list_active_sources() {
            for (id, _, _) in sources {
                enabled_sources.insert(id);
            }
        }
    }
    
    loop {
        interval.tick().await;
        
        if !*running.lock().unwrap() {
            break;
        }
        
        if source_count < config::MAX_NUM_SOURCES {
            // Add a new source
            let _source_id = {
                let controller = source_controller.lock().unwrap();
                match controller.add_source(&initial_uri) {
                    Ok(id) => {
                        println!("Added source {} (total: {})", id, source_count + 1);
                        source_count += 1;
                        enabled_sources.insert(id);
                        Some(id)
                    }
                    Err(e) => {
                        eprintln!("Failed to add source: {:?}", e);
                        None
                    }
                }
            };
            
            if source_count == config::MAX_NUM_SOURCES {
                println!("Reached MAX_NUM_SOURCES ({}), starting deletion timer", config::MAX_NUM_SOURCES);
                
                // Start deletion timer
                let source_controller_clone = source_controller.clone();
                let running_clone = running.clone();
                let enabled_sources_clone = enabled_sources.clone();
                
                tokio::spawn(async move {
                    source_deletion_timer(
                        source_controller_clone,
                        running_clone,
                        enabled_sources_clone,
                    ).await;
                });
                
                break; // Stop adding sources
            }
        }
    }
}

pub async fn source_deletion_timer(
    source_controller: Arc<Mutex<SourceController>>,
    running: Arc<Mutex<bool>>,
    mut enabled_sources: HashSet<SourceId>,
) {
    let mut interval = interval(Duration::from_secs(config::SOURCE_DELETE_INTERVAL_SECS));
    
    loop {
        interval.tick().await;
        
        if !*running.lock().unwrap() {
            break;
        }
        
        // First, handle any sources that have reached EOS
        {
            let controller = source_controller.lock().unwrap();
            if let Ok(removed) = controller.handle_eos_sources() {
                for id in removed {
                    enabled_sources.remove(&id);
                    println!("Removed source {} due to EOS", id);
                }
            }
        }
        
        if enabled_sources.is_empty() {
            println!("All sources stopped, quitting");
            break;
        }
        
        // Randomly select a source to remove
        let sources_vec: Vec<SourceId> = enabled_sources.iter().copied().collect();
        if !sources_vec.is_empty() {
            let mut rng = rand::thread_rng();
            let random_index = rng.gen_range(0..sources_vec.len());
            let source_to_remove = sources_vec[random_index];
            
            let controller = source_controller.lock().unwrap();
            match controller.remove_source(source_to_remove) {
                Ok(_) => {
                    enabled_sources.remove(&source_to_remove);
                    println!("Removed source {} (remaining: {})", source_to_remove, enabled_sources.len());
                    
                    if enabled_sources.is_empty() {
                        println!("All sources stopped, quitting");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to remove source {}: {:?}", source_to_remove, e);
                }
            }
        }
    }
}