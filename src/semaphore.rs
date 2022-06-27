use crate::cleanup_queue::Cleanup;
use crate::device::Device;
use crate::error::Result;
use crate::image::Image;
use crate::types::*;

/// A
#[doc = crate::spec_link!("semaphore", "synchronization-semaphores")]
pub struct Semaphore {
    pub(crate) signaller: Option<SemaphoreSignaller>,
    pub(crate) inner: Arc<SemaphoreRAII>,
}

#[derive(Debug)]
pub(crate) enum SemaphoreSignaller {
    Swapchain(Arc<Image>),
    Queue(Cleanup),
}

pub(crate) struct SemaphoreRAII {
    handle: Handle<VkSemaphore>,
    device: Arc<Device>,
}

impl Semaphore {
    #[doc = crate::man_link!(vkCreateSemaphore)]
    pub fn new(device: &Arc<Device>) -> Result<Self> {
        let mut handle = None;
        unsafe {
            (device.fun.create_semaphore)(
                device.handle(),
                &Default::default(),
                None,
                &mut handle,
            )?;
        }
        Ok(Self {
            signaller: None,
            inner: Arc::new(SemaphoreRAII {
                handle: handle.unwrap(),
                device: device.clone(),
            }),
        })
    }
}

impl Drop for Semaphore {
    /// **Warning:** If a semaphore is passed to
    /// [`SwapchainKHR::acquire_next_image`](crate::vk::ext::SwapchainKHR::acquire_next_image())
    /// and then dropped without being waited on, the swapchain and semaphore
    /// will be leaked, since there is no way to know when it can be safely
    /// dropped other than waiting on it.
    fn drop(&mut self) {
        if let Some(SemaphoreSignaller::Swapchain(sc)) = self.signaller.take() {
            // Semaphore incorrectly dropped
            std::mem::forget(sc); // Leak the swapchain
            std::mem::forget(self.inner.clone()); // Leak the semaphore
            eprintln!(
                "Semaphore used with WSI and then freed without being waited on"
            );
        }
        // Dropping an unwaited semaphore is normally fine since for a
        // queue, the signal op is ordered before the fence signal.
    }
}

impl Drop for SemaphoreRAII {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_semaphore)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl Semaphore {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkSemaphore> {
        self.inner.handle.borrow()
    }
    /// Borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkSemaphore> {
        // Safe because the outer structure is mutably borrowed, and handle is
        // private.
        unsafe { self.inner.handle.borrow_mut_unchecked() }
    }

    /// Panics if there is no signaller
    pub(crate) fn take_signaller(&mut self) -> Arc<dyn Send + Sync> {
        match self.signaller.take().unwrap() {
            SemaphoreSignaller::Queue(cleanup) => Arc::new(cleanup.raii()),
            SemaphoreSignaller::Swapchain(image) => image,
        }
    }
}
