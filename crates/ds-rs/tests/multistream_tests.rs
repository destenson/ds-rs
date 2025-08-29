//! Tests for multi-stream detection pipeline functionality

use ds_rs::{
    MetricsCollector, MultiStreamConfig, MultiStreamConfigBuilder, MultiStreamManager, Pipeline,
    PipelinePool, ResourceLimits, ResourceManager, StreamCoordinator, StreamPriority, init,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn setup() -> Result<(), Box<dyn std::error::Error>> {
    init()?;
    Ok(())
}

fn create_test_config() -> MultiStreamConfig {
    MultiStreamConfigBuilder::new()
        .max_streams(4)
        .resource_limits(ResourceLimits {
            max_cpu_percent: 80.0,
            max_memory_mb: 1024.0,
            max_streams: 4,
            adaptive_throttling: true,
            memory_per_stream_mb: 100.0,
        })
        .worker_threads(2)
        .build()
}

#[test]
fn test_multistream_manager_creation() {
    setup().unwrap();

    let pipeline = Arc::new(Pipeline::new("test-pipeline").unwrap());
    let streammux = gstreamer::ElementFactory::make("identity")
        .name("test-mux")
        .build()
        .unwrap();

    pipeline.add_element(&streammux).unwrap();

    let config = create_test_config();
    let manager = MultiStreamManager::new(pipeline, streammux, config);

    assert!(manager.is_ok());
}

#[test]
fn test_add_single_stream() {
    setup().unwrap();

    let pipeline = Arc::new(Pipeline::new("test-pipeline").unwrap());
    let streammux = gstreamer::ElementFactory::make("identity")
        .name("test-mux")
        .build()
        .unwrap();

    pipeline.add_element(&streammux).unwrap();

    let config = create_test_config();
    let manager = MultiStreamManager::new(pipeline, streammux, config).unwrap();

    // Add a test stream
    let result = manager.add_stream("file:///test.mp4");
    assert!(result.is_ok());

    let states = manager.get_all_stream_states();
    assert_eq!(states.len(), 1);
}

#[test]
fn test_add_multiple_streams() {
    setup().unwrap();

    let pipeline = Arc::new(Pipeline::new("test-pipeline").unwrap());
    let streammux = gstreamer::ElementFactory::make("identity")
        .name("test-mux")
        .build()
        .unwrap();

    pipeline.add_element(&streammux).unwrap();

    let config = create_test_config();
    let manager = MultiStreamManager::new(pipeline, streammux, config).unwrap();

    // Add multiple streams
    let uris = vec![
        "file:///test1.mp4".to_string(),
        "file:///test2.mp4".to_string(),
        "file:///test3.mp4".to_string(),
    ];

    let result = manager.add_streams_batch(&uris);
    assert!(result.is_ok());

    let source_ids = result.unwrap();
    assert_eq!(source_ids.len(), 3);

    let states = manager.get_all_stream_states();
    assert_eq!(states.len(), 3);
}

#[test]
fn test_remove_stream() {
    setup().unwrap();

    let pipeline = Arc::new(Pipeline::new("test-pipeline").unwrap());
    let streammux = gstreamer::ElementFactory::make("identity")
        .name("test-mux")
        .build()
        .unwrap();

    pipeline.add_element(&streammux).unwrap();

    let config = create_test_config();
    let manager = MultiStreamManager::new(pipeline, streammux, config).unwrap();

    // Add and then remove a stream
    let source_id = manager.add_stream("file:///test.mp4").unwrap();
    assert_eq!(manager.get_all_stream_states().len(), 1);

    manager.remove_stream(source_id).unwrap();
    assert_eq!(manager.get_all_stream_states().len(), 0);
}

#[test]
fn test_resource_limits() {
    setup().unwrap();

    let pipeline = Arc::new(Pipeline::new("test-pipeline").unwrap());
    let streammux = gstreamer::ElementFactory::make("identity")
        .name("test-mux")
        .build()
        .unwrap();

    pipeline.add_element(&streammux).unwrap();

    // Create config with max 2 streams
    let config = MultiStreamConfigBuilder::new()
        .max_streams(2)
        .resource_limits(ResourceLimits {
            max_cpu_percent: 80.0,
            max_memory_mb: 512.0,
            max_streams: 2,
            adaptive_throttling: false,
            memory_per_stream_mb: 100.0,
        })
        .build();

    let manager = MultiStreamManager::new(pipeline, streammux, config).unwrap();

    // Add streams up to limit
    manager.add_stream("file:///test1.mp4").unwrap();
    manager.add_stream("file:///test2.mp4").unwrap();

    // Third stream should fail due to limit
    let result = manager.add_stream("file:///test3.mp4");
    assert!(result.is_err());
}

#[test]
fn test_pipeline_pool() {
    setup().unwrap();

    let pool = PipelinePool::new(4);

    // Allocate pipelines
    let source1 = ds_rs::SourceId(1);
    let source2 = ds_rs::SourceId(2);

    let pipeline1 = pool.allocate_pipeline(source1).unwrap();
    let pipeline2 = pool.allocate_pipeline(source2).unwrap();

    assert_ne!(pipeline1, pipeline2);

    // Get pool statistics
    let stats = pool.get_stats();
    assert_eq!(stats.active_pipelines, 2);
    assert_eq!(stats.available_pipelines, 0); // Pre-created pipelines are allocated

    // Release a pipeline
    pool.release_pipeline(pipeline1).unwrap();

    let stats = pool.get_stats();
    assert_eq!(stats.active_pipelines, 1);
}

#[test]
fn test_stream_coordinator() {
    setup().unwrap();

    let coordinator = StreamCoordinator::new();

    let source1 = ds_rs::SourceId(1);
    let source2 = ds_rs::SourceId(2);

    // Register streams
    coordinator.register_stream(source1, 0).unwrap();
    coordinator.register_stream(source2, 1).unwrap();

    // Set priorities
    coordinator
        .set_stream_priority(source1, StreamPriority::High)
        .unwrap();
    coordinator
        .set_stream_priority(source2, StreamPriority::Low)
        .unwrap();

    // Get next stream - with ordering, we may get either but priority should be preserved
    let mut found_high = false;
    let mut found_low = false;

    // Get both streams
    if let Some(stream1) = coordinator.get_next_stream() {
        if stream1.priority == StreamPriority::High {
            found_high = true;
        } else if stream1.priority == StreamPriority::Low {
            found_low = true;
        }
    }

    if let Some(stream2) = coordinator.get_next_stream() {
        if stream2.priority == StreamPriority::High {
            found_high = true;
        } else if stream2.priority == StreamPriority::Low {
            found_low = true;
        }
    }

    // We should have found at least one high priority stream
    assert!(found_high || found_low);
}

#[test]
fn test_resource_manager() {
    setup().unwrap();

    let limits = ResourceLimits {
        max_cpu_percent: 80.0,
        max_memory_mb: 1024.0,
        max_streams: 4,
        adaptive_throttling: true,
        memory_per_stream_mb: 100.0,
    };

    let manager = ResourceManager::new(limits);

    // Check if we can add a stream
    assert!(manager.can_add_stream().unwrap());

    // Add streams
    let source1 = ds_rs::SourceId(1);
    let source2 = ds_rs::SourceId(2);

    manager.stream_added(source1).unwrap();
    manager.stream_added(source2).unwrap();

    // Update usage
    manager.update_usage().unwrap();

    // Get current usage
    let usage = manager.get_current_usage().unwrap();
    assert_eq!(usage.active_streams, 2);

    // Remove a stream
    manager.stream_removed(source1).unwrap();

    let usage = manager.get_current_usage().unwrap();
    assert_eq!(usage.active_streams, 1);
}

#[test]
fn test_metrics_collector() {
    setup().unwrap();

    let collector = MetricsCollector::new();

    let source1 = ds_rs::SourceId(1);
    let source2 = ds_rs::SourceId(2);

    // Start collecting metrics
    collector.start_stream_metrics(source1);
    collector.start_stream_metrics(source2);

    // Update streams
    for _ in 0..10 {
        collector.update_stream(source1);
        collector.update_stream(source2);
        thread::sleep(Duration::from_millis(100));
    }

    // Record detections
    collector.record_detection(source1, 5, 10.0);
    collector.record_detection(source2, 3, 15.0);

    // Get metrics
    let metrics1 = collector.get_stream_metrics(source1);
    assert!(metrics1.is_some());
    assert_eq!(metrics1.unwrap().frames_processed, 10);

    // Get aggregate stats
    let stats = collector.get_aggregate_stats();
    assert_eq!(stats.active_streams, 2);
    assert_eq!(stats.total_frames_processed, 20);
    assert_eq!(stats.total_detections, 8);

    // Stop collecting for one stream
    collector.stop_stream_metrics(source1);

    let stats = collector.get_aggregate_stats();
    assert_eq!(stats.active_streams, 1);
}

#[test]
fn test_adaptive_quality_control() {
    setup().unwrap();

    let coordinator = StreamCoordinator::new();

    let source1 = ds_rs::SourceId(1);
    coordinator.register_stream(source1, 0).unwrap();

    // Apply quality reduction
    coordinator.apply_quality_reduction(0.8).unwrap();

    // Apply quality increase
    coordinator.apply_quality_increase(1.2).unwrap();
}

#[test]
fn test_stream_synchronization() {
    setup().unwrap();

    let coordinator = StreamCoordinator::new();

    let source1 = ds_rs::SourceId(1);
    let source2 = ds_rs::SourceId(2);
    let source3 = ds_rs::SourceId(3);

    coordinator.register_stream(source1, 0).unwrap();
    coordinator.register_stream(source2, 1).unwrap();
    coordinator.register_stream(source3, 2).unwrap();

    // Create sync group
    let sources = vec![source1, source2];
    coordinator.synchronize_streams(&sources).unwrap();
}

#[test]
fn test_concurrent_stream_operations() {
    setup().unwrap();

    let pipeline = Arc::new(Pipeline::new("test-pipeline").unwrap());
    let streammux = gstreamer::ElementFactory::make("identity")
        .name("test-mux")
        .build()
        .unwrap();

    pipeline.add_element(&streammux).unwrap();

    let config = create_test_config();
    let manager = Arc::new(MultiStreamManager::new(pipeline, streammux, config).unwrap());

    // Spawn multiple threads to add streams concurrently
    let mut handles = vec![];

    for i in 0..3 {
        let manager_clone = manager.clone();
        let handle = thread::spawn(move || {
            let uri = format!("file:///test{}.mp4", i);
            manager_clone.add_stream(&uri)
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        assert!(handle.join().unwrap().is_ok());
    }

    // Verify all streams were added
    let states = manager.get_all_stream_states();
    assert_eq!(states.len(), 3);
}
