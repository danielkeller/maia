use std::fmt::Debug;

use crate::types::*;

#[derive(Debug)]
pub struct Image {
    handle: Handle<VkImage>,
    _res: Arc<dyn Send + Sync + Debug>,
}

impl Image {
    pub(crate) fn new(
        handle: Handle<VkImage>,
        _res: Arc<dyn Send + Sync + Debug>,
    ) -> Self {
        Self { handle, _res }
    }

    pub fn borrow(&self) -> Ref<VkImage> {
        self.handle.borrow()
    }
}
