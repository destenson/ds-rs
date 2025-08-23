use super::{DeepStreamElement, DeepStreamElementType};
use crate::backend::BackendType;
use crate::error::Result;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer::glib;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct AbstractedElement {
    element: gst::Element,
    element_type: DeepStreamElementType,
    backend_type: BackendType,
    capabilities: ElementCapabilities,
}

#[derive(Debug, Clone)]
pub struct ElementCapabilities {
    pub supports_batching: bool,
    pub supports_gpu: bool,
    pub max_batch_size: Option<u32>,
    pub native_element_name: String,
    pub fallback_element_name: Option<String>,
}

impl AbstractedElement {
    pub fn new(
        element: gst::Element,
        element_type: DeepStreamElementType,
        backend_type: BackendType,
    ) -> Self {
        let capabilities = Self::determine_capabilities(&element, element_type, backend_type);
        
        Self {
            element,
            element_type,
            backend_type,
            capabilities,
        }
    }
    
    fn determine_capabilities(
        element: &gst::Element,
        element_type: DeepStreamElementType,
        backend_type: BackendType,
    ) -> ElementCapabilities {
        match backend_type {
            BackendType::DeepStream => {
                ElementCapabilities {
                    supports_batching: matches!(
                        element_type,
                        DeepStreamElementType::StreamMux | DeepStreamElementType::Inference
                    ),
                    supports_gpu: true,
                    max_batch_size: Some(30),
                    native_element_name: element_type.name().to_string(),
                    fallback_element_name: None,
                }
            }
            BackendType::Standard => {
                ElementCapabilities {
                    supports_batching: false,
                    supports_gpu: false,
                    max_batch_size: Some(4),
                    native_element_name: element.factory()
                        .map(|f| f.name().to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    fallback_element_name: Some(element_type.name().to_string()),
                }
            }
            BackendType::Mock => {
                ElementCapabilities {
                    supports_batching: true,
                    supports_gpu: false,
                    max_batch_size: Some(10),
                    native_element_name: "identity".to_string(),
                    fallback_element_name: Some(element_type.name().to_string()),
                }
            }
        }
    }
    
    pub fn backend_type(&self) -> BackendType {
        self.backend_type
    }
    
    pub fn capabilities(&self) -> &ElementCapabilities {
        &self.capabilities
    }
    
    pub fn is_hardware_accelerated(&self) -> bool {
        self.capabilities.supports_gpu
    }
    
    pub fn supports_batching(&self) -> bool {
        self.capabilities.supports_batching
    }
    
    pub fn get_max_batch_size(&self) -> Option<u32> {
        self.capabilities.max_batch_size
    }
    
    pub fn configure_for_batch_size(&self, batch_size: u32) -> Result<()> {
        if self.supports_batching() {
            if let Some(max) = self.get_max_batch_size() {
                if batch_size > max {
                    log::warn!(
                        "Requested batch size {} exceeds maximum {} for {}",
                        batch_size, max, self.element_type.name()
                    );
                }
            }
            
            self.element.set_property("batch-size", batch_size);
        }
        Ok(())
    }
    
    pub fn adapt_properties(&self, properties: &[(String, glib::Value)]) -> Vec<(String, glib::Value)> {
        // Adapt properties based on backend type
        match self.backend_type {
            BackendType::DeepStream => {
                // Use properties as-is for DeepStream
                properties.to_vec()
            }
            BackendType::Standard => {
                // Filter out DeepStream-specific properties
                properties
                    .iter()
                    .filter(|(key, _)| {
                        !key.starts_with("gpu-id") && 
                        !key.starts_with("nvbuf-") &&
                        !key.starts_with("ll-")
                    })
                    .cloned()
                    .collect()
            }
            BackendType::Mock => {
                // Mock backend ignores most properties
                Vec::new()
            }
        }
    }
}

impl DeepStreamElement for AbstractedElement {
    fn element_type(&self) -> DeepStreamElementType {
        self.element_type
    }
    
    fn inner(&self) -> &gst::Element {
        &self.element
    }
    
