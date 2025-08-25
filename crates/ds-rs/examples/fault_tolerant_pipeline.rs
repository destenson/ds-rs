use ds_rs::{
    init, timestamp, BackendManager, ElementFactory, Pipeline, 
    RecoveryConfig, RecoveryManager, HealthConfig, SourceHealthMonitor,
    CircuitBreakerConfig, CircuitBreaker, IsolationPolicy, IsolatedSource,
    ErrorClassifier, is_retryable,
};
use gstreamer as gst;
use gst::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use std::thread;

/// Demonstrates fault-tolerant pipeline with error recovery
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DeepStream Rust - Fault-Tolerant Pipeline Example");
    println!("=================================================");
    println!();
    
    // Initialize GStreamer and ds-rs
    init()?;
    
    // Detect and select backend
    let backend_manager = Arc::new(BackendManager::new()?);
    println!("Selected Backend: {}", backend_manager.backend_type().name());
    println!();
    
    // Create recovery configuration
    let recovery_config = RecoveryConfig {
        max_retries: 5,
        initial_backoff: Duration::from_secs(1),
        max_backoff: Duration::from_secs(30),
        backoff_multiplier: 2.0,
        jitter_factor: 0.3,
        health_monitoring_enabled: true,
        health_check_interval: Duration::from_secs(5),
        circuit_breaker_threshold: 3,
        half_open_interval: Duration::from_secs(20),
    };
    
    println!("Recovery Configuration:");
    println!("  Max Retries: {}", recovery_config.max_retries);
    println!("  Initial Backoff: {:?}", recovery_config.initial_backoff);
    println!("  Max Backoff: {:?}", recovery_config.max_backoff);
    println!("  Circuit Breaker Threshold: {}", recovery_config.circuit_breaker_threshold);
    println!();
    
    // Create health monitoring configuration
    let health_config = HealthConfig {
        min_frame_rate: 15.0,
        max_buffer_underruns: 3,
        max_network_latency_ms: 200.0,
        window_size_secs: 10,
        check_interval: Duration::from_secs(5),
        failure_threshold: 2,
    };
    
    println!("Health Monitoring Configuration:");
    println!("  Min Frame Rate: {} fps", health_config.min_frame_rate);
    println!("  Max Buffer Underruns: {}", health_config.max_buffer_underruns);
    println!("  Max Network Latency: {} ms", health_config.max_network_latency_ms);
    println!();
    
    // Create circuit breaker configuration
    let circuit_config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        window_duration: Duration::from_secs(60),
        open_duration: Duration::from_secs(15),
        half_open_max_requests: 3,
        request_timeout: Duration::from_secs(10),
    };
    
    // Build pipeline with fault tolerance
    let pipeline = build_fault_tolerant_pipeline(
        backend_manager.clone(),
        recovery_config.clone(),
        health_config.clone(),
        circuit_config.clone(),
    )?;
    
    // Set up bus watch for monitoring
    let bus = pipeline.bus().unwrap();
    let recovery_manager = Arc::new(RecoveryManager::new(recovery_config.clone()));
    let circuit_breaker = Arc::new(CircuitBreaker::new(
        "main-pipeline".to_string(),
        circuit_config.clone(),
    ));
    
    bus.add_watch(move |_, msg| {
        use gst::MessageView;
        
        match msg.view() {
            MessageView::Error(err) => {
                let error_msg = err.error().to_string();
                println!("[{:.3}] ERROR: {}", timestamp(), error_msg);
                
                // Classify the error
                let classifier = ErrorClassifier::new();
                let error = ds_rs::error::DeepStreamError::Unknown(error_msg.clone());
                
                if is_retryable(&error) {
                    println!("[{:.3}] Error is retryable, attempting recovery...", timestamp());
                    
                    // Check circuit breaker
                    if circuit_breaker.should_allow_request() {
                        // Attempt recovery
                        if let Some(backoff) = recovery_manager.start_recovery() {
                            println!("[{:.3}] Retrying after {:?}", timestamp(), backoff);
                            
                            // In a real implementation, schedule retry after backoff
                            thread::sleep(backoff);
                            
                            // Simulate recovery attempt
                            let success = simulate_recovery_attempt();
                            
                            if success {
                                recovery_manager.mark_recovered();
                                circuit_breaker.record_success();
                                println!("[{:.3}] Recovery successful!", timestamp());
                            } else {
                                recovery_manager.mark_failed(error_msg.clone());
                                circuit_breaker.record_failure(error_msg);
                                println!("[{:.3}] Recovery failed", timestamp());
                            }
                        } else {
                            println!("[{:.3}] Max retries exceeded, giving up", timestamp());
                        }
                    } else {
                        println!("[{:.3}] Circuit breaker is open, rejecting request", timestamp());
                    }
                } else {
                    println!("[{:.3}] Error is not retryable", timestamp());
                }
            }
            MessageView::Warning(warn) => {
                println!("[{:.3}] WARNING: {}", timestamp(), warn.debug().unwrap_or_default());
            }
            MessageView::Info(info) => {
                if let Some(s) = info.structure() {
                    println!("[{:.3}] INFO: {}", timestamp(), s);
                }
            }
            MessageView::StateChanged(state) => {
                if let Some(element) = msg.src() {
                    if element.name() == pipeline.name() {
                        println!(
                            "[{:.3}] Pipeline state changed: {:?} -> {:?}",
                            timestamp(),
                            state.old(),
                            state.current()
                        );
                    }
                }
            }
            _ => {}
        }
        
        gst::glib::ControlFlow::Continue
    })?;
    
    // Simulate various failure scenarios
    println!("\n=== Starting Failure Simulation ===\n");
    
    // Start the pipeline
    pipeline.set_state(gst::State::Playing)?;
    
    // Scenario 1: Transient network failure
    println!("[{:.3}] Scenario 1: Simulating transient network failure", timestamp());
    simulate_network_failure(&pipeline, Duration::from_secs(2));
    thread::sleep(Duration::from_secs(5));
    
    // Scenario 2: Buffer underrun
    println!("[{:.3}] Scenario 2: Simulating buffer underrun", timestamp());
    simulate_buffer_underrun(&pipeline);
    thread::sleep(Duration::from_secs(3));
    
    // Scenario 3: Source failure with recovery
    println!("[{:.3}] Scenario 3: Simulating source failure with recovery", timestamp());
    simulate_source_failure_and_recovery(&pipeline);
    thread::sleep(Duration::from_secs(5));
    
    // Display recovery statistics
    println!("\n=== Recovery Statistics ===");
    let stats = recovery_manager.get_stats();
    println!("Total Attempts: {}", stats.total_attempts);
    println!("Successful Recoveries: {}", stats.successful_recoveries);
    println!("Failed Recoveries: {}", stats.failed_recoveries);
    println!("Current Streak: {}", stats.current_streak);
    println!("Longest Streak: {}", stats.longest_streak);
    
    // Display circuit breaker metrics
    println!("\n=== Circuit Breaker Metrics ===");
    let metrics = circuit_breaker.get_metrics();
    println!("Total Requests: {}", metrics.total_requests);
    println!("Successful Requests: {}", metrics.successful_requests);
    println!("Failed Requests: {}", metrics.failed_requests);
    println!("Rejected Requests: {}", metrics.rejected_requests);
    println!("Circuit Opens: {}", metrics.circuit_opens);
    
    // Keep running for demonstration
    println!("\n[{:.3}] Pipeline running with fault tolerance...", timestamp());
    println!("Press Ctrl+C to stop");
    
    // Wait for interrupt
    let main_loop = gst::glib::MainLoop::new(None, false);
    
    #[cfg(unix)]
    {
        gst::glib::unix_signal_add(
            gst::glib::unix_signal_source::SIGINT,
            move || {
                println!("\n[{:.3}] Received interrupt signal, shutting down...", timestamp());
                main_loop.quit();
                gst::glib::ControlFlow::Break
            },
        );
    }
    
    #[cfg(windows)]
    {
        let main_loop_quit = main_loop.clone();
        ctrlc::set_handler(move || {
            println!("\n[{:.3}] Received interrupt signal, shutting down...", timestamp());
            main_loop_quit.quit();
        })?;
    }
    
    main_loop.run();
    
    // Clean shutdown
    pipeline.set_state(gst::State::Null)?;
    println!("[{:.3}] Pipeline stopped successfully", timestamp());
    
    Ok(())
}

