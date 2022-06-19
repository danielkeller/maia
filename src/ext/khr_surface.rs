use std::mem::MaybeUninit;

use super::load::SurfaceKHRFn;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::ffi::ArrayMut;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;
use crate::subobject::{Owner, Subobject};
use crate::types::*;

pub struct SurfaceLifetime {
    handle: Handle<VkSurfaceKHR>,
    fun: SurfaceKHRFn,
    instance: Arc<Instance>,
}

/// A
#[doc = crate::spec_link!("surface", "_wsi_surface")]
///
/// Create with [crate::window::create_surface()].
#[derive(Debug)]
pub struct SurfaceKHR {
    inner: Owner<SurfaceLifetime>,
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
    /// Create a surface from an existing handle.
    pub fn new(handle: Handle<VkSurfaceKHR>, instance: Arc<Instance>) -> Self {
        Self {
            inner: Owner::new(SurfaceLifetime {
                handle,
                fun: SurfaceKHRFn::new(&instance),
                instance,
            }),
        }
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkSurfaceKHR> {
        self.inner.handle.borrow()
    }
    /// Borrows the inner Vulkan handle.
    pub fn handle_mut(&mut self) -> Mut<VkSurfaceKHR> {
        self.inner.handle.borrow_mut()
    }
    /// Extend the lifetime of the surface until the returned object is dropped.
    pub fn resource(&self) -> Subobject<SurfaceLifetime> {
        Subobject::new(&self.inner)
    }

    /// Returns true if the surface supports 'phy' on 'queue_family'. Returns
    /// [Error::OutOfBounds] if 'queue_family' is out of bounds.
    #[doc = crate::man_link!(vkGetPhysicalDeviceSurfaceSupportKHR)]
    pub fn support(
        &self,
        phy: &PhysicalDevice,
        queue_family: u32,
    ) -> Result<bool> {
        let mut result = Bool::False;
        assert!(std::ptr::eq(&*self.inner.instance, phy.instance()));
        if (queue_family as usize) >= phy.queue_family_properties().len() {
            return Err(Error::OutOfBounds);
        }
        unsafe {
            (self.inner.fun.get_physical_device_surface_support_khr)(
                phy.handle(),
                queue_family,
                self.handle(),
                &mut result,
            )?;
        }
        Ok(result.into())
    }

    #[doc = crate::man_link!(vkGetPhysicalDeviceSurfaceCapabilitiesKHR)]
    pub fn capabilities(
        &self,
        phy: &PhysicalDevice,
    ) -> Result<SurfaceCapabilitiesKHR> {
        assert!(std::ptr::eq(&*self.inner.instance, phy.instance()));
        // Check phy support?
        let mut result = MaybeUninit::uninit();
        unsafe {
            (self.inner.fun.get_physical_device_surface_capabilities_khr)(
                phy.handle(),
                self.handle(),
                &mut result,
            )?;
            Ok(result.assume_init())
        }
    }

    #[doc = crate::man_link!(vkGetPhysicalDeviceSurfaceFormatsKHR)]
    pub fn surface_formats(
        &self,
        phy: &PhysicalDevice,
    ) -> Result<Vec<SurfaceFormatKHR>> {
        assert!(std::ptr::eq(&*self.inner.instance, phy.instance()));
        let mut len = 0;
        let mut result = vec![];
        unsafe {
            (self.inner.fun.get_physical_device_surface_formats_khr)(
                phy.handle(),
                self.handle(),
                &mut len,
                None,
            )?;
            result.reserve(len as usize);
            (self.inner.fun.get_physical_device_surface_formats_khr)(
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