    fn inner_mut(&mut self) -> &mut gst::Element {
        &mut self.element
    }
}

pub struct AbstractedPipeline {
    pipeline: gst::Pipeline,
    elements: Rc<RefCell<Vec<AbstractedElement>>>,
    backend_type: BackendType,
}

impl AbstractedPipeline {
    pub fn new(name: &str, backend_type: BackendType) -> Self {
        let pipeline = gst::Pipeline::builder()
            .name(name)
            .build();
        
        Self {
            pipeline,
            elements: Rc::new(RefCell::new(Vec::new())),
            backend_type,
        }
    }
    
    pub fn add_element(&self, element: AbstractedElement) -> Result<()> {
        self.pipeline.add(element.inner())?;
        self.elements.borrow_mut().push(element);
        Ok(())
    }
    
    pub fn link_elements(&self) -> Result<()> {
        let elements = self.elements.borrow();
        for i in 0..elements.len() - 1 {
            elements[i].link(&elements[i + 1])?;
        }
        Ok(())
    }
    
    pub fn set_state(&self, state: gst::State) -> Result<gst::StateChangeSuccess> {
        self.pipeline
            .set_state(state)
            .map_err(|_| crate::error::DeepStreamError::StateChange(
                format!("Failed to set pipeline to {:?} state", state)
            ))
    }
    
    pub fn get_element_by_name(&self, name: &str) -> Option<AbstractedElement> {
        self.elements
            .borrow()
            .iter()
            .find(|e| e.inner().name() == name)
            .cloned()
    }
    
    pub fn get_elements_by_type(&self, element_type: DeepStreamElementType) -> Vec<AbstractedElement> {
        self.elements
            .borrow()
            .iter()
            .filter(|e| e.element_type() == element_type)
            .cloned()
            .collect()
    }
    
    pub fn backend_type(&self) -> BackendType {
        self.backend_type
    }
    
    pub fn pipeline(&self) -> &gst::Pipeline {
        &self.pipeline
    }
    
    pub fn report_capabilities(&self) {
        log::info!("Pipeline '{}' using {} backend:", self.pipeline.name(), self.backend_type.name());
        
        for element in self.elements.borrow().iter() {
            let caps = element.capabilities();
            log::info!(
                "  - {} ({}): GPU={}, Batching={}, MaxBatch={:?}",
                element.element_type().name(),
                caps.native_element_name,
                caps.supports_gpu,
                caps.supports_batching,
                caps.max_batch_size
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_abstracted_element() {
        let _ = gst::init();
        
        let element = gst::ElementFactory::make("identity")
            .name("test-element")
            .build()
            .unwrap();
        
        let abstracted = AbstractedElement::new(
            element,
            DeepStreamElementType::StreamMux,
            BackendType::Mock,
        );
        
        assert_eq!(abstracted.backend_type(), BackendType::Mock);
        assert_eq!(abstracted.element_type(), DeepStreamElementType::StreamMux);
        assert!(abstracted.capabilities().supports_batching);
    }
    
    #[test]
    fn test_abstracted_pipeline() {
        let _ = gst::init();
        
        let pipeline = AbstractedPipeline::new("test-pipeline", BackendType::Mock);
        
        let element1 = gst::ElementFactory::make("identity")
            .name("element1")
            .build()
            .unwrap();
        
        let element2 = gst::ElementFactory::make("identity")
            .name("element2")
            .build()
            .unwrap();
        
        let abs_elem1 = AbstractedElement::new(
            element1,
            DeepStreamElementType::StreamMux,
            BackendType::Mock,
        );
        
        let abs_elem2 = AbstractedElement::new(
            element2,
            DeepStreamElementType::Inference,
            BackendType::Mock,
        );
        
        assert!(pipeline.add_element(abs_elem1).is_ok());
        assert!(pipeline.add_element(abs_elem2).is_ok());
        assert!(pipeline.link_elements().is_ok());
        
        assert_eq!(pipeline.backend_type(), BackendType::Mock);
        assert!(pipeline.get_element_by_name("element1").is_some());
        assert_eq!(
            pipeline.get_elements_by_type(DeepStreamElementType::StreamMux).len(),
            1
        );
    }
}