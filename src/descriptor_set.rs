use std::mem::MaybeUninit;

use crate::buffer::Buffer;
use crate::device::Device;
use crate::enums::{DescriptorType, ShaderStageFlags};
use crate::error::{Error, Result};
use crate::exclusive::Exclusive;
use crate::ffi::Array;
use crate::subobject::{Owner, Subobject};
use crate::types::*;

use bumpalo::collections::Vec as BumpVec;

pub struct DescriptorSetLayout {
    handle: Handle<VkDescriptorSetLayout>,
    bindings: Vec<DescriptorSetLayoutBinding>,
    device: Arc<Device>,
}

pub struct DescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: ShaderStageFlags,
    pub immutable_samplers: Vec<()>, //TODO
}

impl Device {
    pub fn create_descriptor_set_layout(
        self: &Arc<Self>,
        bindings: Vec<DescriptorSetLayoutBinding>,
    ) -> Result<Arc<DescriptorSetLayout>> {
        for b in &bindings {
            if b.immutable_samplers.len() != 0
                && b.immutable_samplers.len() as u32 != b.descriptor_count
            {
                return Err(Error::InvalidArgument);
            }
        }
        let vk_bindings = bindings
            .iter()
            .map(|b| VkDescriptorSetLayoutBinding {
                binding: b.binding,
                descriptor_type: b.descriptor_type,
                descriptor_count: b.descriptor_count,
                stage_flags: b.stage_flags,
                immutable_samplers: None,
            })
            .collect::<Vec<_>>();
        let mut handle = None;
        unsafe {
            (self.fun.create_descriptor_set_layout)(
                self.borrow(),
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
            device: self.clone(),
        }))
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_descriptor_set_layout)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl DescriptorSetLayout {
    pub fn borrow(&self) -> Ref<VkDescriptorSetLayout> {
        self.handle.borrow()
    }
}

struct DescriptorPoolLifetime {
    handle: Handle<VkDescriptorPool>,
    device: Arc<Device>,
}

struct AllocatedSets;

pub struct DescriptorPool {
    res: Owner<DescriptorPoolLifetime>,
    allocated: Arc<AllocatedSets>,
}

impl Device {
    pub fn create_descriptor_pool(
        self: &Arc<Device>,
        max_sets: u32,
        pool_sizes: &[DescriptorPoolSize],
    ) -> Result<DescriptorPool> {
        let mut handle = None;
        unsafe {
            (self.fun.create_descriptor_pool)(
                self.borrow(),
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
            device: self.clone(),
        });
        let allocated = Arc::new(AllocatedSets);
        Ok(DescriptorPool { res, allocated })
    }
}

impl Drop for DescriptorPoolLifetime {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_descriptor_pool)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl DescriptorPool {
    /// All descriptor sets allocated from the pool must have been dropped.
    pub fn reset(&mut self) -> Result<()> {
        if Arc::get_mut(&mut self.allocated).is_none() {
            return Err(Error::SynchronizationError);
        }
        let res = &mut *self.res;
        unsafe {
            (res.device.fun.reset_descriptor_pool)(
                res.device.borrow(),
                res.handle.borrow_mut(),
                Default::default(),
            )?;
        }
        Ok(())
    }
}

pub struct DescriptorSet {
    handle: Handle<VkDescriptorSet>,
    layout: Arc<DescriptorSetLayout>,
    pool: Subobject<DescriptorPoolLifetime>,
    allocation: Arc<AllocatedSets>,
    resources: Vec<Vec<Option<Arc<dyn Send + Sync>>>>,
}

impl DescriptorPool {
    pub fn allocate(
        &mut self,
        layout: &Arc<DescriptorSetLayout>,
    ) -> Result<DescriptorSet> {
        if !Arc::ptr_eq(&self.res.device, &layout.device) {
            return Err(Error::InvalidArgument);
        }
        let mut handle = MaybeUninit::uninit();
        let res = &mut *self.res;
        let handle = unsafe {
            (res.device.fun.allocate_descriptor_sets)(
                res.device.borrow(),
                &DescriptorSetAllocateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    descriptor_pool: res.handle.borrow_mut(),
                    set_layouts: (&[layout.borrow()]).into(),
                },
                std::array::from_mut(&mut handle).into(),
            )?;
            handle.assume_init()
        };
        let mut resources = vec![];
        for binding in &layout.bindings {
            let len = if binding.descriptor_type
                == DescriptorType::INLINE_UNIFORM_BLOCK
            {
                0
            } else {
                binding.descriptor_count as usize
            };
            resources.push(vec![None; len]);
        }
        Ok(DescriptorSet {
            handle,
            layout: layout.clone(),
            pool: Subobject::new(&self.res),
            allocation: self.allocated.clone(),
            resources,
        })
    }
}

