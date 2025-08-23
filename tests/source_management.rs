use ds_rs as ds;
use ds::{Pipeline, SourceController};
use gstreamer as gst;
use std::sync::Arc;
use std::time::Duration;
use std::thread;

fn create_test_pipeline() -> (Arc<Pipeline>, gst::Element) {
    ds::init().expect("Failed to initialize");
    
    let pipeline = Pipeline::builder("test-pipeline")
        .backend(ds::BackendType::Mock)
        .build()
        .expect("Failed to create pipeline");
    
    let streammux = gst::ElementFactory::make("identity")
        .name("test-streammux")
        .build()
        .expect("Failed to create identity element");
    
    pipeline.add_element(&streammux).expect("Failed to add streammux");
    
    (Arc::new(pipeline), streammux)
}

#[test]
fn test_source_controller_creation() {
    let (pipeline, streammux) = create_test_pipeline();
    
    let controller = SourceController::new(pipeline, streammux);
    assert_eq!(controller.num_active_sources().unwrap(), 0);
}

#[test]
fn test_add_single_source() {
    let (pipeline, streammux) = create_test_pipeline();
    let controller = SourceController::new(pipeline, streammux);
    
    let uri = "file:///tmp/test_video.mp4";
    let source_id = controller.add_source(uri).expect("Failed to add source");
    
    assert_eq!(controller.num_active_sources().unwrap(), 1);
    
    let sources = controller.list_active_sources().unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].0, source_id);
    assert_eq!(sources[0].1, uri);
}

#[test]
fn test_remove_source() {
    let (pipeline, streammux) = create_test_pipeline();
    let controller = SourceController::new(pipeline, streammux);
    
    let uri = "file:///tmp/test_video.mp4";
    let source_id = controller.add_source(uri).expect("Failed to add source");
    assert_eq!(controller.num_active_sources().unwrap(), 1);
    
    controller.remove_source(source_id).expect("Failed to remove source");
    assert_eq!(controller.num_active_sources().unwrap(), 0);
}

#[test]
fn test_add_multiple_sources() {
    let (pipeline, streammux) = create_test_pipeline();
    let controller = SourceController::new(pipeline, streammux);
    
    let uris = vec![
        "file:///tmp/video1.mp4".to_string(),
        "file:///tmp/video2.mp4".to_string(),
        "file:///tmp/video3.mp4".to_string(),
    ];
    
    let source_ids = controller.add_sources_batch(&uris)
        .expect("Failed to add sources");
    
    assert_eq!(source_ids.len(), 3);
    assert_eq!(controller.num_active_sources().unwrap(), 3);
    
    let sources = controller.list_active_sources().unwrap();
    assert_eq!(sources.len(), 3);
}

#[test]
fn test_remove_all_sources() {
    let (pipeline, streammux) = create_test_pipeline();
    let controller = SourceController::new(pipeline, streammux);
    
    let uris = vec![
        "file:///tmp/video1.mp4".to_string(),
        "file:///tmp/video2.mp4".to_string(),
    ];
    
    controller.add_sources_batch(&uris).expect("Failed to add sources");
    assert_eq!(controller.num_active_sources().unwrap(), 2);
    
    controller.remove_all_sources().expect("Failed to remove all sources");
    assert_eq!(controller.num_active_sources().unwrap(), 0);
}

#[test]
fn test_source_state_transitions() {
    let (pipeline, streammux) = create_test_pipeline();
    let controller = SourceController::new(pipeline, streammux);
    
    let uri = "file:///tmp/test_video.mp4";
    let source_id = controller.add_source(uri).expect("Failed to add source");
    
    controller.pause_source(source_id).expect("Failed to pause source");
    
    controller.resume_source(source_id).expect("Failed to resume source");
}

#[test]
fn test_source_capacity() {
    let (pipeline, streammux) = create_test_pipeline();
    let controller = SourceController::new(pipeline, streammux);
    
    assert!(controller.has_capacity().unwrap());
    
    for i in 0..5 {
        let uri = format!("file:///tmp/video{}.mp4", i);
        controller.add_source(&uri).expect("Failed to add source");
    }
    
    assert!(controller.has_capacity().unwrap());
}

#[test]
fn test_event_handler() {
    let (pipeline, streammux) = create_test_pipeline();
    let controller = SourceController::new(pipeline, streammux);
    let event_handler = controller.get_event_handler();
    
    let received_events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = received_events.clone();
    
    event_handler.register_callback(move |event| {
        if let Ok(mut events) = events_clone.lock() {
            events.push(format!("{:?}", event));
        }
    });
    
    let uri = "file:///tmp/test_video.mp4";
    let source_id = controller.add_source(uri).expect("Failed to add source");
    
    thread::sleep(Duration::from_millis(100));
    
    controller.remove_source(source_id).expect("Failed to remove source");
    
    thread::sleep(Duration::from_millis(100));
    
    let events = received_events.lock().unwrap();
    assert!(events.len() >= 2);
}

#[test]
fn test_concurrent_operations() {
    let (pipeline, streammux) = create_test_pipeline();
    let controller = Arc::new(SourceController::new(pipeline, streammux));
    
    let mut handles = vec![];
    
    for i in 0..3 {
        let controller_clone = controller.clone();
        let handle = thread::spawn(move || {
            let uri = format!("file:///tmp/concurrent_video{}.mp4", i);
            controller_clone.add_source(&uri)
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().expect("Thread panicked").expect("Failed to add source");
    }
    
    assert_eq!(controller.num_active_sources().unwrap(), 3);
}

#[test]
fn test_source_manager_direct() {
    ds::init().expect("Failed to initialize");
    
    let manager = ds::SourceManager::with_defaults();
    
    let id1 = manager.generate_source_id().unwrap();
    assert_eq!(id1.0, 0);
    
    manager.mark_source_enabled(id1, true).unwrap();
    
    let id2 = manager.generate_source_id().unwrap();
    assert_eq!(id2.0, 1);
    
    manager.mark_source_enabled(id1, false).unwrap();
    
    let id3 = manager.generate_source_id().unwrap();
    assert_eq!(id3.0, 0);
}

#[test]
fn test_video_source_creation() {
    ds::init().expect("Failed to initialize");
    
    let source_id = ds::SourceId(0);
    let uri = "file:///tmp/test.mp4";
    
    let video_source = ds::VideoSource::new(source_id, uri)
        .expect("Failed to create video source");
    
    assert_eq!(video_source.id(), source_id);
    assert_eq!(video_source.uri(), uri);
    assert_eq!(video_source.current_state(), ds::SourceState::Idle);
}

#[test]
fn test_source_restart() {
    let (pipeline, streammux) = create_test_pipeline();
    let controller = SourceController::new(pipeline, streammux);
    
    let uri = "file:///tmp/test_video.mp4";
    let source_id = controller.add_source(uri).expect("Failed to add source");
    
    controller.restart_source(source_id).expect("Failed to restart source");
    
    assert_eq!(controller.num_active_sources().unwrap(), 1);
}

#[test]
fn test_maximum_sources_limit() {
    let (pipeline, streammux) = create_test_pipeline();
    let controller = SourceController::with_max_sources(pipeline, streammux, 3);
    
    for i in 0..3 {
        let uri = format!("file:///tmp/video{}.mp4", i);
        controller.add_source(&uri).expect("Failed to add source");
    }
    
    assert_eq!(controller.num_active_sources().unwrap(), 3);
    assert!(!controller.has_capacity().unwrap());
    
    let result = controller.add_source("file:///tmp/extra.mp4");
    assert!(result.is_err());
}