use std::mem::MaybeUninit;

use super::load::SurfaceKHRFn;
use crate::enums::*;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;
use crate::types::*;

pub struct SurfaceKHR {
    handle: SurfaceKHRRef<'static>,
    fun: SurfaceKHRFn,
    instance: Arc<Instance>,
}

impl std::fmt::Debug for SurfaceKHR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl Drop for SurfaceKHR {
    fn drop(&mut self) {
        unsafe {
            (self.fun.destroy_surface_khr)(
                self.instance.inst_ref(),
                self.handle,
                None,
            )
        }
    }
}

impl SurfaceKHR {
    // Does this need to be an arc?
    pub(crate) fn new(
        handle: SurfaceKHRRef<'static>,
        instance: Arc<Instance>,
    ) -> Arc<Self> {
        Arc::new(SurfaceKHR {
            handle,
            fun: SurfaceKHRFn::new(&instance),
            instance,
        })
    }

    pub fn surface_ref(&self) -> SurfaceKHRRef<'_> {
        self.handle
    }

    pub fn support(
        &self,
        phy: &PhysicalDevice,
        queue_family: u32,
    ) -> Result<bool> {
        let mut result = Bool::False;
        assert!(Arc::ptr_eq(&self.instance, &phy.instance));
        assert!(
            (queue_family as usize) < phy.queue_family_properties().len(),
            "Queue family index out of bounds"
        );
        unsafe {
            (self.fun.get_physical_device_surface_support_khr)(
                phy.as_ref(),
                queue_family,
                self.handle,
                &mut result,
            )?;
        }
        Ok(result.into())
    }

    pub fn capabilities(
        &self,
        phy: &PhysicalDevice,
    ) -> Result<SurfaceCapabilitiesKHR> {
        assert!(Arc::ptr_eq(&self.instance, &phy.instance));
        // Check phy support?
        let mut result = MaybeUninit::uninit();
        unsafe {
            (self.fun.get_physical_device_surface_capabilities_khr)(
                phy.as_ref(),
                self.handle,
                &mut result,
            )?;
            Ok(result.assume_init())
        }
    }

    pub fn surface_formats(
        &self,
        phy: &PhysicalDevice,
    ) -> Result<Vec<SurfaceFormatKHR>> {
        assert!(Arc::ptr_eq(&self.instance, &phy.instance));
        let mut len = 0;
        let mut result = vec![];
        unsafe {
            (self.fun.get_physical_device_surface_formats_khr)(
                phy.as_ref(),
                self.handle,
                &mut len,
                None,
            )?;
            result.reserve(len.try_into().unwrap());
            (self.fun.get_physical_device_surface_formats_khr)(
                phy.as_ref(),
                self.handle,
                &mut len,
                result.spare_capacity_mut().first_mut(),
            )?;
            result.set_len(len.try_into().unwrap());
        }
        Ok(result)
    }
}
