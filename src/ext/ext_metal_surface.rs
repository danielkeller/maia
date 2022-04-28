use crate::types::*;

use crate::instance::Instance;

use super::khr_surface::SurfaceKHR;
use super::load::MetalSurfaceFn;

pub struct EXTMetalSurface {
    pub(crate) fun: MetalSurfaceFn,
    instance: Arc<Instance>,
}

impl Instance {
    pub fn ext_metal_surface(self: &Arc<Self>) -> EXTMetalSurface {
        EXTMetalSurface {
            fun: MetalSurfaceFn::new(self.inst_ref()),
            instance: self.clone(),
        }
    }
}

impl EXTMetalSurface {
    pub unsafe fn create_metal_surface_ext(
        &self,
        info: &MetalSurfaceCreateInfoEXT,
    ) -> Result<Arc<SurfaceKHR>> {
        let mut handle = None;
        (self.fun.create_metal_surface_ext)(
            self.instance.inst_ref(),
            info,
            None,
            &mut handle,
        )?;
        Ok(SurfaceKHR::new(handle.unwrap(), self.instance.clone()))
    }
}
