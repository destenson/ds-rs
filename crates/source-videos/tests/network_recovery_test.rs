use gstreamer::prelude::*;
use source_videos::{
    init,
    network::{
        GStreamerNetworkSimulator, NetworkConditions, NetworkController, NetworkProfile,
        NetworkScenario, ScenarioPlayer,
    },
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn test_netsim_element_creation() {
    init().unwrap();

    let simulator = GStreamerNetworkSimulator::new();
    let bin = simulator.create_elements("test").unwrap();

    assert!(bin.static_pad("sink").is_some());
    assert!(bin.static_pad("src").is_some());
}

#[test]
fn test_network_conditions_with_netsim_properties() {
    let conditions = NetworkConditions {
        packet_loss: 5.0,
        latency_ms: 100,
        bandwidth_kbps: 1000,
        connection_dropped: false,
        jitter_ms: 20,
        duplicate_probability: 1.0,
        allow_reordering: true,
        min_delay_ms: 50,
        max_delay_ms: 150,
        delay_probability: 100.0,
    };

    assert_eq!(conditions.packet_loss, 5.0);
    assert_eq!(conditions.duplicate_probability, 1.0);
    assert!(conditions.allow_reordering);
}

#[test]
fn test_network_profile_with_new_properties() {
    let mobile3g = NetworkProfile::Mobile3G.into_conditions();
    assert!(mobile3g.packet_loss > 0.0);
    assert!(mobile3g.duplicate_probability >= 0.0);
    assert_eq!(mobile3g.allow_reordering, true);
    assert!(mobile3g.delay_probability > 0.0);

    let perfect = NetworkProfile::Perfect.into_conditions();
    assert_eq!(perfect.packet_loss, 0.0);
    assert_eq!(perfect.duplicate_probability, 0.0);
    assert_eq!(perfect.allow_reordering, false);
}

#[test]
fn test_scenario_with_duplication() {
    let scenario = NetworkScenario::new("test_dup", "Test with packet duplication")
        .add_event(
            Duration::ZERO,
            NetworkConditions {
                packet_loss: 0.0,
                latency_ms: 10,
                bandwidth_kbps: 10000,
                connection_dropped: false,
                jitter_ms: 2,
                duplicate_probability: 0.0,
                allow_reordering: false,
                min_delay_ms: 5,
                max_delay_ms: 15,
                delay_probability: 100.0,
            },
        )
        .add_event(
            Duration::from_secs(30),
            NetworkConditions {
                packet_loss: 2.0,
                latency_ms: 50,
                bandwidth_kbps: 5000,
                connection_dropped: false,
                jitter_ms: 10,
                duplicate_probability: 5.0, // 5% duplication
                allow_reordering: true,
                min_delay_ms: 30,
                max_delay_ms: 70,
                delay_probability: 100.0,
            },
        );

    let conditions_start = scenario.get_conditions_at(Duration::ZERO);
    assert_eq!(conditions_start.duplicate_probability, 0.0);

    let conditions_30s = scenario.get_conditions_at(Duration::from_secs(30));
    assert_eq!(conditions_30s.duplicate_probability, 5.0);
}

#[test]
fn test_scenario_interpolation_with_new_fields() {
    let scenario = NetworkScenario::new("test_interp", "Test interpolation")
        .add_event(
            Duration::ZERO,
            NetworkConditions {
                packet_loss: 0.0,
                latency_ms: 10,
                bandwidth_kbps: 10000,
                connection_dropped: false,
                jitter_ms: 0,
                duplicate_probability: 0.0,
                allow_reordering: false,
                min_delay_ms: 10,
                max_delay_ms: 10,
                delay_probability: 0.0,
            },
        )
        .add_event(
            Duration::from_secs(60),
            NetworkConditions {
                packet_loss: 10.0,
                latency_ms: 100,
                bandwidth_kbps: 1000,
                connection_dropped: false,
                jitter_ms: 50,
                duplicate_probability: 4.0,
                allow_reordering: true,
                min_delay_ms: 50,
                max_delay_ms: 150,
                delay_probability: 100.0,
            },
        );

    // Test interpolation at 30 seconds (halfway)
    let conditions_mid = scenario.get_conditions_at(Duration::from_secs(30));
    assert!((conditions_mid.packet_loss - 5.0).abs() < 0.1);
    assert!((conditions_mid.duplicate_probability - 2.0).abs() < 0.1);
    assert!((conditions_mid.delay_probability - 50.0).abs() < 0.1);
}

#[test]
fn test_gstreamer_simulator_with_conditions() {
    init().unwrap();

    let simulator = GStreamerNetworkSimulator::new();
    let _bin = simulator.create_elements("test_conditions").unwrap();

    let conditions = NetworkConditions {
        packet_loss: 10.0,
        latency_ms: 200,
        bandwidth_kbps: 500,
        connection_dropped: false,
        jitter_ms: 50,
        duplicate_probability: 2.0,
        allow_reordering: true,
        min_delay_ms: 150,
        max_delay_ms: 250,
        delay_probability: 100.0,
    };

    simulator.apply_conditions(conditions.clone());

    let retrieved = simulator.get_conditions();
    assert_eq!(retrieved.packet_loss, conditions.packet_loss);
    assert_eq!(
        retrieved.duplicate_probability,
        conditions.duplicate_probability
    );
}

#[test]
fn test_scenario_player_progress() {
    init().unwrap();

    let scenario = NetworkScenario::degrading();
    let controller = Box::new(GStreamerNetworkSimulator::new()) as Box<dyn NetworkController>;
    let player = ScenarioPlayer::new(scenario, controller);

    assert!(player.progress() < 1.0); // Progress should be minimal at start
    assert!(!player.is_complete());

    // Update should apply conditions
    player.update();
}

#[tokio::test]
async fn test_network_recovery_simulation() {
    init().unwrap();

    // Track condition changes
    let condition_log = Arc::new(Mutex::new(Vec::new()));
    let log_clone = condition_log.clone();

    // Create a custom controller that logs conditions
    struct LoggingController {
        inner: GStreamerNetworkSimulator,
        log: Arc<Mutex<Vec<NetworkConditions>>>,
    }

    impl NetworkController for LoggingController {
        fn apply_conditions(&self, conditions: NetworkConditions) {
            self.log.lock().unwrap().push(conditions.clone());
            self.inner.apply_conditions(conditions);
        }

        fn get_conditions(&self) -> NetworkConditions {
            self.inner.get_conditions()
        }

        fn drop_connection(&self) {
            self.inner.drop_connection();
        }

        fn restore_connection(&self) {
            self.inner.restore_connection();
        }

        fn apply_profile(&self, profile: NetworkProfile) {
            self.inner.apply_profile(profile);
        }

        fn reset(&self) {
            self.inner.reset();
        }
    }

    let controller = Box::new(LoggingController {
        inner: GStreamerNetworkSimulator::new(),
        log: log_clone,
    }) as Box<dyn NetworkController>;

    // Create a short test scenario
    let scenario = NetworkScenario::new("test", "Recovery test")
        .add_event(Duration::ZERO, NetworkConditions::perfect())
        .add_event(Duration::from_millis(100), NetworkConditions::problematic())
        .add_event(Duration::from_millis(200), NetworkConditions::perfect());

    let player = ScenarioPlayer::new(scenario, controller);

    // Run scenario
    player.update();
    tokio::time::sleep(Duration::from_millis(100)).await;
    player.update();
    tokio::time::sleep(Duration::from_millis(100)).await;
    player.update();

    // Check that conditions were applied
    let log = condition_log.lock().unwrap();
    assert!(!log.is_empty(), "Conditions should have been applied");

    // First condition should be perfect (or very close to it)
    if log.len() > 0 {
        assert!(log[0].packet_loss < 0.1); // Allow for small floating point differences
    }
}

#[test]
fn test_drone_scenario_with_new_properties() {
    let scenario = NetworkScenario::drone_urban_flight();

    // Check that drone scenario includes duplicates and reordering
    let conditions_start = scenario.get_conditions_at(Duration::ZERO);
    assert!(conditions_start.packet_loss < 5.0); // Good signal at start

    // Check conditions during building obstruction
    let conditions_obstruction = scenario.get_conditions_at(Duration::from_secs(25));
    assert!(conditions_obstruction.connection_dropped); // Complete signal loss
}

#[test]
fn test_all_profiles_have_new_fields() {
    let profiles = NetworkProfile::all();

    for profile in profiles {
        let conditions = profile.into_conditions();
        // All profiles should have valid delay ranges
        assert!(conditions.max_delay_ms >= conditions.min_delay_ms);
        // Delay probability should be set if there's latency
        if conditions.latency_ms > 0 {
            assert!(conditions.delay_probability > 0.0);
        }
    }
}
