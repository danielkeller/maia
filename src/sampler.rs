use crate::device::Device;
use crate::error::Result;
use crate::types::*;

/// A
#[doc = crate::spec_link!("sampler", "samplers")]
///
/// Create with [Device::create_sampler]
#[derive(Debug, Eq)]
pub struct Sampler {
    handle: Handle<VkSampler>,
    device: Arc<Device>,
}

impl Device {
    #[doc = crate::man_link!(vkCreateSampler)]
    pub fn create_sampler(
        self: &Arc<Self>,
        info: &SamplerCreateInfo,
    ) -> Result<Arc<Sampler>> {
        let mut handle = None;
        unsafe {
            (self.fun.create_sampler)(self.handle(), info, None, &mut handle)?;
        }
        Ok(Arc::new(Sampler { handle: handle.unwrap(), device: self.clone() }))
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_sampler)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl PartialEq for Sampler {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl Sampler {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkSampler> {
        self.handle.borrow()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}
