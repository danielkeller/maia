use std::mem::MaybeUninit;

use crate::types::*;

impl Instance {
    pub fn enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>> {
        let mut len = 0;
        let mut result = Vec::new();
        unsafe {
            (self.0.fun.enumerate_physical_devices)(
                self.as_ref(),
                &mut len,
                None,
            )?;
            result.reserve(len.try_into().unwrap());
            (self.0.fun.enumerate_physical_devices)(
                self.as_ref(),
                &mut len,
                result.spare_capacity_mut().first_mut(),
            )?;
            result.set_len(len.try_into().unwrap());
        }
        Ok(result
            .into_iter()
            .map(|handle| PhysicalDevice::new(handle, self.0.clone()))
            .collect())
    }
}
