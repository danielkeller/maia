use crate::device::Device;
use crate::types::*;

#[derive(Debug)]
pub struct Queue {
    handle: Handle<VkQueue>,
    #[allow(dead_code)]
    device: Arc<Device>,
}

impl Queue {
    pub(crate) fn new(handle: Handle<VkQueue>, device: Arc<Device>) -> Self {
        Self { handle, device }
    }
    pub fn borrow(&self) -> Ref<VkQueue> {
        self.handle.borrow()
    }
    pub fn borrow_mut(&mut self) -> Mut<VkQueue> {
        self.handle.borrow_mut()
    }
}
