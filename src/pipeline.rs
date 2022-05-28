use crate::device::Device;
use crate::error::Result;
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