/// Build a fault-tolerant pipeline
fn build_fault_tolerant_pipeline(
    backend_manager: Arc<BackendManager>,
    recovery_config: RecoveryConfig,
    health_config: HealthConfig,
    circuit_config: CircuitBreakerConfig,
) -> Result<Pipeline, Box<dyn std::error::Error>> {
    let factory = ElementFactory::new(backend_manager.clone());
    let pipeline = Pipeline::new("fault-tolerant-pipeline")?;
    
    // Create elements with isolation
    let _source = IsolatedSource::new(
        ds_rs::source::SourceId(0),
        IsolationPolicy::Full,
    );
    
    // Create test source with error injection capability
    let videosrc = gst::ElementFactory::make("videotestsrc")
        .property("pattern", 18) // Ball pattern
        .property("is-live", true)
        .build()?;
    
    // Create recovery manager for the source
    let _recovery_manager = RecoveryManager::new(recovery_config);
    
    // Create health monitor
    let _health_monitor = SourceHealthMonitor::new(
        ds_rs::source::SourceId(0),
        health_config,
    );
    
    // Create circuit breaker for the source
    let _circuit_breaker = CircuitBreaker::new(
        "source-0".to_string(),
        circuit_config,
    );
    
    // Create converter and sink
    let convert = factory.create_video_convert(Some("convert"))?;
    let sink = factory.create_video_sink(Some("sink"))?;
    
    // Add elements to pipeline
    pipeline.add(&videosrc)?;
    pipeline.add(&convert)?;
    pipeline.add(&sink)?;
    
    // Link elements
    gst::Element::link_many(&[
        &videosrc,
        &convert,
        &sink,
    ])?;
    
    Ok(pipeline)
}

/// Simulate a recovery attempt (50% success rate for demo)
fn simulate_recovery_attempt() -> bool {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_bool(0.5)
}

/// Simulate a network failure
fn simulate_network_failure(_pipeline: &Pipeline, duration: Duration) {
    println!("[{:.3}] Injecting network failure for {:?}", timestamp(), duration);
    
    // In a real scenario, this would disconnect network sources
    // For demo, we'll just log the simulation
    thread::spawn(move || {
        thread::sleep(duration);
        println!("[{:.3}] Network recovered", timestamp());
    });
}

/// Simulate a buffer underrun
fn simulate_buffer_underrun(_pipeline: &Pipeline) {
    println!("[{:.3}] Injecting buffer underrun", timestamp());
    
    // In a real scenario, this would cause the pipeline to pause/stutter
    // For demo, we'll just log the simulation
}

/// Simulate source failure and recovery
fn simulate_source_failure_and_recovery(_pipeline: &Pipeline) {
    println!("[{:.3}] Source failing...", timestamp());
    
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        println!("[{:.3}] Source attempting recovery...", timestamp());
        thread::sleep(Duration::from_secs(1));
        println!("[{:.3}] Source recovered", timestamp());
    });
}