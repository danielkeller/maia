use crate::cleanup_queue::Cleanup;
use crate::device::Device;
use crate::error::Result;
use crate::types::*;

#[derive(Debug)]
pub struct Fence {
    handle: Option<Handle<VkFence>>,
    device: Arc<Device>,
}

#[derive(Debug)]
#[must_use = "Dropping a pending fence leaks it."]
pub struct PendingFence {
    handle: Handle<VkFence>,
    device: Arc<Device>,
    resources: Cleanup,
}

impl Device {
    pub fn create_fence(self: &Arc<Self>) -> Result<Fence> {
        let mut handle = None;
        unsafe {
            (self.fun.create_fence)(
                self.handle(),
                &Default::default(),
                None,
                &mut handle,
            )?;
        }
        Ok(Fence { handle, device: self.clone() })
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        if let Some(handle) = &mut self.handle {
            unsafe {
                (self.device.fun.destroy_fence)(
                    self.device.handle(),
                    handle.borrow_mut(),
                    None,
                )
            }
        }
    }
}

impl Fence {
    pub fn handle_mut(&mut self) -> Mut<VkFence> {
        self.handle.as_mut().unwrap().borrow_mut()
    }
    pub(crate) fn to_pending(mut self, resources: Cleanup) -> PendingFence {
        PendingFence {
            handle: self.handle.take().unwrap(),
            device: self.device.clone(),
            resources,
        }
    }
}

impl PendingFence {
    pub fn handle(&self) -> Ref<VkFence> {
        self.handle.borrow()
    }
    pub fn wait(mut self) -> Result<Fence> {
        unsafe {
            (self.device.fun.wait_for_fences)(
                self.device.handle(),
                1,
                (&[self.handle.borrow()]).into(),
                true.into(),
                u64::MAX,
            )?;
        }
        self.resources.cleanup();
        unsafe {
            (self.device.fun.reset_fences)(
                self.device.handle(),
                1,
                // Safe because the the outer structure is owned here
                (&[self.handle.borrow_mut()]).into(),
            )?;
        }
        Ok(Fence { handle: Some(self.handle), device: self.device })
    }
}
