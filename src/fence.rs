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
pub struct PendingFence(FenceRAII);

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
    pub fn fence_mut(&mut self) -> Mut<VkFence> {
        self.0.handle.borrow_mut()
    }
    pub(crate) fn to_pending(self) -> PendingFence {
        PendingFence(self.0)
    }
}

impl PendingFence {
    pub fn borrow(&self) -> Ref<VkFence> {
        self.0.handle.borrow()
    }
    pub fn wait(self) -> Result<Fence> {
        unsafe {
            (self.0.device.fun.wait_for_fences)(
                self.0.device.borrow(),
                1,
                &self.borrow(),
                true.into(),
                u64::MAX,
            )?;
        }
        Ok(Fence(self.0))
    }
}
