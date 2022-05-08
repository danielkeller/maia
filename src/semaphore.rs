use crate::{error::Result, types::*, vk::Device};

pub struct Semaphore {
    handle: SemaphoreMut<'static>,
    device: Arc<Device>,
}

impl Device {
    pub fn create_semaphore(self: &Arc<Self>) -> Result<Semaphore> {
        let mut handle = None;
        unsafe {
            (self.fun.create_semaphore)(
                self.dev_ref(),
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
                self.device.dev_ref(),
                self.handle.reborrow(),
                None,
            )
        }
    }
}

impl Semaphore {
    pub fn sem_mut(&mut self) -> SemaphoreMut<'_> {
        self.handle.reborrow()
    }
}
