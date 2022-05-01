use crate::enums::*;
use crate::instance::Instance;
use crate::types::*;
use std::mem::{transmute, MaybeUninit};

pub struct SurfaceKHRFn {
    pub destroy_surface_khr: unsafe extern "system" fn(
        InstanceRef<'_>,
        SurfaceKHRRef<'static>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub get_physical_device_surface_support_khr:
        unsafe extern "system" fn(
            PhysicalDeviceRef<'_>,
            u32,
            SurfaceKHRRef<'_>,
            &mut Bool,
        ) -> Result<()>,
    pub get_physical_device_surface_capabilities_khr:
        unsafe extern "system" fn(
            PhysicalDeviceRef<'_>,
            SurfaceKHRRef<'_>,
            &mut MaybeUninit<SurfaceCapabilitiesKHR>,
        ) -> Result<()>,
    pub get_physical_device_surface_formats_khr:
        unsafe extern "system" fn(
            PhysicalDeviceRef<'_>,
            SurfaceKHRRef<'_>,
            &mut u32,
            Option<&mut MaybeUninit<SurfaceFormatKHR>>,
        ) -> Result<()>,
}

impl SurfaceKHRFn {
    pub fn new(inst: &Instance) -> Self {
        unsafe {
            Self {
                destroy_surface_khr: transmute(
                    inst.get_proc_addr("vkDestroySurfaceKHR\0"),
                ),
                get_physical_device_surface_support_khr: transmute(
                    inst.get_proc_addr(
                        "vkGetPhysicalDeviceSurfaceSupportKHR\0",
                    ),
                ),
                get_physical_device_surface_capabilities_khr: transmute(
                    inst.get_proc_addr(
                        "vkGetPhysicalDeviceSurfaceCapabilitiesKHR\0",
                    ),
                ),
                get_physical_device_surface_formats_khr: transmute(
                    inst.get_proc_addr(
                        "vkGetPhysicalDeviceSurfaceFormatsKHR\0",
                    ),
                ),
            }
        }
    }
}

pub struct MetalSurfaceFn {
    pub create_metal_surface_ext: unsafe extern "system" fn(
        InstanceRef<'_>,
        &MetalSurfaceCreateInfoEXT,
        Option<&'_ AllocationCallbacks>,
        &mut Option<SurfaceKHRRef<'static>>,
    ) -> Result<()>,
}

impl MetalSurfaceFn {
    pub fn new(inst: &Instance) -> Self {
        unsafe {
            Self {
                create_metal_surface_ext: transmute(
                    inst.get_proc_addr("vkCreateMetalSurfaceEXT\0"),
                ),
            }
        }
    }
}
