// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc;

use atomic_refcell::AtomicRefCell;

use crate::buffer::Buffer;
use crate::device::Device;
use crate::enums::*;
use crate::enums::{DescriptorType, ShaderStageFlags};
use crate::error::OutOfPoolMemory;
use crate::ffi::Array;
use crate::image::ImageView;
use crate::sampler::Sampler;
use crate::subobject::{Owner, Subobject};
use crate::types::*;

pub mod update;

// I guess that if the buffers and stuff stored their handle inline, you could
// make a derive macro for DescriptorUpdateTemplate.

#[derive(Debug)]
struct DescriptorSetLayoutInner {
    handle: Handle<VkDescriptorSetLayout>,
    bindings: Vec<DescriptorSetLayoutBinding>,
    device: Device,
}

/// A
#[doc = crate::spec_link!("descriptor set layout", "14", "descriptorsets-setlayout")]
#[derive(Debug, Clone)]
pub struct DescriptorSetLayout {
    inner: Arc<DescriptorSetLayoutInner>,
}

/// Note that unlike in Vulkan, the binding number is implicitly the index of
/// the array that is passed into [`DescriptorSetLayout::new`].
/// If non-consecutive binding numbers are desired (not recommended for
/// performance by the way), create dummy descriptors to fill the gaps.
///
/// For [`DescriptorType::COMBINED_IMAGE_SAMPLER`], currently the use of
/// immutable samplers is required.
///
#[doc = crate::man_link!(VkDescriptorSetLayoutBinding)]
#[derive(Debug, PartialEq, Eq, Default)]
pub struct DescriptorSetLayoutBinding {
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: ShaderStageFlags,
    pub immutable_samplers: Vec<Sampler>, // TODO: Support sharing.
}

impl DescriptorSetLayout {
    // TODO: vkGetDescriptorSetLayoutSupport

    #[doc = crate::man_link!(VkDescriptorSetLayout)]
    pub fn new(
        device: &Device, bindings: Vec<DescriptorSetLayoutBinding>,
    ) -> Self {
        for b in &bindings {
            if !b.immutable_samplers.is_empty()
                && b.immutable_samplers.len() as u32 != b.descriptor_count
            {
                panic!(
                    "Immutable samplers must have either 0 or {} items",
                    b.descriptor_count
                );
            }
            if b.descriptor_type == DescriptorType::COMBINED_IMAGE_SAMPLER
                && b.immutable_samplers.is_empty()
            {
                panic!("Combined image samplers require immutable samplers")
            }
        }

        let mut samplers = vec![];
        for b in &bindings {
            samplers.extend(b.immutable_samplers.iter().map(|s| s.handle()));
        }

        let mut s_i = 0;
        let mut vk_bindings = vec![];
        for (i, b) in bindings.iter().enumerate() {
            let s_j = s_i + b.immutable_samplers.len();
            let immutable_samplers = Array::from_slice(&samplers[s_i..s_j]);
            s_i = s_j;

            vk_bindings.push(VkDescriptorSetLayoutBinding {
                binding: i as u32,
                descriptor_type: b.descriptor_type,
                descriptor_count: b.descriptor_count,
                stage_flags: b.stage_flags,
                immutable_samplers,
            });
        }
        let mut handle = None;
        unsafe {
            (device.fun().create_descriptor_set_layout)(
                device.handle(),
                &VkDescriptorSetLayoutCreateInfo {
                    bindings: vk_bindings.as_slice().into(),
                    ..Default::default()
                },
                None,
                &mut handle,
            )
            .unwrap();
        }

        let inner = Arc::new(DescriptorSetLayoutInner {
            handle: handle.unwrap(),
            bindings,
            device: device.clone(),
        });
        DescriptorSetLayout { inner }
    }
}

impl Drop for DescriptorSetLayoutInner {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun().destroy_descriptor_set_layout)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl PartialEq for DescriptorSetLayout {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for DescriptorSetLayout {}
impl std::hash::Hash for DescriptorSetLayout {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&*self.inner as *const _ as usize).hash(state)
    }
}

impl DescriptorSetLayout {
    /// Borrows the inner Vulkan handle.
    pub fn borrow(&self) -> Ref<VkDescriptorSetLayout> {
        self.inner.handle.borrow()
    }
    /// Returns the number of dynamic offsets the descriptor set will require.
    pub(crate) fn num_dynamic_offsets(&self) -> u32 {
        let mut result = 0;
        for b in &self.inner.bindings {
            if b.descriptor_type == DescriptorType::UNIFORM_BUFFER_DYNAMIC
                || b.descriptor_type == DescriptorType::STORAGE_BUFFER_DYNAMIC
            {
                result += b.descriptor_count
            }
        }
        result
    }
    /// Returns the number of bindings of the specified type and stage.
    pub(crate) fn num_bindings(
        &self, descriptor_type: DescriptorType, stage_flags: ShaderStageFlags,
    ) -> u32 {
        self.inner
            .bindings
            .iter()
            .filter(|b| {
                b.descriptor_type == descriptor_type
                    && !(b.stage_flags & stage_flags).is_empty()
            })
            .count() as u32
    }

    pub(crate) fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }

    pub(crate) fn device(&self) -> &Device {
        &self.inner.device
    }

    pub(crate) fn bindings(&self) -> &[DescriptorSetLayoutBinding] {
        &self.inner.bindings
    }
}

#[derive(Debug)]
struct DescriptorPoolInner {
    handle: Handle<VkDescriptorPool>,
    device: Device,
}

