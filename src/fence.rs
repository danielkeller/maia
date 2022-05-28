use crate::cleanup_queue::Cleanup;
use crate::device::Device;
use crate::error::Result;
use crate::types::*;

#[derive(Debug)]
struct FenceRAII {
    handle: Handle<VkFence>,
    #[allow(dead_code)]
    device: Arc<Device>,
}

#[derive(Debug)]
pub struct Fence(FenceRAII);

#[derive(Debug)]
#[must_use = "Dropping a pending fence does not wait on it."]
pub struct PendingFence {
    fence: FenceRAII,
    resources: Cleanup,
}

impl Device {
    pub fn create_fence(self: &Arc<Self>) -> Result<Fence> {
        let mut handle = None;
        unsafe {
            (self.fun.create_fence)(
                self.borrow(),
                &Default::default(),
                None,
                &mut handle,
            )?;
        }
        Ok(Fence(FenceRAII { handle: handle.unwrap(), device: self.clone() }))
    }
}

impl Drop for FenceRAII {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_fence)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl Fence {
    pub fn borrow_mut(&mut self) -> Mut<VkFence> {
        self.0.handle.borrow_mut()
    }
    pub(crate) fn to_pending(self, resources: Cleanup) -> PendingFence {
        PendingFence { fence: self.0, resources }
    }
}

impl PendingFence {
    pub fn borrow(&self) -> Ref<VkFence> {
        self.fence.handle.borrow()
    }
    pub fn wait(mut self) -> Result<Fence> {
        unsafe {
            (self.fence.device.fun.wait_for_fences)(
                self.fence.device.borrow(),
                1,
                (&[self.fence.handle.borrow()]).into(),
                true.into(),
                u64::MAX,
            )?;
        }
        self.resources.cleanup();
        unsafe {
            (self.fence.device.fun.reset_fences)(
                self.fence.device.borrow(),
                1,
                (&[self.fence.handle.borrow_mut()]).into(),
            )?;
        }
        Ok(Fence(self.fence))
    }
}
