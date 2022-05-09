use crate::device::Device;
use crate::types::*;

#[derive(Debug)]
pub struct Queue {
    handle: Ref<'static, VkQueue>,
    #[allow(dead_code)]
    device: Arc<Device>,
}

impl Queue {
    pub(crate) fn new(
        handle: Ref<'static, VkQueue>,
        device: Arc<Device>,
    ) -> Self {
        Self { handle, device }
    }
    pub fn borrow(&self) -> Ref<'_, VkQueue> {
        self.handle
    }
}
