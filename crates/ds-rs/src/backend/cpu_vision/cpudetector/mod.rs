use gstreamer::glib;
use gstreamer as gst;
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
    // Register signals for the element
    let type_ = imp::CpuDetector::type_();
    
    let signal_id = glib::Signal::builder("inference-result")
        .param_types([u64::static_type(), glib::Value::static_type()])
        .build();
        
    let type_data = type_.type_data().ok_or_else(|| glib::BoolError)?;
    type_data.add_signal(&signal_id);
    
    gst::Element::register(
        Some(plugin),
        "cpudetector",
        gst::Rank::NONE,
        CpuDetector::static_type(),
    )
}