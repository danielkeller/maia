use crate::enums::*;
use crate::error::Result;
use crate::ext::khr_swapchain::SwapchainImages;
use crate::subobject::Subobject;
use crate::types::*;
use crate::vk::Device;

use std::fmt::Debug;

#[derive(Debug)]
pub(crate) enum ImageOwner {
    Swapchain(Subobject<SwapchainImages>),
    Application, // TODO: Check for this on drop
}

#[derive(Debug)]
pub struct Image {
    handle: Handle<VkImage>,
    device: Arc<Device>,
    _res: ImageOwner,
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.device == other.device && self.handle == other.handle
    }
}
impl Eq for Image {}
impl std::hash::Hash for Image {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.device.hash(state);
        self.handle.hash(state);
    }
}

impl Image {
    pub(crate) fn new(
        handle: Handle<VkImage>,
        device: Arc<Device>,
        _res: ImageOwner,
    ) -> Self {
        Self { handle, device, _res }
    }

    pub fn borrow(&self) -> Ref<VkImage> {
        self.handle.borrow()
    }
}

#[derive(Debug)]
pub struct ImageView {
    handle: Handle<VkImageView>,
    image: Arc<Image>,
}

impl PartialEq for ImageView {
    fn eq(&self, other: &Self) -> bool {
        self.image.device == other.image.device && self.handle == other.handle
    }
}
impl Eq for ImageView {}
impl std::hash::Hash for ImageView {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.image.device.hash(state);
        self.handle.hash(state);
    }
}

#[derive(Default)]
pub struct ImageViewCreateInfo {
    pub flags: ImageViewCreateFlags,
    pub view_type: ImageViewType,
    pub format: Format,
    pub components: ComponentMapping,
    pub subresource_range: ImageSubresourceRange,
}

impl Image {
    pub fn create_view(
        self: &Arc<Self>,
        info: &ImageViewCreateInfo,
    ) -> Result<Arc<ImageView>> {
        let vk_info = VkImageViewCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: info.flags,
            image: self.borrow(),
            view_type: info.view_type,
            format: info.format,
            components: info.components,
            subresource_range: info.subresource_range,
        };
        let mut handle = None;
        unsafe {
            (self.device.fun.create_image_view)(
                self.device.borrow(),
                &vk_info,
                None,
                &mut handle,
            )?;
        }
        Ok(Arc::new(ImageView { handle: handle.unwrap(), image: self.clone() }))
    }
}

impl ImageView {
    pub fn borrow(&self) -> Ref<VkImageView> {
        self.handle.borrow()
    }
}
