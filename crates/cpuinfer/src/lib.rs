use gstreamer as gst;
use gstreamer::glib;

mod cpudetector;
pub mod detector;
pub mod config;

#[cfg(feature = "ort")]
pub use ort;

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    // GST_PLUGIN_PATH=$PWD/target/release:$GST_PLUGIN_PATH
    #[cfg(debug_assertions)]
    unsafe { 
        match std::env::var("GST_PLUGIN_PATH") {
            Ok(path) => {
                let workspace_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
                std::env::set_var("GST_PLUGIN_PATH", format!("{}/target/debug:{}", workspace_dir, path));
            },
            Err(_) => {
                let workspace_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
                std::env::set_var("GST_PLUGIN_PATH", format!("{}/target/debug", workspace_dir));
            }
        };
    };
    #[cfg(all(test, not(debug_assertions)))]
    unsafe { 
        match std::env::var("GST_PLUGIN_PATH") {
            Ok(path) => {
                let workspace_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
                std::env::set_var("GST_PLUGIN_PATH", format!("{}/target/release:{}", workspace_dir, path));
            },
            Err(_) => {
                let workspace_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
                std::env::set_var("GST_PLUGIN_PATH", format!("{}/target/release", workspace_dir));
            }
        };
    };
    cpudetector::register(plugin)?;
    Ok(())
}

gst::plugin_define!(
    cpuinfer,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    concat!(env!("CARGO_PKG_VERSION"), "-", env!("COMMIT_ID")),
    "MIT/Apache-2.0",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    "https://github.com/user/ds-rs",
    env!("BUILD_REL_DATE")
);
