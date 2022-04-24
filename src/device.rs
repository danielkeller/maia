use crate::types::*;

impl Device {
    pub fn queue(&self, family_index: u32, queue_index: u32) -> Result<Queue> {
        let i = family_index as usize;
        if i > self.0.queues.len() || self.0.queues[i] <= queue_index {
            return Err(Error::INITIALIZATION_FAILED);
        }
        let mut handle = None;
        unsafe {
            (self.0.fun.get_device_queue)(
                self.as_ref(),
                family_index,
                queue_index,
                &mut handle,
            );
        }
        Ok(Queue::new(handle.unwrap(), self.0.clone()))
    }
}
