use std::ffi::c_void;
use std::intrinsics::transmute;
use std::ptr::NonNull;

use crate::enums::*;
use crate::types::*;

use crate::error::Result;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;

use super::khr_surface::SurfaceKHR;

/// An KHR_xlib_surface extension object.
pub struct KHRXlibSurface {
    fun: XlibSurfaceFn,
    instance: Arc<Instance>,
}

impl KHRXlibSurface {
    /// Creates an [KHRXlibSurface] extension object. Panics if the extension
    /// functions can't be loaded.
    pub fn new(instance: &Arc<Instance>) -> Self {
        Self {
            fun: XlibSurfaceFn::new(&instance),
            instance: instance.clone(),
        }
    }

    #[doc = crate::man_link!(vkGetPhysicalDeviceXlibPresentationSupportKHR)]
    pub unsafe fn presentation_support(
        &self,
        phy: &PhysicalDevice,
        queue_family_index: u32,
        display: NonNull<c_void>,
        visual_id: usize,
    ) -> bool {
        (self.fun.get_physical_device_xlib_presentation_support_khr)(
            phy.handle(),
            queue_family_index,
            display,
            visual_id,
        )
        .into()
    }
    #[doc = crate::man_link!(vkCreateXlibSurfaceKHR)]
    pub unsafe fn create_xlib_surface_ext(
        &self,
        info: &XlibSurfaceCreateInfoKHR,
    ) -> Result<SurfaceKHR> {
        let mut handle = None;
        (self.fun.create_xlib_surface_khr)(
            self.instance.handle(),
            info,
            None,
            &mut handle,
        )?;
        Ok(SurfaceKHR::new(handle.unwrap(), self.instance.clone()))
    }
}

pub struct XlibSurfaceFn {
    pub get_physical_device_xlib_presentation_support_khr:
        unsafe extern "system" fn(
            Ref<VkPhysicalDevice>,
            u32,
            NonNull<c_void>,
            usize,
        ) -> Bool,
    pub create_xlib_surface_khr: unsafe extern "system" fn(
        Ref<VkInstance>,
        &XlibSurfaceCreateInfoKHR,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkSurfaceKHR>>,
    ) -> VkResult,
}

impl XlibSurfaceFn {
    pub fn new(inst: &Instance) -> Self {
        Self {
            get_physical_device_xlib_presentation_support_khr: unsafe {
                transmute(inst.get_proc_addr(
                    "vkGetPhysicalDeviceXlibPresentationSupportKHR\0",
                ))
            },
            create_xlib_surface_khr: unsafe {
                transmute(inst.get_proc_addr("vkCreateXlibSurfaceKHR\0"))
            },
        }
    }
}
