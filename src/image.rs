// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::enums::*;
use crate::error::{Error, ErrorAndSelf, Result, ResultAndSelf};
use crate::ext::khr_swapchain::SwapchainImages;
use crate::memory::{DeviceMemory, MemoryLifetime};
use crate::subobject::Subobject;
use crate::types::*;
use crate::vk::Device;

use std::fmt::Debug;

/// An image with no memory. Call [`Image::new`] to bind memory and create an
/// [`Image`].
#[derive(Debug)]
pub struct ImageWithoutMemory {
    handle: Handle<VkImage>,
    format: Format,
    extent: Extent3D,
    mip_levels: u32,
    array_layers: u32,
    res: ImageOwner,
    device: Arc<Device>,
}

/// An
#[doc = crate::spec_link!("image", "12", "resources-images")]
/// with memory attached to it.
#[derive(Debug)]
pub struct Image {
    inner: ImageWithoutMemory,
    _memory: Option<Subobject<MemoryLifetime>>,
}

#[derive(Debug)]
enum ImageOwner {
    Swapchain(Subobject<SwapchainImages>),
    Application,
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

impl ImageWithoutMemory {
    #[doc = crate::man_link!(vkCreateImage)]
    pub fn new(
        device: &Arc<Device>, info: &ImageCreateInfo<'_>,
    ) -> Result<Self> {
        let max_dim =
            info.extent.width.max(info.extent.height).max(info.extent.depth);
        if (info.image_type == ImageType::_1D
            && max_dim > device.limits().max_image_dimension_1d)
            || (info.image_type == ImageType::_2D
                && (info.flags & ImageCreateFlags::CUBE_COMPATIBLE).is_empty()
                && max_dim > device.limits().max_image_dimension_2d)
            || (info.image_type == ImageType::_2D
                && !(info.flags & ImageCreateFlags::CUBE_COMPATIBLE).is_empty()
                && max_dim > device.limits().max_image_dimension_cube)
            || (info.image_type == ImageType::_3D
                && max_dim > device.limits().max_image_dimension_3d)
            || (info.array_layers > device.limits().max_image_array_layers)
        {
            return Err(Error::LimitExceeded);
        }
        let mut handle = None;
        unsafe {
            (device.fun.create_image)(
                device.handle(),
                info,
                None,
                &mut handle,
            )?;
        }
        Ok(Self {
            handle: handle.unwrap(),
            extent: info.extent,
            format: info.format,
            mip_levels: info.mip_levels,
            array_layers: info.array_layers,
            res: ImageOwner::Application,
            device: device.clone(),
        })
    }
}
impl Image {
    #[doc = crate::man_link!(vkBindImageMemory)]
    pub fn new(
        image: ImageWithoutMemory, memory: &DeviceMemory, offset: u64,
    ) -> ResultAndSelf<Arc<Self>, ImageWithoutMemory> {
        assert_eq!(memory.device(), &image.device);
        if !memory.check(offset, image.memory_requirements()) {
            return Err(ErrorAndSelf(Error::InvalidArgument, image));
        }
        Self::bind_image_impl(image, memory, offset)
    }

    fn bind_image_impl(
        mut inner: ImageWithoutMemory, memory: &DeviceMemory, offset: u64,
    ) -> ResultAndSelf<Arc<Self>, ImageWithoutMemory> {
        if let Err(err) = unsafe {
            (memory.device().fun.bind_image_memory)(
                memory.device().handle(),
                inner.mut_handle(),
                memory.handle(),
                offset,
            )
        } {
            return Err(ErrorAndSelf(err.into(), inner));
        }
        Ok(Arc::new(Self { inner, _memory: Some(memory.resource()) }))
    }
}

impl Drop for ImageWithoutMemory {
    fn drop(&mut self) {
        if let ImageOwner::Application = &self.res {
            unsafe {
                (self.device.fun.destroy_image)(
                    self.device.handle(),
                    self.handle.borrow_mut(),
                    None,
                )
            }
        }
    }
}

impl ImageWithoutMemory {
    /// Borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkImage> {
        self.handle.borrow_mut()
    }
    #[doc = crate::man_link!(vkGetImageMemoryRequirements)]
    pub fn memory_requirements(&self) -> MemoryRequirements {
        let mut result = Default::default();
        unsafe {
            (self.device.fun.get_image_memory_requirements)(
                self.device.handle(),
                self.handle.borrow(),
                &mut result,
            );
        }
        result
    }
    /// Allocate a single piece of memory for the image and bind it.
    pub fn allocate_memory(
        self, memory_type_index: u32,
    ) -> ResultAndSelf<Arc<Image>, Self> {
        let mem_req = self.memory_requirements();
        if (1 << memory_type_index) & mem_req.memory_type_bits == 0 {
            return Err(ErrorAndSelf(Error::InvalidArgument, self));
        }
        let memory = match DeviceMemory::new(
            &self.device,
            mem_req.size,
            memory_type_index,
        ) {
            Ok(memory) => memory,
            Err(err) => return Err(ErrorAndSelf(err, self)),
        };
        // Don't need to check requirements
        Image::bind_image_impl(self, &memory, 0)
    }
}

