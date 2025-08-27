use gstreamer as gst;
use gstreamer::glib;

mod cpudetector;
pub mod detector;

#[cfg(feature = "ort")]
pub use ort;

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
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
    env!("CARGO_PKG_REPOSITORY"),
    env!("BUILD_REL_DATE")
);
