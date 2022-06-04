use std::fmt::Debug;

use crate::cleanup_queue::CleanupQueue;
use crate::command_buffer::CommandBuffer;
use crate::device::Device;
use crate::error::{Error, Result};
use crate::fence::{Fence, PendingFence};
use crate::ffi::Array;
use crate::semaphore::{Semaphore, SemaphoreSignaller};
use crate::types::*;
use crate::vk::PipelineStageFlags;

#[derive(Debug)]
pub struct Queue {
    handle: Handle<VkQueue>,
    device: Arc<Device>,
    pub(crate) resources: CleanupQueue,
}

impl Queue {
    pub(crate) fn new(handle: Handle<VkQueue>, device: Arc<Device>) -> Self {
        Self { handle, device, resources: CleanupQueue::new(100) }
    }
    pub fn borrow(&self) -> Ref<VkQueue> {
        self.handle.borrow()
    }
    pub fn borrow_mut(&mut self) -> Mut<VkQueue> {
        self.handle.borrow_mut()
    }
}

impl Drop for Queue {
    /// Waits for the queue to be idle before dropping resources
    fn drop(&mut self) {
        unsafe {
            if let Err(err) =
                (self.device.fun.queue_wait_idle)(self.handle.borrow_mut())
            {
                self.resources.leak();
                panic!("vkQueueWaitIdle failed: {}", err);
            }
        }
    }
}

pub struct SubmitInfo<'a> {
    pub wait: &'a mut [(&'a mut Semaphore, PipelineStageFlags)],
    pub commands: &'a mut [&'a mut CommandBuffer],
    pub signal: &'a mut [&'a mut Semaphore],
}

impl Queue {
    pub fn submit(
        &mut self,
        infos: &mut [SubmitInfo<'_>],
        mut fence: Fence,
    ) -> Result<PendingFence> {
        for info in infos.iter() {
            for (sem, _) in info.wait.iter() {
                if sem.signaller.is_none() {
                    return Err(Error::InvalidArgument);
                }
            }
            for sem in info.signal.iter() {
                if sem.signaller.is_some() {
                    return Err(Error::InvalidArgument);
                }
            }
        }

        let mut recordings = vec![];
        let handles = infos
            .iter_mut()
            .map(|i| {
                for c in i.commands.iter() {
                    recordings.push(
                        c.lock_resources().ok_or(Error::InvalidArgument)?,
                    );
                }
                Ok((
                    i.wait.iter().map(|w| w.0.borrow()).collect::<Vec<_>>(),
                    i.wait.iter().map(|w| w.1).collect::<Vec<_>>(),
                    i.commands
                        .iter_mut()
                        .map(|c| c.borrow_mut())
                        .collect::<Result<Vec<_>>>()?,
                    i.signal.iter().map(|s| s.borrow()).collect::<Vec<_>>(),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let vk_infos = handles
            .iter()
            .map(|(ws, wss, cs, ss)| VkSubmitInfo {
                wait_semaphores: ws.into(),
                wait_stage_masks: Array::from_slice(wss),
                command_buffers: cs.into(),
                signal_semaphores: ss.into(),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        unsafe {
            (self.device.fun.queue_submit)(
                self.borrow_mut(),
                vk_infos.len() as u32,
                Array::from_slice(&vk_infos),
                Some(fence.borrow_mut()),
            )?;
        }

        for info in infos {
            for (sem, _) in info.wait.iter_mut() {
                self.resources.push(sem.take_signaller());
                self.resources.push(sem.inner.clone());
            }
            for command in info.commands.iter() {
                self.resources.push(command.0.clone());
            }
            for sem in info.signal.iter_mut() {
                sem.signaller = Some(SemaphoreSignaller::Queue(
                    self.resources.new_cleanup(),
                ));
                self.resources.push(sem.inner.clone());
            }
        }
        self.resources.extend(recordings.into_iter());

        Ok(fence.to_pending(self.resources.new_cleanup()))
    }
}
