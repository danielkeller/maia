// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Weak;

use crate::cleanup::Cleanup;
use crate::instance::Instance;
use crate::load::DeviceFn;
use crate::physical_device::{make_api_version, PhysicalDevice};
use crate::queue::Queue;
use crate::types::*;

struct Impl {
    handle: Handle<VkDevice>,
    fun: DeviceFn,
    physical_device: PhysicalDevice,
    limits: PhysicalDeviceLimits,
    enabled: PhysicalDeviceFeatures,
    memory_allocation_count: AtomicU32,
    sampler_allocation_count: AtomicU32,
    queues: Vec<u32>,
    cleanup: Weak<Cleanup>,
    // Maybe include device_lost so we don't double panic all the time
}

/// A logical device.
#[derive(Clone)]
pub struct Device {
    inner: Arc<Impl>,
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.handle.fmt(f)
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for Device {}
impl std::hash::Hash for Device {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&*self.inner as *const _ as usize).hash(state)
    }
}

impl Drop for Impl {
    fn drop(&mut self) {
        unsafe {
            (self.fun.destroy_device)(self.handle.borrow_mut(), None);
        }
    }
}

impl Device {
    /// Create a logical device for this physical device. Queues are returned in
    /// the order requested in `info.queue_create_infos`. The physical device
    /// version must be >=1.1.
    #[doc = crate::man_link!(vkCreateDevice)]
    pub fn new(
        phy: &PhysicalDevice, info: &DeviceCreateInfo<'_>,
    ) -> (Self, Vec<Vec<Queue>>) {
        let phy_properties = phy.properties();
        assert!(phy_properties.api_version >= make_api_version(0, 1, 1, 0));
        assert!(phy_properties.api_version < make_api_version(0, 2, 0, 0));
        let props = phy.queue_family_properties();
        let mut queues = vec![0; props.len()];
        for q in info.queue_create_infos {
            let i = q.queue_family_index as usize;
            if i >= props.len() {
                panic!("Queue family index {i} out of bounds")
            }
            if q.queue_priorities.len() > props[i].queue_count {
                panic!(
                    "Too many queues for family {i}: {} > {}",
                    q.queue_priorities.len(),
                    props[i].queue_count
                )
            }
            queues[i] = q.queue_priorities.len();
        }

        let mut handle = None;
        unsafe {
            (phy.instance().fun().create_device)(
                phy.handle(),
                info,
                None,
                &mut handle,
            )
            .unwrap();
        }
        let handle = handle.unwrap();
        let fun = DeviceFn::new(phy.instance(), handle.borrow());

        let cleanup = Arc::new_cyclic(|cleanup| {
            let inner = Impl {
                handle,
                fun,
                physical_device: phy.clone(),
                limits: phy_properties.limits,
                enabled: info.enabled_features.cloned().unwrap_or_default(),
                memory_allocation_count: 0.into(),
                sampler_allocation_count: 0.into(),
                queues,
                cleanup: cleanup.clone(),
            };
            let this = Device { inner: Arc::new(inner) };
            Cleanup::new(&this)
        });
        let this = cleanup.device().clone();

        let queues = this
            .inner
            .queues
            .iter()
            .enumerate()
            .map(|(i, &n)| {
                (0..n).map(|n| Queue::new(&cleanup, i as u32, n)).collect()
            })
            .collect();
        (this, queues)
    }

    /// Get the device functions.
    pub fn fun(&self) -> &DeviceFn {
        &self.inner.fun
    }

    /// Returns the cleanup object, if it still exists. If this function returns
    /// None, the device is idle and any resource can be cleaned up immediately.
    pub fn cleanup(&self) -> Option<Arc<Cleanup>> {
        self.inner.cleanup.upgrade()
    }

    /// Dispose of the resource at the next safe time.
    pub fn dispose(&self, resource: impl Send + Sync + 'static) {
        if let Some(cleanup) = self.cleanup() {
            cleanup.dispose(resource);
        }
    }
}

impl Device {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkDevice> {
        self.inner.handle.borrow()
    }
    /// Returns the limits of the device.
    pub fn limits(&self) -> &PhysicalDeviceLimits {
        &self.inner.limits
    }
    /// Returns the enabled features.
    pub fn enabled(&self) -> &PhysicalDeviceFeatures {
        &self.inner.enabled
    }
    /// Returns the associated phyical device.
    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.inner.physical_device
    }
    /// Returns the associated instance.
    pub fn instance(&self) -> &Instance {
        self.inner.physical_device.instance()
    }
    /// Returns true if a queue with this family index and index exists.
    pub fn has_queue(&self, queue_family_index: u32, queue_index: u32) -> bool {
        let i = queue_family_index as usize;
        i < self.inner.queues.len() && self.inner.queues[i] >= queue_index
    }
    pub(crate) fn increment_memory_alloc_count(&self) {
        // Reserve allocation number 'val'.
        // Overflow is incredibly unlikely here
        let val =
            self.inner.memory_allocation_count.fetch_add(1, Ordering::Relaxed);
        if val >= self.inner.limits.max_memory_allocation_count {
            self.inner.memory_allocation_count.fetch_sub(1, Ordering::Relaxed);
            panic!(
                "Too many memory allocations: {val} > {}",
                self.inner.limits.max_memory_allocation_count
            )
        }
    }
    pub(crate) fn decrement_memory_alloc_count(&self) {
        self.inner.memory_allocation_count.fetch_sub(1, Ordering::Relaxed);
    }
    pub(crate) fn increment_sampler_alloc_count(&self) {
        // Reserve allocation number 'val'.
        // Overflow is incredibly unlikely here
        let val =
            self.inner.sampler_allocation_count.fetch_add(1, Ordering::Relaxed);
        if val >= self.inner.limits.max_sampler_allocation_count {
            self.inner.sampler_allocation_count.fetch_sub(1, Ordering::Relaxed);
            panic!(
                "Too many sampler objects: {val} > {}",
                self.inner.limits.max_sampler_allocation_count
            )
        }
    }
    pub(crate) fn decrement_sampler_alloc_count(&self) {
        self.inner.sampler_allocation_count.fetch_sub(1, Ordering::Relaxed);
    }
}
