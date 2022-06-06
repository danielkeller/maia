use std::collections::HashSet;
use std::mem::MaybeUninit;

use crate::device::Device;
use crate::error::{Error, Result};
use crate::types::*;

pub struct PipelineLayout {
    handle: Handle<VkPipelineLayout>,
    device: Arc<Device>,
}

impl Device {
    pub fn create_pipeline_layout(
        self: &Arc<Self>,
        info: &PipelineLayoutCreateInfo<'_>,
    ) -> Result<Arc<PipelineLayout>> {
        let mut handle = None;
        unsafe {
            (self.fun.create_pipeline_layout)(
                self.borrow(),
                info,
                None,
                &mut handle,
            )?;
        }
        Ok(Arc::new(PipelineLayout {
            handle: handle.unwrap(),
            device: self.clone(),
        }))
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_pipeline_layout)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl PipelineLayout {
    pub fn borrow(&self) -> Ref<VkPipelineLayout> {
        self.handle.borrow()
    }
}

#[derive(Debug)]
pub struct Pipeline {
    handle: Handle<VkPipeline>,
    device: Arc<Device>,
}

impl Device {
    pub fn create_graphics_pipeline(
        self: &Arc<Self>,
        info: &GraphicsPipelineCreateInfo<'_>,
    ) -> Result<Arc<Pipeline>> {
        // TODO: check the render pass index is in bounds
        let mut bindings = HashSet::new();
        for b in info.vertex_input_state.vertex_binding_descriptions {
            if !bindings.insert(b.binding) {
                return Err(Error::InvalidArgument);
            }
        }
        let mut locations = HashSet::new();
        for att in info.vertex_input_state.vertex_attribute_descriptions {
            if !locations.insert(att.location)
                || !bindings.contains(&att.binding)
            {
                return Err(Error::InvalidArgument);
            }
        }
        let mut handle = MaybeUninit::uninit();
        unsafe {
            (self.fun.create_graphics_pipelines)(
                self.borrow(),
                None,
                1,
                std::array::from_ref(info).into(),
                None,
                std::array::from_mut(&mut handle).into(),
            )?;
        }
        Ok(Arc::new(Pipeline {
            handle: unsafe { handle.assume_init() },
            device: self.clone(),
        }))
    }
}

impl Pipeline {
    pub fn borrow(&self) -> Ref<VkPipeline> {
        self.handle.borrow()
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_pipeline)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}
