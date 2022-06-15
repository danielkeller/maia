use std::mem::MaybeUninit;

use crate::device::Device;
use crate::error::Result;
use crate::ffi::ArrayMut;
use crate::instance::Instance;
use crate::types::*;

#[derive(Debug)]
pub struct PhysicalDevice {
    handle: Handle<VkPhysicalDevice>,
    pub(crate) instance: Arc<Instance>,
}

impl PhysicalDevice {
    pub(crate) fn new(
        handle: Handle<VkPhysicalDevice>,
        instance: Arc<Instance>,
    ) -> Self {
        Self { handle, instance }
    }
    pub fn handle(&self) -> Ref<VkPhysicalDevice> {
        self.handle.borrow()
    }
}

impl Clone for PhysicalDevice {
    fn clone(&self) -> Self {
        Self {
            // Safety: phyiscal device has no externally synchronized functions
            // and is not freed
            handle: unsafe { self.handle.clone() },
            instance: self.instance.clone(),
        }
    }
}

impl PhysicalDevice {
    pub fn properties(&self) -> PhysicalDeviceProperties {
        let mut result = MaybeUninit::uninit();
        unsafe {
            (self.instance.fun.get_physical_device_properties)(
                self.handle(),
                &mut result,
            );
            result.assume_init()
        }
    }

    pub fn queue_family_properties(&self) -> Vec<QueueFamilyProperties> {
        let mut len = 0;
        let mut result = Vec::new();
        unsafe {
            (self.instance.fun.get_physical_device_queue_family_properties)(
                self.handle(),
                &mut len,
                None,
            );
            result.reserve(len as usize);
            (self.instance.fun.get_physical_device_queue_family_properties)(
                self.handle(),
                &mut len,
                ArrayMut::from_slice(result.spare_capacity_mut()),
            );
            result.set_len(len as usize);
        }
        result
    }

    pub fn memory_properties(&self) -> PhysicalDeviceMemoryProperties {
        let mut result = Default::default();
        unsafe {
            (self.instance.fun.get_physical_device_memory_properties)(
                self.handle(),
                &mut result,
            );
        }
        result
    }

    pub fn device_extension_properties(
        &self,
    ) -> Result<Vec<ExtensionProperties>> {
        let mut len = 0;
        let mut result = Vec::new();
        unsafe {
            (self.instance.fun.enumerate_device_extension_properties)(
                self.handle(),
                None,
                &mut len,
                None,
            )?;
            result.reserve(len as usize);
            (self.instance.fun.enumerate_device_extension_properties)(
                self.handle(),
                None,
                &mut len,
                ArrayMut::from_slice(result.spare_capacity_mut()),
            )?;
            result.set_len(len as usize);
        }
        Ok(result)
    }

    pub fn create_device(
        &self,
        info: &DeviceCreateInfo<'_>,
    ) -> Result<Arc<Device>> {
        let props = self.queue_family_properties();
        let mut queues = vec![0; props.len()];
        for q in info.queue_create_infos.as_slice() {
            let i = q.queue_family_index as usize;
            assert!(i < props.len(), "Queue family index out of bounds");
            assert!(
                q.queue_priorities.len() <= props[i].queue_count,
                "Too many queues requested"
            );
            queues[i] = q.queue_priorities.len();
        }

        let mut handle = None;
        unsafe {
            (self.instance.fun.create_device)(
                self.handle(),
                info,
                None,
                &mut handle,
            )?;
        }
        Ok(Device::new(handle.unwrap(), self.clone(), queues))
    }
}
