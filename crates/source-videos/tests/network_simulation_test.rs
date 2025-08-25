use source_videos::network::{
    NetworkSimulator, NetworkController, NetworkConditions,
    NetworkProfile, StandardProfiles, GStreamerNetworkSimulator
};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn test_network_simulator_basic() {
    let simulator = NetworkSimulator::new();
    
    // Test enable/disable
    assert!(!simulator.is_enabled());
    simulator.enable();
    assert!(simulator.is_enabled());
    simulator.disable();
    assert!(!simulator.is_enabled());
}

#[test]
fn test_network_conditions() {
    let simulator = NetworkSimulator::new();
    simulator.enable();
    
    // Test perfect conditions
    simulator.apply_conditions(NetworkConditions::perfect());
    let conditions = simulator.get_conditions();
    assert_eq!(conditions.packet_loss, 0.0);
    assert_eq!(conditions.latency_ms, 0);
    assert!(!conditions.connection_dropped);
    
    // Test problematic conditions
    simulator.apply_conditions(NetworkConditions::problematic());
    let conditions = simulator.get_conditions();
    assert!(conditions.packet_loss > 0.0);
    assert!(conditions.latency_ms > 0);
    
    // Test disconnected conditions
    simulator.apply_conditions(NetworkConditions::disconnected());
    let conditions = simulator.get_conditions();
    assert!(conditions.connection_dropped);
}

#[test]
fn test_packet_loss_simulation() {
    let simulator = NetworkSimulator::new();
    simulator.enable();
    
    // Test 0% packet loss
    simulator.apply_conditions(NetworkConditions {
        packet_loss: 0.0,
        latency_ms: 0,
        bandwidth_kbps: 0,
        connection_dropped: false,
        jitter_ms: 0,
    });
    
    let mut dropped = 0;
    for _ in 0..1000 {
        if simulator.should_drop_packet() {
            dropped += 1;
        }
    }
    assert_eq!(dropped, 0);
    
    // Test 100% packet loss
    simulator.apply_conditions(NetworkConditions {
        packet_loss: 100.0,
        latency_ms: 0,
        bandwidth_kbps: 0,
        connection_dropped: false,
        jitter_ms: 0,
    });
    
    dropped = 0;
    for _ in 0..100 {
        if simulator.should_drop_packet() {
            dropped += 1;
        }
    }
    assert_eq!(dropped, 100);
}

#[test]
fn test_latency_simulation() {
    let simulator = NetworkSimulator::new();
    simulator.enable();
    
    // Test no latency
    simulator.apply_conditions(NetworkConditions {
        packet_loss: 0.0,
        latency_ms: 0,
        bandwidth_kbps: 0,
        connection_dropped: false,
        jitter_ms: 0,
    });
    assert_eq!(simulator.get_latency_delay(), Duration::ZERO);
    
    // Test fixed latency
    simulator.apply_conditions(NetworkConditions {
        packet_loss: 0.0,
        latency_ms: 100,
        bandwidth_kbps: 0,
        connection_dropped: false,
        jitter_ms: 0,
    });
    assert_eq!(simulator.get_latency_delay(), Duration::from_millis(100));
    
    // Test latency with jitter
    simulator.apply_conditions(NetworkConditions {
        packet_loss: 0.0,
        latency_ms: 100,
        bandwidth_kbps: 0,
        connection_dropped: false,
        jitter_ms: 50,
    });
    let delay = simulator.get_latency_delay();
    assert!(delay >= Duration::from_millis(100));
    assert!(delay <= Duration::from_millis(150));
}

#[test]
fn test_connection_control() {
    let simulator = NetworkSimulator::new();
    simulator.enable();
    
    // Test connection drop
    assert!(!simulator.is_connection_dropped());
    simulator.drop_connection();
    assert!(simulator.is_connection_dropped());
    
    // Test connection restore
    simulator.restore_connection();
    assert!(!simulator.is_connection_dropped());
}

#[test]
fn test_network_profiles() {
    let simulator = NetworkSimulator::new();
    simulator.enable();
    
    // Test each profile
    for profile in NetworkProfile::all() {
        simulator.apply_profile(profile);
        let conditions = simulator.get_conditions();
        
        match profile {
            NetworkProfile::Perfect => {
                assert_eq!(conditions.packet_loss, 0.0);
                assert_eq!(conditions.latency_ms, 0);
            }
            NetworkProfile::Mobile3G => {
                assert!(conditions.packet_loss > 0.0);
                assert!(conditions.latency_ms > 100);
                assert!(conditions.bandwidth_kbps > 0);
            }
            NetworkProfile::Satellite => {
                assert!(conditions.latency_ms > 500);
            }
            NetworkProfile::Poor => {
                assert!(conditions.packet_loss > 5.0);
                assert!(conditions.latency_ms > 300);
            }
            _ => {}
        }
    }
}

#[test]
fn test_standard_profiles() {
    // Test error recovery profile
    let profile = StandardProfiles::for_error_recovery();
    assert_eq!(profile, NetworkProfile::Poor);
    
    // Test reconnection conditions
    let conditions = StandardProfiles::for_reconnection_test();
    assert!(conditions.connection_dropped);
    
    // Test buffer test profile
    let profile = StandardProfiles::for_buffer_test();
    assert_eq!(profile, NetworkProfile::Mobile3G);
    
    // Test latency test profile
    let profile = StandardProfiles::for_latency_test();
    assert_eq!(profile, NetworkProfile::Satellite);
}

