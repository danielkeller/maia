use std::fmt::Debug;

use super::{DescriptorSet, DescriptorSetLayoutBinding};
use crate::buffer::Buffer;
use crate::device::Device;
use crate::enums::DescriptorType;
use crate::enums::ImageLayout;
use crate::error::{Error, Result};
use crate::exclusive::Exclusive;
use crate::ffi::Array;
use crate::image::ImageView;
use crate::sampler::Sampler;
use crate::types::*;

use bumpalo::collections::Vec as BumpVec;

/// An object to build calls to vkUpdateDescriptorSets. It's best to re-use it
/// as much as possible, since it holds onto some memory to avoid allocating.
///
#[doc = crate::man_link!(vkUpdateDescriptorSets)]
///
/// ```rust
/// # use maia::vk;
/// # let instance = vk::Instance::new(&Default::default())?;
/// # let (device, _) = vk::Device::new(
/// #     &instance.enumerate_physical_devices()?[0], &Default::default())?;
/// # let layout = vk::DescriptorSetLayout::new(
/// #     &device,
/// #     vec![vk::DescriptorSetLayoutBinding {
/// #         descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
/// #         descriptor_count: 1,
/// #         stage_flags: vk::ShaderStageFlags::VERTEX,
/// #         immutable_samplers: vec![],
/// #     },
/// #     vk::DescriptorSetLayoutBinding {
/// #         descriptor_type: vk::DescriptorType::SAMPLER,
/// #         descriptor_count: 1,
/// #         stage_flags: vk::ShaderStageFlags::FRAGMENT,
/// #         immutable_samplers: vec![],
/// #     }],
/// # )?;
/// # let mut pool = vk::DescriptorPool::new(
/// #     &device,
/// #     2,
/// #     &[vk::DescriptorPoolSize {
/// #         descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
/// #         descriptor_count: 2,
/// #     },
/// #     vk::DescriptorPoolSize {
/// #         descriptor_type: vk::DescriptorType::SAMPLER,
/// #         descriptor_count: 2,
/// #     }],
/// # )?;
/// let mut desc_set1 = vk::DescriptorSet::new(&mut pool, &layout)?;
/// let mut desc_set2 = vk::DescriptorSet::new(&mut pool, &layout)?;
/// # let buffer = vk::BufferWithoutMemory::new(&device, &vk::BufferCreateInfo {
/// #         size: 256,
/// #         ..Default::default()
/// #     })?.allocate_memory(0)?;
/// # let sampler = vk::Sampler::new(&device, &Default::default())?;
/// let mut update = vk::DescriptorSetUpdateBuilder::new(&device);
/// update
///     .begin()
///     .dst_set(&mut desc_set1)
///         .uniform_buffers(
///             0,
///             0,
///             &[vk::DescriptorBufferInfo {
///                 buffer: &buffer,
///                 offset: 0,
///                 range: buffer.len(),
///             }],
///         )?
///         .samplers(1, 0, &[&sampler])?
///     .dst_set(&mut desc_set2)
///         .uniform_buffers(
///             0,
///             0,
///             &[vk::DescriptorBufferInfo {
///                 buffer: &buffer,
///                 offset: 0,
///                 range: buffer.len(),
///             }],
///         )?
///         .samplers(1, 0, &[&sampler])?
///     .end();
/// # Ok::<_, vk::Error>(())
/// ```
pub struct DescriptorSetUpdateBuilder {
    pub(crate) device: Arc<Device>,
    pub(crate) scratch: Exclusive<bumpalo::Bump>,
}

impl DescriptorSetUpdateBuilder {
    /// Create an object that builds calls to vkUpdateDescriptorSets.
    pub fn new(device: &Arc<Device>) -> Self {
        DescriptorSetUpdateBuilder {
            scratch: Exclusive::new(bumpalo::Bump::new()),
            device: device.clone(),
        }
    }
}

struct Resource {
    set: usize,
    binding: usize,
    element: usize,
    resource: Arc<dyn Send + Sync + Debug>,
}

/// A builder for a call to vkUpdateDescriptorSets.
#[must_use = "This object does nothing until end() is called."]
pub struct DescriptorSetUpdates<'a> {
    device: &'a Device,
    bump: &'a bumpalo::Bump,
    writes: BumpVec<'a, VkWriteDescriptorSet<'a>>,
    copies: BumpVec<'a, VkCopyDescriptorSet<'a>>,
    dst_sets: BumpVec<'a, &'a mut DescriptorSet>,
    resources: BumpVec<'a, Resource>,
}

