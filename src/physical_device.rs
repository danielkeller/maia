use std::mem::MaybeUninit;

use crate::error::Result;
use crate::ffi::ArrayMut;
use crate::instance::Instance;
use crate::types::*;

#[derive(Debug)]
pub struct PhysicalDevice {
    handle: Handle<VkPhysicalDevice>,
    instance: Arc<Instance>,
}

impl Instance {
    pub fn enumerate_physical_devices(
        self: &Arc<Self>,
    ) -> Result<Vec<PhysicalDevice>> {
        let mut len = 0;
        let mut result = Vec::new();
        unsafe {
            (self.fun.enumerate_physical_devices)(
                self.handle(),
                &mut len,
                None,
            )?;
            result.reserve(len as usize);
            (self.fun.enumerate_physical_devices)(
                self.handle(),
                &mut len,
                ArrayMut::from_slice(result.spare_capacity_mut()),
            )?;
            result.set_len(len as usize);
        }
        Ok(result
            .into_iter()
            .map(|handle| PhysicalDevice { handle, instance: self.clone() })
            .collect())
    }
}

impl PhysicalDevice {
    pub fn handle(&self) -> Ref<VkPhysicalDevice> {
        self.handle.borrow()
    }
    pub fn instance(&self) -> &Instance {
        &*self.instance
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
}