#[test]
fn test_profile_parsing() {
    // Test valid profiles
    assert_eq!("perfect".parse::<NetworkProfile>().unwrap(), NetworkProfile::Perfect);
    assert_eq!("3g".parse::<NetworkProfile>().unwrap(), NetworkProfile::Mobile3G);
    assert_eq!("mobile3g".parse::<NetworkProfile>().unwrap(), NetworkProfile::Mobile3G);
    assert_eq!("4g".parse::<NetworkProfile>().unwrap(), NetworkProfile::Mobile4G);
    assert_eq!("lte".parse::<NetworkProfile>().unwrap(), NetworkProfile::Mobile4G);
    assert_eq!("5g".parse::<NetworkProfile>().unwrap(), NetworkProfile::Mobile5G);
    assert_eq!("wifi".parse::<NetworkProfile>().unwrap(), NetworkProfile::WiFiHome);
    assert_eq!("public".parse::<NetworkProfile>().unwrap(), NetworkProfile::WiFiPublic);
    assert_eq!("satellite".parse::<NetworkProfile>().unwrap(), NetworkProfile::Satellite);
    assert_eq!("broadband".parse::<NetworkProfile>().unwrap(), NetworkProfile::Broadband);
    assert_eq!("poor".parse::<NetworkProfile>().unwrap(), NetworkProfile::Poor);
    
    // Test invalid profile
    assert!("invalid".parse::<NetworkProfile>().is_err());
}

#[test]
fn test_gstreamer_simulator() {
    gst::init().unwrap();
    
    let simulator = GStreamerNetworkSimulator::new();
    
    // Test element creation
    let bin = simulator.create_elements("test").unwrap();
    assert!(bin.static_pad("sink").is_some());
    assert!(bin.static_pad("src").is_some());
    
    // Test condition application
    let conditions = NetworkConditions {
        packet_loss: 10.0,
        latency_ms: 100,
        bandwidth_kbps: 1000,
        connection_dropped: false,
        jitter_ms: 20,
    };
    
    simulator.enable_with_conditions(conditions.clone());
    
    let current = simulator.get_conditions();
    assert_eq!(current.packet_loss, conditions.packet_loss);
    assert_eq!(current.latency_ms, conditions.latency_ms);
    assert_eq!(current.bandwidth_kbps, conditions.bandwidth_kbps);
}

#[test]
fn test_gstreamer_pipeline_integration() {
    gst::init().unwrap();
    
    // Create a simple test pipeline
    let pipeline = gst::Pipeline::with_name("test-pipeline");
    
    let source = gst::ElementFactory::make("videotestsrc")
        .name("source")
        .property("num-buffers", 100i32)
        .build()
        .unwrap();
    
    let sink = gst::ElementFactory::make("fakesink")
        .name("sink")
        .build()
        .unwrap();
    
    pipeline.add_many(&[&source, &sink]).unwrap();
    source.link(&sink).unwrap();
    
    // Insert network simulation
    let simulator = GStreamerNetworkSimulator::new();
    simulator.insert_into_pipeline(&pipeline, &source, &sink, "test_sim").unwrap();
    
    // Apply network conditions
    simulator.enable_with_profile(NetworkProfile::Mobile4G);
    
    // Start pipeline
    pipeline.set_state(gst::State::Playing).unwrap();
    
    // Wait for completion
    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::from_seconds(5)) {
        match msg.view() {
            gst::MessageView::Eos(_) => break,
            gst::MessageView::Error(err) => {
                panic!("Error in pipeline: {}", err.error());
            }
            _ => {}
        }
    }
    
    // Stop pipeline
    pipeline.set_state(gst::State::Null).unwrap();
}

#[test]
fn test_connection_drops_with_recovery() {
    let simulator = Arc::new(NetworkSimulator::new());
    simulator.enable();
    
    let drop_detected = Arc::new(Mutex::new(false));
    let restore_detected = Arc::new(Mutex::new(false));
    
    // Apply normal conditions
    simulator.apply_profile(NetworkProfile::Mobile4G);
    assert!(!simulator.is_connection_dropped());
    
    // Drop connection
    simulator.drop_connection();
    *drop_detected.lock().unwrap() = simulator.is_connection_dropped();
    
    // Restore after delay
    std::thread::sleep(Duration::from_millis(100));
    simulator.restore_connection();
    *restore_detected.lock().unwrap() = !simulator.is_connection_dropped();
    
    assert!(*drop_detected.lock().unwrap());
    assert!(*restore_detected.lock().unwrap());
}

#[test]
fn test_simulator_reset() {
    let simulator = NetworkSimulator::new();
    
    // Apply poor conditions
    simulator.apply_profile(NetworkProfile::Poor);
    simulator.enable();
    assert!(simulator.is_enabled());
    
    let conditions = simulator.get_conditions();
    assert!(conditions.packet_loss > 0.0);
    
    // Reset
    simulator.reset();
    assert!(!simulator.is_enabled());
    
    let conditions = simulator.get_conditions();
    assert_eq!(conditions.packet_loss, 0.0);
    assert_eq!(conditions.latency_ms, 0);
}