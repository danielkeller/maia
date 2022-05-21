use crate::{error::Result, types::*, vk::Device};

pub struct Semaphore {
    handle: Handle<VkSemaphore>,
    device: Arc<Device>,
}

impl Device {
    pub fn create_semaphore(self: &Arc<Self>) -> Result<Semaphore> {
        let mut handle = None;
        unsafe {
            (self.fun.create_semaphore)(
                self.borrow(),
                &Default::default(),
                None,
                &mut handle,
            )?;
        }
        Ok(Semaphore { handle: handle.unwrap(), device: self.clone() })
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_semaphore)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl Semaphore {
    pub fn borrow(&self) -> Ref<VkSemaphore> {
        self.handle.borrow()
    }
    pub fn borrow_mut(&mut self) -> Mut<VkSemaphore> {
        self.handle.borrow_mut()
    }
}