impl DescriptorSetUpdateBuilder {
    /// Begin creating a call to vkUpdateDescriptorSets. Since these calls are
    /// expensive, try to combine them as much as possible.
    pub fn begin(&mut self) -> DescriptorSetUpdates<'_> {
        let bump = &*self.scratch.get_mut();
        DescriptorSetUpdates {
            device: &self.device,
            bump,
            writes: bumpalo::vec![in bump],
            copies: bumpalo::vec![in bump],
            dst_sets: bumpalo::vec![in bump],
            resources: bumpalo::vec![in bump],
        }
    }
}

/// A builder to update a single descriptor set.
#[must_use = "This object does nothing until end() is called."]
pub struct DescriptorSetUpdate<'a> {
    pub(crate) updates: DescriptorSetUpdates<'a>,
    pub(crate) set: &'a mut DescriptorSet,
}

impl<'a> DescriptorSetUpdates<'a> {
    /// Add updates to the given set to the builder.
    pub fn dst_set(
        self,
        set: &'a mut DescriptorSet,
    ) -> DescriptorSetUpdate<'a> {
        assert_eq!(&*set.layout.device, self.device);
        DescriptorSetUpdate { updates: self, set }
    }
    pub(crate) fn end(mut self) {
        for res in self.resources {
            self.dst_sets[res.set].resources[res.binding][res.element] =
                Some(res.resource);
        }
        unsafe {
            (self.device.fun.update_descriptor_sets)(
                self.device.handle(),
                self.writes.len() as u32,
                Array::from_slice(&self.writes),
                self.copies.len() as u32,
                Array::from_slice(&self.copies),
            )
        }
    }
}

#[doc = crate::man_link!(VkDescriptorBufferInfo)]
pub struct DescriptorBufferInfo<'a> {
    pub buffer: &'a Arc<Buffer>,
    pub offset: u64,
    pub range: u64,
}

impl<'a> DescriptorSetUpdate<'a> {
    /// Finish the builder and call vkUpdateDescriptorSets.
    #[doc = crate::man_link!(vkUpdateDescriptorSets)]
    pub fn end(mut self) {
        self.updates.dst_sets.push(self.set);
        self.updates.end()
    }

