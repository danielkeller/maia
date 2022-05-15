use std::ffi::c_void;
use std::ptr::NonNull;

use crate::error::Result;
use crate::ffi::ArrayMut;
use crate::load::InstanceFn;
use crate::physical_device::PhysicalDevice;
use crate::types::*;

pub struct Instance {
    handle: Handle<VkInstance>,
    pub(crate) fun: InstanceFn,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { (self.fun.destroy_instance)(self.handle.borrow_mut(), None) }
    }
}

impl Instance {
    pub(crate) fn new(handle: Handle<VkInstance>) -> Arc<Self> {
        let fun = InstanceFn::new(handle.borrow());
        Arc::new(Instance { handle, fun })
    }
    pub fn borrow(&self) -> Ref<VkInstance> {
        self.handle.borrow()
    }
}

impl Instance {
    /// Load instance function. Panics if the string is not null-terminated or
    /// the function was not found.
    pub fn get_proc_addr(&self, name: &str) -> NonNull<c_void> {
        crate::load::load(Some(self.borrow()), name)
    }

    pub fn enumerate_physical_devices(
        self: &Arc<Self>,
    ) -> Result<Vec<PhysicalDevice>> {
        let mut len = 0;
        let mut result = Vec::new();
        unsafe {
            (self.fun.enumerate_physical_devices)(
                self.borrow(),
                &mut len,
                None,
            )?;
            result.reserve(len as usize);
            (self.fun.enumerate_physical_devices)(
                self.borrow(),
                &mut len,
                ArrayMut::from_slice(result.spare_capacity_mut()),
            )?;
            result.set_len(len as usize);
        }
        Ok(result
            .into_iter()
            .map(|handle| PhysicalDevice::new(handle, self.clone()))
            .collect())
    }
}
