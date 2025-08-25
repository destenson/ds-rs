use crate::source::SourceController;
use super::config;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::rc::Rc;
use gstreamer::glib;
use rand::Rng;

/// State for managing source addition and deletion timers
pub struct TimerState {
    pub source_controller: Arc<Mutex<SourceController>>,
    pub initial_uri: String,
    pub num_sources: usize,
    pub enabled_sources: Vec<bool>,
    pub main_loop: glib::MainLoop,
}

impl TimerState {
    pub fn new(
        source_controller: Arc<Mutex<SourceController>>,
        initial_uri: String,
        main_loop: glib::MainLoop,
    ) -> Self {
        let mut enabled_sources = vec![false; config::MAX_NUM_SOURCES];
        enabled_sources[0] = true; // Initial source is already added
        
        Self {
            source_controller,
            initial_uri,
            num_sources: 1, // Start with 1 because initial source is already added
            enabled_sources,
            main_loop,
        }
    }
}

/// Timer callback for adding sources periodically
/// Returns Continue(true) to keep the timer running, Continue(false) to stop
pub fn add_sources_callback(state: Rc<RefCell<TimerState>>) -> glib::ControlFlow {
    let timestamp = crate::timestamp();
    let mut state_borrow = state.borrow_mut();
    
    // Find an available slot
    let mut source_id = None;
    for i in 0..config::MAX_NUM_SOURCES {
        if !state_borrow.enabled_sources[i] {
            source_id = Some(i);
            break;
        }
    }
    
    if let Some(slot) = source_id {
        println!("[{:.3}] Timer: Adding source at slot {}", timestamp, slot);
        
        // Add the source
        let result = {
            let controller = state_borrow.source_controller.lock().unwrap();
            controller.add_source(&state_borrow.initial_uri)
        };
        
        match result {
            Ok(id) => {
                state_borrow.enabled_sources[slot] = true;
                state_borrow.num_sources += 1;
                println!("[{:.3}] Added source {} at slot {} (total: {})", 
                         timestamp, id, slot, state_borrow.num_sources);
                
                // Check if we've reached the maximum
                if state_borrow.num_sources >= config::MAX_NUM_SOURCES {
                    println!("[{:.3}] Reached MAX_NUM_SOURCES ({}), starting deletion timer", 
                             timestamp, config::MAX_NUM_SOURCES);
                    
                    // Start the deletion timer
                    let state_clone = state.clone();
                    glib::timeout_add_seconds_local(
                        config::SOURCE_DELETE_INTERVAL_SECS as u32,
                        move || delete_sources_callback(state_clone.clone())
                    );
                    
                    // Stop the addition timer
                    return glib::ControlFlow::Break;
                }
            }
            Err(e) => {
                eprintln!("[{:.3}] Failed to add source: {:?}", timestamp, e);
            }
        }
    }
    
    glib::ControlFlow::Continue
}

/// Timer callback for deleting sources periodically
pub fn delete_sources_callback(state: Rc<RefCell<TimerState>>) -> glib::ControlFlow {
    let timestamp = crate::timestamp();
    let mut state_borrow = state.borrow_mut();
    
    // First, handle any sources that have reached EOS
    let eos_removed_count = {
        let controller = state_borrow.source_controller.lock().unwrap();
        if let Ok(removed) = controller.handle_eos_sources() {
            let count = removed.len();
            for _id in removed {
                println!("[{:.3}] Removed source due to EOS", timestamp);
            }
            count
        } else {
            0
        }
    };
    
    // Update the count after releasing the lock
    state_borrow.num_sources = state_borrow.num_sources.saturating_sub(eos_removed_count);
    
    if state_borrow.num_sources == 0 {
        println!("[{:.3}] All sources stopped, quitting", timestamp);
        state_borrow.main_loop.quit();
        return glib::ControlFlow::Break;
    }
    
    // Find an enabled source to remove randomly
    let mut enabled_indices = Vec::new();
    for (i, &enabled) in state_borrow.enabled_sources.iter().enumerate() {
        if enabled {
            enabled_indices.push(i);
        }
    }
    
    if !enabled_indices.is_empty() {
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..enabled_indices.len());
        let slot_to_remove = enabled_indices[random_index];
        
        println!("[{:.3}] Calling stop for slot {}", timestamp, slot_to_remove);
        
        // Get the actual sources to find the one at this slot
        let removal_result = {
            let controller = state_borrow.source_controller.lock().unwrap();
            if let Ok(sources) = controller.list_active_sources() {
                // Remove the first source we find (simulating slot-based removal)
                // In a real implementation, we'd map slots to source IDs properly
                if let Some((source_id, _, _)) = sources.get(slot_to_remove) {
                    Some((*source_id, controller.remove_source(*source_id)))
                } else {
                    None
                }
            } else {
                None
            }
        };
        
        if let Some((source_id, result)) = removal_result {
            match result {
                Ok(_) => {
                    state_borrow.enabled_sources[slot_to_remove] = false;
                    state_borrow.num_sources -= 1;
                    println!("[{:.3}] Removed source {} from slot {} (remaining: {})", 
                             timestamp, source_id, slot_to_remove, state_borrow.num_sources);
                    
                    if state_borrow.num_sources == 0 {
                        println!("[{:.3}] All sources stopped, quitting", timestamp);
                        state_borrow.main_loop.quit();
                        return glib::ControlFlow::Break;
                    }
                }
                Err(e) => {
                    eprintln!("[{:.3}] Failed to remove source: {:?}", timestamp, e);
                }
            }
        }
    }
    
    glib::ControlFlow::Continue
}