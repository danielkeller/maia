use crate::device::Device;
use crate::error::Result;
use crate::{queue, types::*};

#[derive(Debug)]
struct FenceRAII {
    handle: Handle<VkFence>,
    #[allow(dead_code)]
    device: Arc<Device>,
}

#[derive(Debug)]
pub struct Fence(FenceRAII);

#[derive(Debug)]
struct PendingFenceRAII {
    fence: FenceRAII,
    _resources: queue::PendingResources,
}

#[derive(Debug)]
pub struct PendingFence(Option<PendingFenceRAII>);

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
    pub(crate) fn to_pending(
        self,
        resources: queue::PendingResources,
    ) -> PendingFence {
        PendingFence(Some(PendingFenceRAII {
            fence: self.0,
            _resources: resources,
        }))
    }
}

impl PendingFence {
    pub fn borrow(&self) -> Ref<VkFence> {
        self.0.as_ref().unwrap().fence.handle.borrow()
    }
    pub fn wait(mut self) -> Result<Fence> {
        let mut inner = self.0.take().unwrap();
        inner.wait_impl()?;
        Ok(Fence(inner.fence))
    }
}
impl PendingFenceRAII {
    fn wait_impl(&mut self) -> Result<()> {
        unsafe {
            (self.fence.device.fun.wait_for_fences)(
                self.fence.device.borrow(),
                1,
                (&[self.fence.handle.borrow()]).into(),
                true.into(),
                u64::MAX,
            )?;
            (self.fence.device.fun.reset_fences)(
                self.fence.device.borrow(),
                1,
                (&[self.fence.handle.borrow_mut()]).into(),
            )?;
        }
        Ok(())
    }
}

/// Dropping a pending fence is incorrect. Doesn't panic, because that can
/// obscure other errors.
impl Drop for PendingFence {
    fn drop(&mut self) {
        if let Some(inner) = &mut self.0 {
            if let Err(err) = inner.wait_impl() {
                // No possible safe way to continue
                eprintln!("Bug: Dropped pending fence.");
                eprintln!("Couldn't wait for fence: {}", err);
                std::process::abort()
            }
            eprintln!("Bug: Dropped pending fence.");
        }
    }
}
