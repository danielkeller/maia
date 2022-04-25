// use crate::ffi::*;
use crate::load::load;
use crate::types::*;
use std::mem::transmute;

pub struct SurfaceKHRFn {
    pub destroy_surface_khr: unsafe extern "system" fn(
        InstanceRef<'_>,
        SurfaceKHRRef<'static>,
        Option<&'_ AllocationCallbacks>,
    ),
}

impl SurfaceKHRFn {
    pub fn new(inst: InstanceRef<'_>) -> Self {
        let inst = Some(inst);
        unsafe {
            Self {
                destroy_surface_khr: transmute(load(
                    inst,
                    "vkDestroySurfaceKHR\0",
                )),
            }
        }
    }
}

pub struct MetalSurfaceFn {
    pub create_metal_surface_ext: unsafe extern "system" fn(
        InstanceRef<'_>,
        &MetalSurfaceCreateInfoEXT<'_>,
        Option<&'_ AllocationCallbacks>,
        &mut Option<SurfaceKHRRef<'static>>,
    ) -> Result<()>,
}

impl MetalSurfaceFn {
    pub fn new(inst: InstanceRef<'_>) -> Self {
        let inst = Some(inst);
        unsafe {
            Self {
                create_metal_surface_ext: transmute(load(
                    inst,
                    "vkCreateMetalSurfaceEXT\0",
                )),
            }
        }
    }
}
