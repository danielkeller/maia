use super::load::SurfaceKHRFn;
use crate::instance::Instance;
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
    pub(crate) fn new(
        handle: SurfaceKHRRef<'static>,
        instance: Arc<Instance>,
    ) -> Arc<Self> {
        Arc::new(SurfaceKHR {
            handle,
            fun: SurfaceKHRFn::new(instance.inst_ref()),
            instance,
        })
    }
}
