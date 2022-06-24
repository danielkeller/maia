use std::fmt::Debug;
use std::mem::MaybeUninit;

use crate::buffer::Buffer;
use crate::device::Device;
use crate::enums::ImageLayout;
use crate::enums::{DescriptorType, ShaderStageFlags};
use crate::error::{Error, Result};
use crate::exclusive::Exclusive;
use crate::ffi::Array;
use crate::image::ImageView;
use crate::sampler::Sampler;
use crate::subobject::{Owner, Subobject};
use crate::types::*;

use bumpalo::collections::Vec as BumpVec;

/// A
#[doc = crate::spec_link!("descriptor set layout", "descriptorsets-setlayout")]
#[derive(Debug, Eq)]
pub struct DescriptorSetLayout {
    handle: Handle<VkDescriptorSetLayout>,
    bindings: Vec<DescriptorSetLayoutBinding>,
    device: Arc<Device>,
}

/// Note that unlike in Vulkan, the binding number is implicitly the index of
/// the array that is passed into [DescriptorSetLayout::new()].
/// If non-consecutive binding numbers are desired, create dummy descriptors to
/// fill the gaps.
///
/// For [DescriptorType::COMBINED_IMAGE_SAMPLER], currently the use of
/// immutable samplers is required.
///
#[doc = crate::man_link!(VkDescriptorSetLayoutBinding)]
#[derive(Debug, PartialEq, Eq, Default)]
pub struct DescriptorSetLayoutBinding {
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: ShaderStageFlags,
    pub immutable_samplers: Vec<Arc<Sampler>>,
}

