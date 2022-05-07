use crate::device::Device;
use crate::error::Result;
use crate::types::*;

#[derive(Debug)]
struct FenceRAII {
    handle: FenceMut<'static>,
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
                self.dev_ref(),
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
                self.device.dev_ref(),
                self.handle,
                None,
            )
        }
    }
}

impl Fence {
    pub fn fence_mut(&mut self) -> FenceMut<'_> {
        self.0.handle
    }
    pub(crate) fn to_pending(self) -> PendingFence {
        PendingFence(self.0)
    }
}

impl PendingFence {
    pub fn fence_ref(&self) -> PendingFenceRef<'_> {
        unsafe { self.0.handle.to_pending() }
    }
    pub fn wait(self) -> Result<Fence> {
        unsafe {
            (self.0.device.fun.wait_for_fences)(
                self.0.device.dev_ref(),
                1,
                &self.fence_ref(),
                true.into(),
                u64::MAX,
            )?;
        }
        Ok(Fence(self.0))
    }
}
