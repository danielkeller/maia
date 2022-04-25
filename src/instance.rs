use std::sync::Arc;

use crate::load::InstanceFn;
use crate::physical_device::PhysicalDevice;
use crate::types::*;

pub struct Instance {
    handle: InstanceRef<'static>,
    pub(crate) fun: InstanceFn,
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { (self.fun.destroy_instance)(self.handle, None) }
    }
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl Instance {
    pub(crate) fn new(handle: InstanceRef<'static>) -> Arc<Self> {
        Arc::new(Instance { handle, fun: InstanceFn::new(handle) })
    }
    pub fn inst_ref(&self) -> InstanceRef<'_> {
        self.handle
    }
}

impl Instance {
    pub fn enumerate_physical_devices(
        self: &Arc<Self>,
    ) -> Result<Vec<PhysicalDevice>> {
        let mut len = 0;
        let mut result = Vec::new();
        unsafe {
            (self.fun.enumerate_physical_devices)(
                self.inst_ref(),
                &mut len,
                None,
            )?;
            result.reserve(len.try_into().unwrap());
            (self.fun.enumerate_physical_devices)(
                self.inst_ref(),
                &mut len,
                result.spare_capacity_mut().first_mut(),
            )?;
            result.set_len(len.try_into().unwrap());
        }
        Ok(result
            .into_iter()
            .map(|handle| PhysicalDevice::new(handle, self.clone()))
            .collect())
    }
}
