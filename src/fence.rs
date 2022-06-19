use crate::cleanup_queue::Cleanup;
use crate::device::Device;
use crate::error::Result;
use crate::types::*;

/// A
#[doc = concat!(crate::spec_link!("fence", "synchronization-fences"), ".")]
/// When submitted to a [Queue](crate::vk::Queue), becomes a [PendingFence].
///
/// Create with [Device::create_fence()]
#[derive(Debug)]
pub struct Fence {
    handle: Option<Handle<VkFence>>,
    device: Arc<Device>,
}

/// A
#[doc = crate::spec_link!("fence", "synchronization-fences")]
/// with a signal operation pending.
#[derive(Debug)]
#[must_use = "Dropping a pending fence leaks it."]
pub struct PendingFence {
    handle: Handle<VkFence>,
    device: Arc<Device>,
    resources: Cleanup,
}

impl Device {
    #[doc = crate::man_link!(vkCreateFence)]
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
    /// Borrows the inner Vulkan handle.
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
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkFence> {
        self.handle.borrow()
    }
    /// Waits for the fence, decrements the reference count of any objects
    /// (including [CommandPools](crate::vk::CommandPool)) submitted to
    /// the queue, and resets the fence.
    #[doc = crate::man_link!(vkWaitForFences)]
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
