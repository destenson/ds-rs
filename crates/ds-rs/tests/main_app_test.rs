#![allow(unused)]
use ds_rs::{app::Application, init};
use std::time::Duration;
use tokio::runtime::Runtime;

#[test]
fn test_application_creation() {
    init().expect("Failed to initialize");

    let app = Application::new("fakesrc".to_string());
    assert!(app.is_ok());
}

#[test]
fn test_application_init() {
    init().expect("Failed to initialize");

    let mut app = Application::new("fakesrc".to_string()).expect("Failed to create app");
    let result = app.init();
    assert!(result.is_ok());
}

#[test]
#[ignore] // This test requires actual runtime
fn test_application_run_brief() {
    init().expect("Failed to initialize");

    // The Application uses GLib MainLoop, not async/await
    // For testing, we'll just verify the app can be created and initialized
    let mut app = Application::new("fakesrc".to_string()).expect("Failed to create app");
    let result = app.init();
    assert!(result.is_ok());

    // Note: run_with_glib_signals() is synchronous and blocks until interrupted
    // For actual runtime testing, this would need to be run in a separate thread
    // with a mechanism to stop it after a delay
}