/// A
#[doc = crate::spec_link!("descriptor pool", "14", "descriptorsets-allocation")]
/// .
#[derive(Debug)]
pub struct DescriptorPool {
    inner: Owner<DescriptorPoolInner>,
}

impl DescriptorPool {
    #[doc = crate::man_link!(vkCreateDescriptorPool)]
    pub fn new(
        device: &Device, max_sets: u32, pool_sizes: &[DescriptorPoolSize],
    ) -> Self {
        let mut handle = None;
        unsafe {
            (device.fun().create_descriptor_pool)(
                device.handle(),
                &DescriptorPoolCreateInfo {
                    max_sets,
                    pool_sizes: pool_sizes.into(),
                    ..Default::default()
                },
                None,
                &mut handle,
            )
            .unwrap();
        }
        let inner = DescriptorPoolInner {
            handle: handle.unwrap(),
            device: device.clone(),
        };
        DescriptorPool { inner: inner.into() }
    }

    #[doc = crate::man_link!(vkResetDescriptorPool)]
    pub fn reset(&mut self) -> bool {
        if !self.inner.is_unique() {
            return false;
        }
        let inner = &mut *self.inner;
        unsafe {
            (inner.device.fun().reset_descriptor_pool)(
                inner.device.handle(),
                inner.handle.borrow_mut(),
                Default::default(),
            )
            .unwrap();
        }
        return true;
    }

    /// Returns the associated device.
    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

impl Drop for DescriptorPoolInner {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun().destroy_descriptor_pool)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

// Theoretically the descriptor set layout can be destroyed while the set is
// still in use, only creating and updating the set need it. But the immutable
// samplers still need to outlive the set, and this shorter lifetime is tricky
// to allow, so we force the layout to outlive the set as well. Also, updating
// the set is easier when we have the layout.
// We could also store an Arc with the sampler in the set...

// The problem with making descriptor sets mutable is that you don't want to
// have to wait for the GPU to be done with them. (Just like command pools!)
// In particular because background operations can cause arbitrary delays with
// the epoch model.

// DSes are sort of the odd duck here, since they're the only object that you
// really modify after creating.

/*
The convenient API:

I want this descriptor set with this contents, make it happen! (re-use/recycle)
So dropped DSes get returned to a free list when the GPU is done with them.

Free list can just be implemented by the application I think

(Issues with pool fragmentation if we use a lot of layouts, maybe 1 pool per
layout is better. Also when do you reset the pool? Never I guess.)

The cleanup would be some kind of SmallBox<dyn FnOnce()>? SmallBox<dyn Drop>?
Or an Arc so we can clean up conditionally on the refcount?


The DS can be more or less
struct DS(Arc<Pool>, usize); The refcount of the DS is internal to the pool
and only affect when it can be reused. The benefit of this is that it's easy to
"send it back" to the freelist by marking it as free through the arc and
dropping it. This also lets you drop and bring back DSes without allocating.

Where do you put resource references then?
*/

#[doc = crate::man_link!(VkDescriptorBufferInfo)]
#[doc = crate::man_link!(VkDescriptorImageInfo)]
#[non_exhaustive]
#[derive(Debug, Default)]
pub enum Descriptor {
    #[default]
    Null,
    Buffer(Buffer, std::ops::RangeFrom<u64>),
    BufferRange(Buffer, std::ops::Range<u64>),
    Sampler(Sampler),
    Image(ImageView, ImageLayout),
    ImageSampler(Sampler, ImageView, ImageLayout),
}

#[derive(Debug)]
struct DescriptorSetInner {
    handle: Handle<VkDescriptorSet>,
    pool: Subobject<DescriptorPoolInner>,
    layout: DescriptorSetLayout,
    resources: Vec<Descriptor>,
}

/// A
#[doc = concat!(crate::spec_link!("descriptor set", "14", "descriptorsets-sets"), ".")]
///
/// The descriptor set may be written to by creating a
/// [`DescriptorSetUpdateBuilder`](crate::vk::DescriptorSetUpdateBuilder).
///
/// Any resources that are written into the descriptor set have their reference
/// count incremented and held by the set. To decrement the count and allow the
/// resources to be freed, either the descriptor must be overwritten with
/// another resource, or the descriptor set must be dropped.
#[derive(Debug, Clone)]
struct DescriptorSet {
    inner: Arc<DescriptorSetInner>,
}

impl DescriptorSet {
    #[doc = crate::man_link!(vkAllocateDescriptorSets)]
    pub fn new(
        pool: &mut DescriptorPool, layout: &DescriptorSetLayout,
        contents: &[Descriptor],
    ) -> Result<Self, OutOfPoolMemory> {
        assert_eq!(pool.device(), &layout.inner.device);
        let mut handle = MaybeUninit::uninit();
        let handle = unsafe {
            (pool.device().fun().allocate_descriptor_sets)(
                pool.device().handle(),
                &DescriptorSetAllocateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    descriptor_pool: pool.inner.handle.borrow_mut(),
                    set_layouts: (&[layout.borrow()]).into(),
                },
                std::array::from_mut(&mut handle).into(),
            )
            .unwrap_or_oopm()?;
            handle.assume_init()
        };
        let size: u32 =
            layout.bindings().iter().map(|b| b.descriptor_count).sum();
        Ok(DescriptorSet { handle, layout: layout.clone(), shadow })
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkDescriptorSet> {
        self.handle.borrow()
    }
    /// Returns the set's layout.
    pub fn layout(&self) -> &DescriptorSetLayout {
        &self.layout
    }
}
