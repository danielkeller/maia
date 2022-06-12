use crate::enums::*;
use crate::error::{Error, ErrorAndSelf, Result, ResultAndSelf};
use crate::ext::khr_swapchain::SwapchainImages;
use crate::memory::{DeviceMemory, MemoryPayload};
use crate::subobject::Subobject;
use crate::types::*;
use crate::vk::Device;

use std::fmt::Debug;

#[derive(Debug)]
enum ImageOwner {
    Swapchain(Subobject<SwapchainImages>),
    Application,
}

#[derive(Debug)]
pub struct ImageWithoutMemory {
    handle: Handle<VkImage>,
    extent: Extent3D,
    res: ImageOwner,
    device: Arc<Device>,
}

#[derive(Debug)]
pub struct Image {
    inner: ImageWithoutMemory,
    _memory: Option<Subobject<MemoryPayload>>,
}

impl PartialEq for Image {
    fn eq(&self, other: &Self) -> bool {
        self.inner.device == other.inner.device
            && self.inner.handle == other.inner.handle
    }
}
impl Eq for Image {}
impl std::hash::Hash for Image {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.device.hash(state);
        self.inner.handle.hash(state);
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
        Ok(ImageWithoutMemory {
            handle: handle.unwrap(),
            extent: info.extent,
            res: ImageOwner::Application,
            device: self.clone(),
        })
    }
}
impl DeviceMemory {
    pub fn bind_image_memory(
        &self,
        image: ImageWithoutMemory,
        offset: u64,
    ) -> ResultAndSelf<Arc<Image>, ImageWithoutMemory> {
        if !Arc::ptr_eq(&self.inner.device, &image.device)
            || !self.check(offset, image.memory_requirements())
        {
            return Err(ErrorAndSelf(Error::InvalidArgument, image));
        }
        self.bind_image_impl(image, offset)
    }

    fn bind_image_impl(
        &self,
        mut inner: ImageWithoutMemory,
        offset: u64,
    ) -> ResultAndSelf<Arc<Image>, ImageWithoutMemory> {
        if let Err(err) = unsafe {
            (self.inner.device.fun.bind_image_memory)(
                self.inner.device.borrow(),
                inner.borrow_mut(),
                self.borrow(),
                offset,
            )
        } {
            return Err(ErrorAndSelf(err.into(), inner));
        }
        Ok(Arc::new(Image {
            inner,
            _memory: Some(Subobject::new(&self.inner)),
        }))
    }
}

impl Drop for ImageWithoutMemory {
    fn drop(&mut self) {
        if let ImageOwner::Application = &self.res {
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
    /// Allocate a single piece of memory for the image and bind it.
    pub fn allocate_memory(
        self,
        memory_type_index: u32,
    ) -> ResultAndSelf<Arc<Image>, Self> {
        let mem_req = self.memory_requirements();
        if (1 << memory_type_index) & mem_req.memory_type_bits == 0 {
            return Err(ErrorAndSelf(Error::InvalidArgument, self));
        }
        let memory = match self
            .device
            .allocate_memory(mem_req.size, memory_type_index)
        {
            Ok(memory) => memory,
            Err(err) => return Err(ErrorAndSelf(err.into(), self)),
        };
        // Don't need to check requirements
        memory.bind_image_impl(self, 0)
    }
}

impl Image {
    pub(crate) fn new(
        handle: Handle<VkImage>,
        device: Arc<Device>,
        res: Subobject<SwapchainImages>,
        extent: Extent3D,
    ) -> Self {
        Self {
            inner: ImageWithoutMemory {
                handle,
                extent,
                res: ImageOwner::Swapchain(res),
                device,
            },
            _memory: None,
        }
    }

    pub fn borrow(&self) -> Ref<VkImage> {
        self.inner.handle.borrow()
    }
    pub fn device(&self) -> &Device {
        &*self.inner.device
    }
    pub fn extent(&self) -> Extent3D {
        self.inner.extent
    }
}

#[derive(Debug)]
pub struct ImageView {
    handle: Handle<VkImageView>,
    pub(crate) image: Arc<Image>,
}

impl PartialEq for ImageView {
    fn eq(&self, other: &Self) -> bool {
        self.image.inner.device == other.image.inner.device
            && self.handle == other.handle
    }
}
impl Eq for ImageView {}
impl std::hash::Hash for ImageView {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.image.inner.device.hash(state);
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
            (self.inner.device.fun.create_image_view)(
                self.inner.device.borrow(),
                &vk_info,
                None,
                &mut handle,
            )?;
        }
        Ok(Arc::new(ImageView { handle: handle.unwrap(), image: self.clone() }))
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            (self.image.device().fun.destroy_image_view)(
                self.image.device().borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl ImageView {
    pub fn borrow(&self) -> Ref<VkImageView> {
        self.handle.borrow()
    }
}
