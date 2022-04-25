use std::sync::Arc;

use crate::device::Device;
use crate::types::*;

#[derive(Debug)]
pub struct Queue {
    handle: QueueRef<'static>,
    device: Arc<Device>,
}

impl Queue {
    pub(crate) fn new(handle: QueueRef<'static>, device: Arc<Device>) -> Self {
        Self { handle, device }
    }
    pub fn queue_ref(&self) -> QueueRef<'_> {
        self.handle
    }
}
