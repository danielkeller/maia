use std::ffi::c_void;
use std::ptr::NonNull;

use crate::error::{Error, Result};
use crate::load::DeviceFn;
use crate::physical_device::PhysicalDevice;
use crate::queue::Queue;
use crate::types::*;

pub struct Device {
    handle: Handle<VkDevice>,
    pub(crate) fun: DeviceFn,
    physical_device: PhysicalDevice,
    pub(crate) queues: Vec<u32>,
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
    pub(crate) fn new(
        handle: Handle<VkDevice>,
        physical_device: PhysicalDevice,
        queues: Vec<u32>,
    ) -> Arc<Self> {
        let fun = DeviceFn::new(&physical_device.instance, handle.borrow());
        Arc::new(Device { handle, fun, physical_device, queues })
    }
    pub fn handle(&self) -> Ref<VkDevice> {
        self.handle.borrow()
    }
    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }
}

impl Device {
    /// Load device function. Panics if the string is not null-terminated or the
    /// function was not found.
    pub fn get_proc_addr(&self, name: &str) -> NonNull<c_void> {
        self.physical_device.instance.load(self.handle(), name)
    }

    pub fn queue(
        self: &Arc<Self>,
        family_index: u32,
        queue_index: u32,
    ) -> Result<Queue> {
        let i = family_index as usize;
        if i > self.queues.len() || self.queues[i] <= queue_index {
            return Err(Error::OutOfBounds);
        }
        let mut handle = None;
        unsafe {
            (self.fun.get_device_queue)(
                self.handle(),
                family_index,
                queue_index,
                &mut handle,
            );
        }
        Ok(Queue::new(handle.unwrap(), self.clone()))
    }
}
