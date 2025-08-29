use gstreamer as gst;
use gstreamer::glib;
use gstreamer::prelude::*;
use gstreamer_base as gst_base;

mod imp;

glib::wrapper! {
    pub struct CpuDetector(ObjectSubclass<imp::CpuDetector>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

impl CpuDetector {
    pub fn new(name: Option<&str>) -> CpuDetector {
        glib::Object::builder()
            .property("name", name.unwrap_or("cpudetector0"))
            .build()
    }
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    // Simple registration without custom signals for now
    // Signals would require more complex GObject setup
    gst::Element::register(
        Some(plugin),
        "cpudetector",
        gst::Rank::NONE,
        CpuDetector::static_type(),
    )
}
