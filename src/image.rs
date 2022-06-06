use crate::enums::*;
use crate::error::{Error, ErrorAndSelf, Result, ResultAndSelf};
use crate::ext::khr_swapchain::SwapchainImages;
use crate::memory::{DeviceMemory, MemoryPayload};
use crate::subobject::Subobject;
use crate::types::*;
use crate::vk::Device;

use std::fmt::Debug;

#[derive(Debug)]
pub(crate) enum ImageOwner {
    Swapchain(Subobject<SwapchainImages>),
    Application(Subobject<MemoryPayload>),
}

#[must_use = "Image is leaked if it is not bound to memory"]
#[derive(Debug)]
pub struct ImageWithoutMemory {
    handle: Handle<VkImage>,
    device: Arc<Device>,
}

#[derive(Debug)]
pub struct Image {
    handle: Handle<VkImage>,
    pub(crate) device: Arc<Device>,
    res: ImageOwner,
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

impl Device {
    pub fn create_image(
        self: &Arc<Self>,
        info: &ImageCreateInfo<'_>,
    ) -> Result<ImageWithoutMemory> {
        let mut handle = None;
        unsafe {
            (self.fun.create_image)(self.borrow(), info, None, &mut handle)?;
        }
        Ok(ImageWithoutMemory { handle: handle.unwrap(), device: self.clone() })
    }
}
impl DeviceMemory {
    pub fn bind_image_memory(
        &self,
        mut image: ImageWithoutMemory,
        offset: u64,
    ) -> ResultAndSelf<Arc<Image>, ImageWithoutMemory> {
        if !Arc::ptr_eq(&self.inner.device, &image.device)
            || !self.check(offset, image.memory_requirements())
        {
            return Err(ErrorAndSelf(Error::InvalidArgument, image));
        }
        if let Err(err) = unsafe {
            (self.inner.device.fun.bind_image_memory)(
                self.inner.device.borrow(),
                image.borrow_mut(),
                self.borrow(),
                offset,
            )
        } {
            return Err(ErrorAndSelf(err.into(), image));
        }
        Ok(Arc::new(Image {
            handle: image.handle,
            device: image.device,
            res: ImageOwner::Application(Subobject::new(&self.inner)),
        }))
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        if let ImageOwner::Application(_) = &self.res {
            unsafe {
                (self.device.fun.destroy_image)(
                    self.device.borrow(),
                    self.handle.borrow_mut(),
                    None,
                )
            }
        }
    }
}

impl ImageWithoutMemory {
    pub fn borrow_mut(&mut self) -> Mut<VkImage> {
        self.handle.borrow_mut()
    }
    pub fn memory_requirements(&self) -> MemoryRequirements {
        let mut result = Default::default();
        unsafe {
            (self.device.fun.get_image_memory_requirements)(
                self.device.borrow(),
                self.handle.borrow(),
                &mut result,
            );
        }
        result
    }
}

impl Image {
    pub(crate) fn new(
        handle: Handle<VkImage>,
        device: Arc<Device>,
        res: ImageOwner,
    ) -> Self {
        Self { handle, device, res }
    }

    pub fn borrow(&self) -> Ref<VkImage> {
        self.handle.borrow()
    }
}

#[derive(Debug)]
pub struct ImageView {
    handle: Handle<VkImageView>,
    pub(crate) image: Arc<Image>,
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