    pub(crate) fn set_ref(&mut self) -> Mut<'a, VkDescriptorSet> {
        // Safety: The set is kept mutably borrowed while the builder
        // is alive, and one call to vkUpdateDescriptorSets counts as
        // a single use as far as external synchronization is concerned
        unsafe { self.set.handle.borrow_mut().reborrow_mut_unchecked() }
    }

    /// Add updates to the given set to the builder.
    pub fn dst_set(
        mut self,
        set: &'a mut DescriptorSet,
    ) -> DescriptorSetUpdate<'a> {
        self.updates.dst_sets.push(self.set);
        self.updates.dst_set(set)
    }

    pub(crate) fn buffers_impl(
        mut self,
        dst_binding: u32,
        dst_array_element: u32,
        buffers: &'_ [DescriptorBufferInfo<'a>],
        max_range: u32,
        descriptor_type: DescriptorType,
    ) -> Result<Self> {
        let iter = BindingIter::new(
            &self.set.layout.bindings,
            dst_binding as usize,
            dst_array_element,
            descriptor_type,
        );
        for (b, be) in buffers.iter().zip(iter) {
            let (binding, element) = be?;
            if b.range > max_range as u64 {
                return Err(Error::LimitExceeded);
            }
            assert!(std::ptr::eq(&**b.buffer.device(), self.updates.device));
            self.updates.resources.push(Resource {
                set: self.updates.dst_sets.len(),
                binding,
                element,
                resource: b.buffer.clone(),
            });
        }

        let buffer_infos =
            self.updates.bump.alloc_slice_fill_iter(buffers.iter().map(|b| {
                VkDescriptorBufferInfo {
                    buffer: b.buffer.handle(),
                    offset: b.offset,
                    range: b.range,
                }
            }));
        let dst_set = self.set_ref();
        self.updates.writes.push(VkWriteDescriptorSet {
            stype: Default::default(),
            next: Default::default(),
            dst_set,
            dst_binding,
            dst_array_element,
            descriptor_count: buffer_infos.len() as u32,
            descriptor_type,
            image_info: None,
            buffer_info: Array::from_slice(buffer_infos),
            texel_buffer_view: None,
        });
        Ok(self)
    }
    /// Update uniform buffer bindings. Returns [Error::OutOfBounds] if there
    /// are not enough bindings, and [Error::InvalidArgument] if some of the
    /// bindings in the destination range are of a different type.
    pub fn uniform_buffers(
        self,
        dst_binding: u32,
        dst_array_element: u32,
        buffers: &'_ [DescriptorBufferInfo<'a>],
    ) -> Result<Self> {
        let max_range = self.updates.device.limits().max_uniform_buffer_range;
        self.buffers_impl(
            dst_binding,
            dst_array_element,
            buffers,
            max_range,
            DescriptorType::UNIFORM_BUFFER,
        )
    }
    /// Update storage buffer bindings. Returns [Error::OutOfBounds] if there
    /// are not enough bindings, and [Error::InvalidArgument] if some of the
    /// bindings in the destination range are of a different type.
    pub fn storage_buffers(
        self,
        dst_binding: u32,
        dst_array_element: u32,
        buffers: &'_ [DescriptorBufferInfo<'a>],
    ) -> Result<Self> {
        let max_range = self.updates.device.limits().max_storage_buffer_range;
        self.buffers_impl(
            dst_binding,
            dst_array_element,
            buffers,
            max_range,
            DescriptorType::STORAGE_BUFFER,
        )
    }
    /// Update dynamic uniform buffer bindings. Returns [Error::OutOfBounds] if
    /// there are not enough bindings, and [Error::InvalidArgument] if some of
    /// the bindings in the destination range are of a different type.
    pub fn uniform_buffers_dynamic(
        self,
        dst_binding: u32,
        dst_array_element: u32,
        buffers: &'_ [DescriptorBufferInfo<'a>],
    ) -> Result<Self> {
        let max_range = self.updates.device.limits().max_uniform_buffer_range;
        self.buffers_impl(
            dst_binding,
            dst_array_element,
            buffers,
            max_range,
            DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        )
    }
    /// Update dynamic storage buffer bindings. Returns [Error::OutOfBounds] if
    /// there are not enough bindings, and [Error::InvalidArgument] if some of
    /// the bindings in the destination range are of a different type.
    pub fn storage_buffers_dynamic(
        self,
        dst_binding: u32,
        dst_array_element: u32,
        buffers: &'_ [DescriptorBufferInfo<'a>],
    ) -> Result<Self> {
        let max_range = self.updates.device.limits().max_storage_buffer_range;
        self.buffers_impl(
            dst_binding,
            dst_array_element,
            buffers,
            max_range,
            DescriptorType::STORAGE_BUFFER_DYNAMIC,
        )
    }

    /// Update sampler bindings. Returns [Error::OutOfBounds] if
    /// there are not enough bindings, and [Error::InvalidArgument] if some of
    /// the bindings in the destination range are of a different type or already
    /// have immutable samplers.
    pub fn samplers(
        mut self,
        dst_binding: u32,
        dst_array_element: u32,
        samplers: &[&'a Arc<Sampler>],
    ) -> Result<Self> {
        let iter = BindingIter::new(
            &self.set.layout.bindings,
            dst_binding as usize,
            dst_array_element,
            DescriptorType::SAMPLER,
        );
        for (&s, be) in samplers.iter().zip(iter) {
            let (binding, element) = be?;
            if !self.set.layout.bindings[binding].immutable_samplers.is_empty()
            {
                return Err(Error::InvalidArgument);
            }
            assert!(std::ptr::eq(&**s.device(), self.updates.device));
            self.updates.resources.push(Resource {
                set: self.updates.dst_sets.len(),
                binding,
                element,
                resource: s.clone(),
            });
        }
        let image_info =
            self.updates.bump.alloc_slice_fill_iter(samplers.iter().map(|s| {
                VkDescriptorImageInfo {
                    sampler: Some(s.handle()),
                    ..Default::default()
                }
            }));
        let dst_set = self.set_ref();
        self.updates.writes.push(VkWriteDescriptorSet {
            stype: Default::default(),
            next: Default::default(),
            dst_set,
            dst_binding,
            dst_array_element,
            descriptor_count: image_info.len() as u32,
            descriptor_type: DescriptorType::SAMPLER,
            image_info: Array::from_slice(image_info),
            buffer_info: None,
            texel_buffer_view: None,
        });
        Ok(self)
    }

    pub(crate) fn images_impl(
        mut self,
        dst_binding: u32,
        dst_array_element: u32,
        images: &[(&'a Arc<ImageView>, ImageLayout)],
        descriptor_type: DescriptorType,
    ) -> Result<Self> {
        let iter = BindingIter::new(
            &self.set.layout.bindings,
            dst_binding as usize,
            dst_array_element,
            descriptor_type,
        );
        for (&(i, _), be) in images.iter().zip(iter) {
            let (binding, element) = be?;
            assert!(std::ptr::eq(&**i.device(), self.updates.device));
            self.updates.resources.push(Resource {
                set: self.updates.dst_sets.len(),
                binding,
                element,
                resource: i.clone(),
            });
        }
        let image_info = self.updates.bump.alloc_slice_fill_iter(
            images.iter().map(|&(i, image_layout)| VkDescriptorImageInfo {
                image_view: Some(i.handle()),
                image_layout,
                ..Default::default()
            }),
        );
        let dst_set = self.set_ref();
        self.updates.writes.push(VkWriteDescriptorSet {
            stype: Default::default(),
            next: Default::default(),
            dst_set,
            dst_binding,
            dst_array_element,
            descriptor_count: image_info.len() as u32,
            descriptor_type,
            image_info: Array::from_slice(image_info),
            buffer_info: None,
            texel_buffer_view: None,
        });
        Ok(self)
    }
    /// Update sampled image bindings. Returns [Error::OutOfBounds] if
    /// there are not enough bindings, and [Error::InvalidArgument] if some of
    /// the bindings in the destination range are of a different type.
    pub fn sampled_images(
        self,
        dst_binding: u32,
        dst_array_element: u32,
        images: &[(&'a Arc<ImageView>, ImageLayout)],
    ) -> Result<Self> {
        self.images_impl(
            dst_binding,
            dst_array_element,
            images,
            DescriptorType::SAMPLED_IMAGE,
        )
    }
    /// Update storage image bindings. Returns [Error::OutOfBounds] if
    /// there are not enough bindings, and [Error::InvalidArgument] if some of
    /// the bindings in the destination range are of a different type.
    pub fn storage_images(
        self,
        dst_binding: u32,
        dst_array_element: u32,
        images: &[(&'a Arc<ImageView>, ImageLayout)],
    ) -> Result<Self> {
        self.images_impl(
            dst_binding,
            dst_array_element,
            images,
            DescriptorType::STORAGE_IMAGE,
        )
    }
    /// Update input attachment bindings. Returns [Error::OutOfBounds] if
    /// there are not enough bindings, and [Error::InvalidArgument] if some of
    /// the bindings in the destination range are of a different type.
    pub fn input_attachments(
        self,
        dst_binding: u32,
        dst_array_element: u32,
        images: &[(&'a Arc<ImageView>, ImageLayout)],
    ) -> Result<Self> {
        self.images_impl(
            dst_binding,
            dst_array_element,
            images,
            DescriptorType::INPUT_ATTACHMENT,
        )
    }

    /// Update combined image-sampler bindings. Returns [Error::OutOfBounds] if
    /// there are not enough bindings, and [Error::InvalidArgument] if some of
    /// the bindings in the destination range are of a different type.
    pub fn combined_image_samplers(
        mut self,
        dst_binding: u32,
        dst_array_element: u32,
        images: &[(&'a Arc<ImageView>, ImageLayout)],
    ) -> Result<Self> {
        let iter = BindingIter::new(
            &self.set.layout.bindings,
            dst_binding as usize,
            dst_array_element,
            DescriptorType::COMBINED_IMAGE_SAMPLER,
        );
        for (&(i, _), be) in images.iter().zip(iter) {
            let (binding, element) = be?;
            assert!(std::ptr::eq(&**i.device(), self.updates.device));
            self.updates.resources.push(Resource {
                set: self.updates.dst_sets.len(),
                binding,
                element,
                resource: i.clone(),
            });
        }
        let image_info = self.updates.bump.alloc_slice_fill_iter(
            images.iter().map(|&(i, image_layout)| VkDescriptorImageInfo {
                image_view: Some(i.handle()),
                image_layout,
                ..Default::default()
            }),
        );
        let dst_set = self.set_ref();
        self.updates.writes.push(VkWriteDescriptorSet {
            stype: Default::default(),
            next: Default::default(),
            dst_set,
            dst_binding,
            dst_array_element,
            descriptor_count: image_info.len() as u32,
            descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
            image_info: Array::from_slice(image_info),
            buffer_info: None,
            texel_buffer_view: None,
        });
        Ok(self)
    }
}

/// Gets the binding and element number for a series of consecutive bindings,
/// doing bounds and type checking on each.
struct BindingIter<'a> {
    bindings: &'a [DescriptorSetLayoutBinding],
    binding: usize,
    element: u32,
    descriptor_type: DescriptorType,
}

impl<'a> BindingIter<'a> {
    fn new(
        bindings: &'a [DescriptorSetLayoutBinding],
        binding: usize,
        element: u32,
        descriptor_type: DescriptorType,
    ) -> Self {
        Self { bindings, binding, element, descriptor_type }
    }
}

impl<'a> Iterator for BindingIter<'a> {
    type Item = Result<(usize, usize)>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.binding >= self.bindings.len() {
            return Some(Err(Error::OutOfBounds));
        }
        while self.element >= self.bindings[self.binding].descriptor_count {
            self.element -= self.bindings[self.binding].descriptor_count;
            self.binding += 1;
            if self.binding >= self.bindings.len() {
                return Some(Err(Error::OutOfBounds));
            }
        }
        if self.bindings[self.binding].descriptor_type != self.descriptor_type {
            return Some(Err(Error::InvalidArgument));
        }
        self.element += 1;

        Some(Ok((self.binding, self.element as usize - 1)))
    }
}
