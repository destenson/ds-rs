
pub mod error;
pub mod platform;
pub mod backend;
pub mod elements;
pub mod config;

pub use error::{DeepStreamError, Result};
pub use platform::{Platform, PlatformInfo};
pub use backend::{Backend, BackendType, BackendCapabilities, BackendManager};
pub use elements::{DeepStreamElement, DeepStreamElementType, ElementBuilder};
pub use config::ApplicationConfig;

pub fn init() -> Result<()> {
    gstreamer::init().map_err(|e| DeepStreamError::GStreamer(e.into()))?;
    
    // Initialize logging if not already done
    let _ = log::set_logger(&SimpleLogger);
    log::set_max_level(log::LevelFilter::Info);
    
    Ok(())
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
    
    #[test]
    fn test_platform_detection() {
        let _ = init();
        let platform = PlatformInfo::detect();
        assert!(platform.is_ok());
    }
    
    #[test]
    fn test_backend_manager_creation() {
        let _ = init();
        let manager = BackendManager::new();
        assert!(manager.is_ok());
    }
}