impl Image {
    pub(crate) fn new_from(
        handle: Handle<VkImage>, device: Arc<Device>,
        res: Subobject<SwapchainImages>, format: Format, extent: Extent3D,
        array_layers: u32,
    ) -> Self {
        Self {
            inner: ImageWithoutMemory {
                handle,
                extent,
                array_layers,
                mip_levels: 1,
                res: ImageOwner::Swapchain(res),
                device,
                format,
            },
            _memory: None,
        }
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkImage> {
        self.inner.handle.borrow()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Arc<Device> {
        &self.inner.device
    }
    /// Returns the format of the image.
    pub fn format(&self) -> Format {
        self.inner.format
    }
    /// Returns the extent of the image.
    pub fn extent(&self, mip_level: u32) -> Extent3D {
        let ex = self.inner.extent;
        Extent3D {
            width: ex.width >> mip_level,
            height: ex.height >> mip_level,
            depth: ex.depth >> mip_level,
        }
    }
    /// Returns true if the given values are within the image's array layers.
    pub fn array_bounds_check(
        &self, base_array_layer: u32, layer_count: u32,
    ) -> bool {
        self.inner.array_layers >= base_array_layer
            && self.inner.array_layers - base_array_layer >= layer_count
    }
    /// Returns true if the given point is within the image at the given mip
    /// level.
    pub fn offset_bounds_check(
        &self, mip_level: u32, offset: Offset3D,
    ) -> bool {
        let ex = self.extent(mip_level);
        mip_level < self.inner.mip_levels
            && (offset.x >= 0 && offset.y >= 0 && offset.z >= 0)
            && ex.width >= offset.x as u32
            && ex.height >= offset.y as u32
            && ex.depth >= offset.z as u32
    }
    /// Returns true if the given rectangle is within the image at the given mip
    /// level.
    pub fn bounds_check(
        &self, mip_level: u32, offset: Offset3D, extent: Extent3D,
    ) -> bool {
        let ex = self.extent(mip_level);
        self.offset_bounds_check(mip_level, offset)
            && ex.width - offset.x as u32 >= extent.width
            && ex.height - offset.y as u32 >= extent.height
            && ex.depth - offset.z as u32 >= extent.depth
    }
}

/// An
#[doc = crate::spec_link!("image view", "12", "resources-image-views")]
#[derive(Debug)]
pub struct ImageView {
    handle: Handle<VkImageView>,
    image: Arc<Image>,
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

#[doc = crate::man_link!(VkImageViewCreateInfo)]
#[derive(Default)]
pub struct ImageViewCreateInfo {
    pub flags: ImageViewCreateFlags,
    pub view_type: ImageViewType,
    pub format: Format,
    pub components: ComponentMapping,
    pub subresource_range: ImageSubresourceRange,
}

impl ImageView {
    /// Create an image view of the image.
    pub fn new(
        image: &Arc<Image>, info: &ImageViewCreateInfo,
    ) -> Result<Arc<Self>> {
        let vk_info = VkImageViewCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: info.flags,
            image: image.handle(),
            view_type: info.view_type,
            format: info.format,
            components: info.components,
            subresource_range: info.subresource_range,
        };
        let mut handle = None;
        unsafe {
            (image.inner.device.fun.create_image_view)(
                image.inner.device.handle(),
                &vk_info,
                None,
                &mut handle,
            )?;
        }
        Ok(Arc::new(Self { handle: handle.unwrap(), image: image.clone() }))
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            (self.image.device().fun.destroy_image_view)(
                self.image.device().handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl ImageView {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkImageView> {
        self.handle.borrow()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Arc<Device> {
        self.image.device()
    }
}
