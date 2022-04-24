use std::mem::MaybeUninit;

use crate::types::*;

impl PhysicalDevice {
    pub fn properties(&self) -> PhysicalDeviceProperties {
        let mut result = MaybeUninit::uninit();
        unsafe {
            (self.instance.fun.get_physical_device_properties)(
                self.as_ref(),
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
                self.as_ref(),
                &mut len,
                None,
            );
            result.reserve(len.try_into().unwrap());
            (self.instance.fun.get_physical_device_queue_family_properties)(
                self.as_ref(),
                &mut len,
                result.spare_capacity_mut().first_mut(),
            );
            result.set_len(len.try_into().unwrap());
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
                self.as_ref(),
                None,
                &mut len,
                None,
            )?;
            result.reserve(len.try_into().unwrap());
            (self.instance.fun.enumerate_device_extension_properties)(
                self.as_ref(),
                None,
                &mut len,
                result.spare_capacity_mut().first_mut(),
            )?;
            result.set_len(len.try_into().unwrap());
        }
        Ok(result)
    }

    pub fn create_device(&self, info: &DeviceCreateInfo<'_>) -> Result<Device> {
        let props = self.queue_family_properties();
        let DeviceCreateInfo::S { queue_create_infos, .. } = info;
        for queue in queue_create_infos.as_slice() {
            let DeviceQueueCreateInfo::S {
                queue_family_index,
                queue_priorities,
                ..
            } = queue;
            let i = *queue_family_index as usize;
            if i > props.len() || queue_priorities.len() > props[i].queue_count
            {
                return Err(Error::INITIALIZATION_FAILED);
            }
        }

        let mut handle = None;
        unsafe {
            (self.instance.fun.create_device)(
                self.as_ref(),
                info,
                None,
                &mut handle,
            )?;
        }
        Ok(Device::new(handle.unwrap(), self.instance.clone()))
    }
}
