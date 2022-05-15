use crate::ext::khr_swapchain::SwapchainImages;
use crate::subobject::Subobject;
use crate::types::*;

use std::fmt::Debug;

#[derive(Debug)]
pub(crate) enum ImageOwner {
    Swapchain(Subobject<SwapchainImages>),
}

#[derive(Debug)]
pub struct Image {
    handle: Handle<VkImage>,
    _res: ImageOwner,
}

impl Image {
    pub(crate) fn new(handle: Handle<VkImage>, _res: ImageOwner) -> Self {
        Self { handle, _res }
    }

    pub fn borrow(&self) -> Ref<VkImage> {
        self.handle.borrow()
    }
}
