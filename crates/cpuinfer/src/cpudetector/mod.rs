use gstreamer::glib;
use gstreamer::prelude::*;

mod imp;

glib::wrapper! {
    pub struct CpuDetector(ObjectSubclass<imp::CpuDetector>) @extends gstreamer_base::BaseTransform, gstreamer::Element, gstreamer::Object;
}

pub fn register(plugin: &gstreamer::Plugin) -> Result<(), glib::BoolError> {
    gstreamer::Element::register(
        Some(plugin),
        "cpuinfer",
        gstreamer::Rank::NONE,
        CpuDetector::static_type(),
    )
}
