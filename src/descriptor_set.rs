use std::mem::MaybeUninit;

use crate::device::Device;
use crate::enums::{DescriptorType, ShaderStageFlags};
use crate::error::{Error, Result};
use crate::subobject::{Owner, Subobject};
use crate::types::*;

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
