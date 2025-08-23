use ds_rs::{init, app::Application};
use tokio::runtime::Runtime;
use std::time::Duration;

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
    
    let runtime = Runtime::new().expect("Failed to create runtime");
    
    runtime.block_on(async {
        let mut app = Application::new("fakesrc".to_string()).expect("Failed to create app");
        app.init().expect("Failed to init app");
        
        // Run for just 1 second then stop
        let app_handle = std::sync::Arc::new(app);
        let app_for_stop = app_handle.clone();
        
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let _ = app_for_stop.stop();
        });
        
        let mut app = std::sync::Arc::try_unwrap(app_handle)
            .map_err(|_| "Failed to unwrap").unwrap();
        
        let result = app.run().await;
        assert!(result.is_ok());
    });
}