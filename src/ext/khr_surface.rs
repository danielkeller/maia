use std::mem::MaybeUninit;

use super::load::SurfaceKHRFn;
use crate::enums::*;
use crate::error::Result;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;
use crate::types::*;

pub(crate) struct SurfaceResource {
    handle: SurfaceKHRRef<'static>,
    fun: SurfaceKHRFn,
    instance: Arc<Instance>,
}

#[derive(Debug)]
pub struct SurfaceKHR {
    pub(crate) res: Arc<SurfaceResource>,
}

impl std::fmt::Debug for SurfaceResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl Drop for SurfaceResource {
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
    ) -> Self {
        Self {
            res: Arc::new(SurfaceResource {
                handle,
                fun: SurfaceKHRFn::new(&instance),
                instance,
            }),
        }
    }

    pub fn surface_ref(&self) -> SurfaceKHRRef<'_> {
        self.res.handle
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
                phy.phy_ref(),
                queue_family,
                self.surface_ref(),
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
                phy.phy_ref(),
                self.surface_ref(),
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
                phy.phy_ref(),
                self.surface_ref(),
                &mut len,
                None,
            )?;
            result.reserve(len.try_into().unwrap());
            (self.res.fun.get_physical_device_surface_formats_khr)(
                phy.phy_ref(),
                self.surface_ref(),
                &mut len,
                result.spare_capacity_mut().first_mut(),
            )?;
            result.set_len(len.try_into().unwrap());
        }
        Ok(result)
    }
}
