use std::mem::MaybeUninit;

use crate::device::Device;
use crate::error::Result;
use crate::instance::Instance;
use crate::types::*;

#[derive(Debug)]
pub struct PhysicalDevice {
    handle: Ref<'static, VkPhysicalDevice>,
    pub(crate) instance: Arc<Instance>,
}

impl PhysicalDevice {
    pub(crate) fn new(
        handle: Ref<'static, VkPhysicalDevice>,
        instance: Arc<Instance>,
    ) -> Self {
        Self { handle, instance }
    }
    pub fn phy_ref(&self) -> Ref<'_, VkPhysicalDevice> {
        self.handle
    }
}

impl PhysicalDevice {
    pub fn properties(&self) -> PhysicalDeviceProperties {
        let mut result = MaybeUninit::uninit();
        unsafe {
            (self.instance.fun.get_physical_device_properties)(
                self.phy_ref(),
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
                self.phy_ref(),
                &mut len,
                None,
            );
            result.reserve(len as usize);
            (self.instance.fun.get_physical_device_queue_family_properties)(
                self.phy_ref(),
                &mut len,
                result.spare_capacity_mut().first_mut(),
            );
            result.set_len(len as usize);
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
                self.phy_ref(),
                None,
                &mut len,
                None,
            )?;
            result.reserve(len as usize);
            (self.instance.fun.enumerate_device_extension_properties)(
                self.phy_ref(),
                None,
                &mut len,
                result.spare_capacity_mut().first_mut(),
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
                self.phy_ref(),
                info,
                None,
                &mut handle,
            )?;
        }
        Ok(Device::new(handle.unwrap(), self.instance.clone(), queues))
    }
}