impl DescriptorSet {
    pub fn borrow(&self) -> Ref<VkDescriptorSet> {
        self.handle.borrow()
    }
    pub fn borrow_mut(&mut self) -> Mut<VkDescriptorSet> {
        self.handle.borrow_mut()
    }
}

pub struct DescriptorSetUpdateBuilder {
    device: Arc<Device>,
    scratch: Exclusive<bumpalo::Bump>,
}

impl Device {
    pub fn create_descriptor_set_update_builder(
        self: &Arc<Self>,
    ) -> DescriptorSetUpdateBuilder {
        DescriptorSetUpdateBuilder {
            scratch: Exclusive::new(bumpalo::Bump::new()),
            device: self.clone(),
        }
    }
}

struct Resource {
    set: usize,
    binding: usize,
    element: usize,
    resource: Arc<dyn Send + Sync>,
}

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

#[must_use = "This object does nothing until end() is called."]
pub struct DescriptorSetUpdate<'a> {
    updates: DescriptorSetUpdates<'a>,
    set: &'a mut DescriptorSet,
}

impl<'a> DescriptorSetUpdates<'a> {
    pub fn dst_set(
        self,
        set: &'a mut DescriptorSet,
    ) -> DescriptorSetUpdate<'a> {
        DescriptorSetUpdate { updates: self, set }
    }
    fn end(mut self) {
        for res in self.resources {
            self.dst_sets[res.set].resources[res.binding][res.element] =
                Some(res.resource);
        }
        unsafe {
            (self.device.fun.update_descriptor_sets)(
                self.device.borrow(),
                self.writes.len() as u32,
                Array::from_slice(&self.writes),
                self.copies.len() as u32,
                Array::from_slice(&self.copies),
            )
        }
    }
}

pub struct DescriptorBufferInfo<'a> {
    pub buffer: &'a Arc<Buffer>,
    pub offset: u64,
    pub range: u64,
}

impl<'a> DescriptorSetUpdate<'a> {
    fn set_ref(&mut self) -> Mut<'a, VkDescriptorSet> {
        // Safety: The set is kept mutably borrowed while the builder
        // is alive, and one call to vkUpdateDescriptorSets counts as
        // a single use as far as external synchronization is concerned
        unsafe { self.set.handle.borrow_mut().reborrow_mut_unchecked() }
    }
    pub fn dst_set(
        mut self,
        set: &'a mut DescriptorSet,
    ) -> DescriptorSetUpdate<'a> {
        self.updates.dst_sets.push(self.set);
        self.updates.dst_set(set)
    }
    pub fn end(mut self) {
        self.updates.dst_sets.push(self.set);
        self.updates.end()
    }
    pub fn uniform_buffers(
        mut self,
        dst_binding: u32,
        dst_array_element: u32,
        buffers: &'_ [DescriptorBufferInfo<'a>],
    ) -> Result<Self> {
        let mut binding = dst_binding as usize;
        let mut element = dst_array_element;
        let bindings = &self.set.layout.bindings;
        for b in buffers {
            if binding >= bindings.len() {
                return Err(Error::InvalidArgument);
            }
            while element >= bindings[binding].descriptor_count {
                element -= bindings[binding].descriptor_count;
                binding += 1;
                if binding >= bindings.len() {
                    return Err(Error::InvalidArgument);
                }
            }
            if bindings[binding].descriptor_type
                != DescriptorType::UNIFORM_BUFFER
            {
                return Err(Error::InvalidArgument);
            }
            self.updates.resources.push(Resource {
                set: self.updates.dst_sets.len(),
                binding,
                element: element as usize,
                resource: b.buffer.clone(),
            });
            element += 1;
        }

        let buffer_infos =
            self.updates.bump.alloc_slice_fill_iter(buffers.iter().map(|b| {
                VkDescriptorBufferInfo {
                    buffer: b.buffer.borrow(),
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
            descriptor_type: DescriptorType::UNIFORM_BUFFER,
            image_info: None,
            buffer_info: Array::from_slice(buffer_infos),
            texel_buffer_view: None,
        });
        Ok(self)
    }
}
