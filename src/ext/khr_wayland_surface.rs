use std::ffi::c_void;
use std::intrinsics::transmute;
use std::ptr::NonNull;

use crate::enums::*;
use crate::types::*;

use crate::error::Result;
use crate::instance::Instance;
use crate::physical_device::PhysicalDevice;

use super::khr_surface::SurfaceKHR;

/// An KHR_wayland_surface extension object. Create with
/// [Instance::khr_wayland_surface()]
pub struct KHRWaylandSurface {
    fun: WaylandSurfaceFn,
    instance: Arc<Instance>,
}

impl Instance {
    /// Creates an [KHRWaylandSurface] extension object. Panics if the extension
    /// functions can't be loaded.
    pub fn khr_wayland_surface(self: &Arc<Self>) -> KHRWaylandSurface {
        KHRWaylandSurface {
            fun: WaylandSurfaceFn::new(&self),
            instance: self.clone(),
        }
    }
}

impl KHRWaylandSurface {
    #[doc = crate::man_link!(vkGetPhysicalDeviceWaylandPresentationSupportKHR)]
    pub unsafe fn presentation_support(
        &self,
        phy: &PhysicalDevice,
        queue_family_index: u32,
        display: NonNull<c_void>,
    ) -> bool {
        (self.fun.get_physical_device_wayland_presentation_support_khr)(
            phy.handle(),
            queue_family_index,
            display,
        )
        .into()
    }
    #[doc = crate::man_link!(vkCreateWaylandSurfaceKHR)]
    pub unsafe fn create_wayland_surface_ext(
        &self,
        info: &WaylandSurfaceCreateInfoKHR,
    ) -> Result<SurfaceKHR> {
        let mut handle = None;
        (self.fun.create_wayland_surface_khr)(
            self.instance.handle(),
            info,
            None,
            &mut handle,
        )?;
        Ok(SurfaceKHR::new(handle.unwrap(), self.instance.clone()))
    }
}

pub struct WaylandSurfaceFn {
    pub get_physical_device_wayland_presentation_support_khr:
        unsafe extern "system" fn(
            Ref<VkPhysicalDevice>,
            u32,
            NonNull<c_void>,
        ) -> Bool,
    pub create_wayland_surface_khr: unsafe extern "system" fn(
        Ref<VkInstance>,
        &WaylandSurfaceCreateInfoKHR,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkSurfaceKHR>>,
    ) -> VkResult,
}

impl WaylandSurfaceFn {
    pub fn new(inst: &Instance) -> Self {
        Self {
            get_physical_device_wayland_presentation_support_khr: unsafe {
                transmute(inst.get_proc_addr(
                    "vkGetPhysicalDeviceWaylandPresentationSupportKHR\0",
                ))
            },
            create_wayland_surface_khr: unsafe {
                transmute(inst.get_proc_addr("vkCreateWaylandSurfaceKHR\0"))
            },
        }
    }
}
