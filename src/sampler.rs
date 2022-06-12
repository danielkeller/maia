use crate::device::Device;
use crate::error::Result;
use crate::types::*;

#[derive(Debug, Eq)]
pub struct Sampler {
    handle: Handle<VkSampler>,
    pub(crate) device: Arc<Device>,
}

impl Device {
    pub fn create_sampler(
        self: &Arc<Self>,
        info: &SamplerCreateInfo,
    ) -> Result<Arc<Sampler>> {
        let mut handle = None;
        unsafe {
            (self.fun.create_sampler)(self.borrow(), info, None, &mut handle)?;
        }
        Ok(Arc::new(Sampler { handle: handle.unwrap(), device: self.clone() }))
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_sampler)(
                self.device.borrow(),
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
    pub fn borrow(&self) -> Ref<VkSampler> {
        self.handle.borrow()
    }
}
