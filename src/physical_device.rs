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
}
