use std::mem::MaybeUninit;

use super::load::SurfaceKHRFn;
use crate::enums::*;
use crate::error::Result;
use crate::ffi::ArrayMut;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;
use crate::subobject::Owner;
use crate::types::*;

pub(crate) struct SurfaceLifetime {
    handle: Handle<VkSurfaceKHR>,
    fun: SurfaceKHRFn,
    instance: Arc<Instance>,
}

#[derive(Debug)]
pub struct SurfaceKHR {
    pub(crate) res: Owner<SurfaceLifetime>,
}

impl std::fmt::Debug for SurfaceLifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurfaceResource").finish()
    }
}

impl Drop for SurfaceLifetime {
    fn drop(&mut self) {
        unsafe {
            (self.fun.destroy_surface_khr)(
                self.instance.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl SurfaceKHR {
    pub(crate) fn new(
        handle: Handle<VkSurfaceKHR>,
        instance: Arc<Instance>,
    ) -> Self {
        Self {
            res: Owner::new(SurfaceLifetime {
                handle,
                fun: SurfaceKHRFn::new(&instance),
                instance,
            }),
        }
    }

    pub fn handle(&self) -> Ref<VkSurfaceKHR> {
        self.res.handle.borrow()
    }
    pub fn handle_mut(&mut self) -> Mut<VkSurfaceKHR> {
        self.res.handle.borrow_mut()
    }

    pub fn support(
        &self,
        phy: &PhysicalDevice,
        queue_family: u32,
    ) -> Result<bool> {
        let mut result = Bool::False;
        assert!(Arc::ptr_eq(&self.res.instance, &phy.instance));
        assert!(
            (queue_family as usize) < phy.queue_family_properties().len(),
            "Queue family index out of bounds"
        );
        unsafe {
            (self.res.fun.get_physical_device_surface_support_khr)(
                phy.handle(),
                queue_family,
                self.handle(),
                &mut result,
            )?;
        }
        Ok(result.into())
    }

    pub fn capabilities(
        &self,
        phy: &PhysicalDevice,
    ) -> Result<SurfaceCapabilitiesKHR> {
        assert!(Arc::ptr_eq(&self.res.instance, &phy.instance));
        // Check phy support?
        let mut result = MaybeUninit::uninit();
        unsafe {
            (self.res.fun.get_physical_device_surface_capabilities_khr)(
                phy.handle(),
                self.handle(),
                &mut result,
            )?;
            Ok(result.assume_init())
        }
    }

    pub fn surface_formats(
        &self,
        phy: &PhysicalDevice,
    ) -> Result<Vec<SurfaceFormatKHR>> {
        assert!(Arc::ptr_eq(&self.res.instance, &phy.instance));
        let mut len = 0;
        let mut result = vec![];
        unsafe {
            (self.res.fun.get_physical_device_surface_formats_khr)(
                phy.handle(),
                self.handle(),
                &mut len,
                None,
            )?;
            result.reserve(len as usize);
            (self.res.fun.get_physical_device_surface_formats_khr)(
                phy.handle(),
                self.handle(),
                &mut len,
                ArrayMut::from_slice(result.spare_capacity_mut()),
            )?;
            result.set_len(len as usize);
        }
        Ok(result)
    }
}
