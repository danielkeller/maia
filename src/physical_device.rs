use std::mem::MaybeUninit;

use crate::error::Result;
use crate::ffi::ArrayMut;
use crate::instance::Instance;
use crate::types::*;

/// A physical device. It is not freed separately from the instance and so can
/// be freely cloned.
///
/// Returned from [`Instance::enumerate_physical_devices`]
#[derive(Debug)]
pub struct PhysicalDevice {
    handle: Handle<VkPhysicalDevice>,
    instance: Arc<Instance>,
}

impl Instance {
    /// Returns the instance's physical devices.
    #[doc = crate::man_link!(vkEnumeratePhysicalDevices)]
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
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkPhysicalDevice> {
        self.handle.borrow()
    }
    /// Gets the instance.
    pub fn instance(&self) -> &Arc<Instance> {
        &self.instance
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
    #[doc = crate::man_link!(vkGetPhysicalDeviceProperties)]
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

    #[doc = crate::man_link!(vkGetPhysicalDeviceQueueFamilyProperties)]
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

    #[doc = crate::man_link!(vkGetPhysicalDeviceMemoryProperties)]
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

    #[doc = crate::man_link!(vkEnumerateDeviceExtensionProperties)]
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