impl DescriptorSetLayout {
    #[doc = crate::man_link!(vkDescriptorSetLayout)]
    pub fn new(
        device: &Arc<Device>,
        bindings: Vec<DescriptorSetLayoutBinding>,
    ) -> Result<Arc<Self>> {
        for b in &bindings {
            if !b.immutable_samplers.is_empty()
                && b.immutable_samplers.len() as u32 != b.descriptor_count
            {
                return Err(Error::InvalidArgument);
            }
            if b.descriptor_type == DescriptorType::COMBINED_IMAGE_SAMPLER
                && b.immutable_samplers.is_empty()
            {
                return Err(Error::InvalidArgument);
            }
        }
        let vk_samplers = bindings
            .iter()
            .map(|b| b.immutable_samplers.iter().map(|s| s.handle()).collect())
            .collect::<Vec<Vec<_>>>();
        let vk_bindings = bindings
            .iter()
            .zip(vk_samplers.iter())
            .enumerate()
            .map(|(i, (b, s))| VkDescriptorSetLayoutBinding {
                binding: i as u32,
                descriptor_type: b.descriptor_type,
                descriptor_count: b.descriptor_count,
                stage_flags: b.stage_flags,
                immutable_samplers: Array::from_slice(s),
            })
            .collect::<Vec<_>>();
        let mut handle = None;
        unsafe {
            (device.fun.create_descriptor_set_layout)(
                device.handle(),
                &VkDescriptorSetLayoutCreateInfo {
                    bindings: vk_bindings.as_slice().into(),
                    ..Default::default()
                },
                None,
                &mut handle,
            )?;
        }

        Ok(Arc::new(DescriptorSetLayout {
            handle: handle.unwrap(),
            bindings,
            device: device.clone(),
        }))
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_descriptor_set_layout)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl PartialEq for DescriptorSetLayout {
    /// Compatible descriptor sets layouts are equal
    fn eq(&self, other: &Self) -> bool {
        self.bindings == other.bindings && self.device == other.device
    }
}

impl DescriptorSetLayout {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkDescriptorSetLayout> {
        self.handle.borrow()
    }
    /// Returns the number of dynamic offsets the descriptor set will require.
    pub fn num_dynamic_offsets(&self) -> u32 {
        let mut result = 0;
        for b in &self.bindings {
            if b.descriptor_type == DescriptorType::UNIFORM_BUFFER_DYNAMIC
                || b.descriptor_type == DescriptorType::STORAGE_BUFFER_DYNAMIC
            {
                result += b.descriptor_count
            }
        }
        result
    }
    /// Returns the number of bindings of the specified type and stage.
    pub fn num_bindings(
        &self,
        descriptor_type: DescriptorType,
        stage_flags: ShaderStageFlags,
    ) -> u32 {
        self.bindings
            .iter()
            .filter(|b| {
                b.descriptor_type == descriptor_type
                    && !(b.stage_flags & stage_flags).is_empty()
            })
            .count() as u32
    }
}

struct DescriptorPoolLifetime {
    handle: Handle<VkDescriptorPool>,
    device: Arc<Device>,
}

#[derive(Debug)]
struct AllocatedSets;

/// A
#[doc = crate::spec_link!("descriptor pool", "descriptorsets-allocation")]
pub struct DescriptorPool {
    res: Owner<DescriptorPoolLifetime>,
    allocated: Arc<AllocatedSets>,
}

impl DescriptorPool {
    #[doc = crate::man_link!(vkCreateDescriptorPool)]
    pub fn new(
        device: &Arc<Device>,
        max_sets: u32,
        pool_sizes: &[DescriptorPoolSize],
    ) -> Result<Self> {
        let mut handle = None;
        unsafe {
            (device.fun.create_descriptor_pool)(
                device.handle(),
                &DescriptorPoolCreateInfo {
                    max_sets,
                    pool_sizes: pool_sizes.into(),
                    ..Default::default()
                },
                None,
                &mut handle,
            )?;
        }
        let res = Owner::new(DescriptorPoolLifetime {
            handle: handle.unwrap(),
            device: device.clone(),
        });
        let allocated = Arc::new(AllocatedSets);
        Ok(DescriptorPool { res, allocated })
    }
}

impl Drop for DescriptorPoolLifetime {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_descriptor_pool)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl DescriptorPool {
    /// If all descriptor sets allocated from the pool have not been dropped,
    /// returns [Error::SynchronizationError].
    pub fn reset(&mut self) -> Result<()> {
        if Arc::get_mut(&mut self.allocated).is_none() {
            return Err(Error::SynchronizationError);
        }
        let res = &mut *self.res;
        unsafe {
            (res.device.fun.reset_descriptor_pool)(
                res.device.handle(),
                res.handle.borrow_mut(),
                Default::default(),
            )?;
        }
        Ok(())
    }
}

/// A
#[doc = concat!(crate::spec_link!("descriptor set", "descriptorsets-sets"), ".")]
///
/// Any resources that are written into the descriptor set have their reference
/// count incremented and held by the set. To decrement the count and allow the
/// resources to be freed, the descriptor set must be dropped. (Note that calling
/// [bind_descriptor_sets()](crate::command_buffer::CommandRecording::bind_descriptor_sets)
/// will prevent the set from being freed until the command pool is
/// [reset](crate::command_buffer::CommandPool::reset).)
#[derive(Debug)]
pub struct DescriptorSet {
    handle: Handle<VkDescriptorSet>,
    layout: Arc<DescriptorSetLayout>,
    resources: Vec<Vec<Option<Arc<dyn Send + Sync + Debug>>>>,
    _allocation: Arc<AllocatedSets>,
    _pool: Subobject<DescriptorPoolLifetime>,
}

impl DescriptorSet {
    #[doc = crate::man_link!(vkAllocateDescriptorSets)]
    pub fn new(
        pool: &mut DescriptorPool,
        layout: &Arc<DescriptorSetLayout>,
    ) -> Result<Self> {
        if !Arc::ptr_eq(&pool.res.device, &layout.device) {
            return Err(Error::InvalidArgument);
        }
        let mut handle = MaybeUninit::uninit();
        let res = &mut *pool.res;
        let handle = unsafe {
            (res.device.fun.allocate_descriptor_sets)(
                res.device.handle(),
                &DescriptorSetAllocateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    descriptor_pool: res.handle.borrow_mut(),
                    set_layouts: (&[layout.handle()]).into(),
                },
                std::array::from_mut(&mut handle).into(),
            )?;
            handle.assume_init()
        };
        let mut resources = vec![];
        for binding in &layout.bindings {
            resources.push(vec![None; binding.descriptor_count as usize]);
        }
        Ok(DescriptorSet {
            handle,
            layout: layout.clone(),
            _pool: Subobject::new(&pool.res),
            _allocation: pool.allocated.clone(),
            resources,
        })
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkDescriptorSet> {
        self.handle.borrow()
    }
    /// Mutably borrows the inner Vulkan handle.
    pub fn handle_mut(&mut self) -> Mut<VkDescriptorSet> {
        self.handle.borrow_mut()
    }
    /// Returns the set's layout.
    pub fn layout(&self) -> &Arc<DescriptorSetLayout> {
        &self.layout
    }
    /// Returns true if every member of the set has had a value written to it.
    pub fn is_initialized(&self) -> bool {
        self.resources.iter().all(|rs| rs.iter().all(|r| r.is_some()))
    }
}

/// An object to build calls to vkUpdateDescriptorSets. It's best to re-use it
/// as much as possible, since it holds onto some memory to avoid allocating.
///  
#[doc = crate::man_link!(vkUpdateDescriptorSets)]
pub struct DescriptorSetUpdateBuilder {
    device: Arc<Device>,
    scratch: Exclusive<bumpalo::Bump>,
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
    pub fn begin<'a>(&'a mut self) -> DescriptorSetUpdates<'a> {
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
    updates: DescriptorSetUpdates<'a>,
    set: &'a mut DescriptorSet,
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
    fn end(mut self) {
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

    fn set_ref(&mut self) -> Mut<'a, VkDescriptorSet> {
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

    fn buffers_impl(
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

    fn images_impl(
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
