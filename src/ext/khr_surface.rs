use super::load::SurfaceKHRFn;
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
            fun: SurfaceKHRFn::new(instance.inst_ref()),
            instance,
        })
    }

    pub fn surface_ref(&self) -> SurfaceKHRRef<'_> {
        self.handle
    }

    pub fn physical_device_support(
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
}
