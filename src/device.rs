use std::sync::atomic::AtomicU32;

use crate::error::{Error, Result};
use crate::instance::Instance;
use crate::load::DeviceFn;
use crate::physical_device::PhysicalDevice;
use crate::queue::Queue;
use crate::types::*;

/// A logical device.
pub struct Device {
    handle: Handle<VkDevice>,
    pub(crate) fun: DeviceFn,
    physical_device: PhysicalDevice,
    limits: PhysicalDeviceLimits,
    memory_allocation_count: AtomicU32,
    sampler_allocation_count: AtomicU32,
    queues: Vec<u32>,
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}
impl Eq for Device {}
impl std::hash::Hash for Device {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.handle.hash(state)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            (self.fun.device_wait_idle)(self.handle.borrow_mut()).unwrap();
            (self.fun.destroy_device)(self.handle.borrow_mut(), None);
        }
    }
}

impl Device {
    /// Create a logical device for this physical device. Queues are returned in
    /// the order requested in `info.queue_create_infos`.
    #[doc = crate::man_link!(vkCreateDevice)]
    pub fn new(
        phy: &PhysicalDevice,
        info: &DeviceCreateInfo<'_>,
    ) -> Result<(Arc<Self>, Vec<Vec<Queue>>)> {
        let props = phy.queue_family_properties();
        let mut queues = vec![0; props.len()];
        for q in info.queue_create_infos {
            let i = q.queue_family_index as usize;
            if i >= props.len()
                || q.queue_priorities.len() > props[i].queue_count
            {
                return Err(Error::OutOfBounds);
            }
            queues[i] = q.queue_priorities.len();
        }

        let mut handle = None;
        unsafe {
            (phy.instance().fun.create_device)(
                phy.handle(),
                info,
                None,
                &mut handle,
            )?;
        }
        let handle = handle.unwrap();
        let fun = DeviceFn::new(phy.instance(), handle.borrow());
        let device = Arc::new(Device {
            handle,
            fun,
            physical_device: phy.clone(),
            limits: phy.properties().limits,
            memory_allocation_count: AtomicU32::new(0),
            sampler_allocation_count: AtomicU32::new(0),
            queues,
        });
        let queues = info
            .queue_create_infos
            .into_iter()
            .map(|q| {
                (0..q.queue_priorities.len())
                    .map(|n| device.queue(q.queue_family_index, n))
                    .collect()
            })
            .collect();
        Ok((device, queues))
    }
}

impl Device {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkDevice> {
        self.handle.borrow()
    }
    /// Returns the limits of the device.
    pub fn limits(&self) -> &PhysicalDeviceLimits {
        &self.limits
    }
    /// Returns the associated phyical device.
    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }
    /// Returns the associated instance.
    pub fn instance(&self) -> &Arc<Instance> {
        self.physical_device.instance()
    }
    /// Returns true if a queue with this family index and index exists.
    pub fn has_queue(&self, queue_family_index: u32, queue_index: u32) -> bool {
        let i = queue_family_index as usize;
        i < self.queues.len() && self.queues[i] >= queue_index
    }
    pub(crate) fn increment_memory_alloc_count(&self) -> Result<()> {
        use std::sync::atomic::Ordering;
        // Reserve allocation number 'val'.
        // Overflow is incredibly unlikely here
        let val = self.memory_allocation_count.fetch_add(1, Ordering::Relaxed);
        if val >= self.limits.max_memory_allocation_count {
            self.memory_allocation_count.fetch_sub(1, Ordering::Relaxed);
            Err(Error::LimitExceeded)
        } else {
            Ok(())
        }
    }
    pub(crate) fn decrement_memory_alloc_count(&self) {
        use std::sync::atomic::Ordering;
        self.memory_allocation_count.fetch_sub(1, Ordering::Relaxed);
    }
    pub(crate) fn increment_sampler_alloc_count(&self) -> Result<()> {
        use std::sync::atomic::Ordering;
        // Reserve allocation number 'val'.
        // Overflow is incredibly unlikely here
        let val = self.sampler_allocation_count.fetch_add(1, Ordering::Relaxed);
        if val >= self.limits.max_sampler_allocation_count {
            self.sampler_allocation_count.fetch_sub(1, Ordering::Relaxed);
            Err(Error::LimitExceeded)
        } else {
            Ok(())
        }
    }
    pub(crate) fn decrement_sampler_alloc_count(&self) {
        use std::sync::atomic::Ordering;
        self.sampler_allocation_count.fetch_sub(1, Ordering::Relaxed);
    }
}
