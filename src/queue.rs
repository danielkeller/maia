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
    scratch: bumpalo::Bump,
}

// Safety: 'scratch' is not accessed by any shared-ref methods
unsafe impl Sync for Queue {}

impl Queue {
    pub(crate) fn new(handle: Handle<VkQueue>, device: Arc<Device>) -> Self {
        Self {
            handle,
            device,
            resources: CleanupQueue::new(100),
            scratch: bumpalo::Bump::new(),
        }
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

#[derive(Default)]
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

        self.scratch.reset();

        // This needs to stay in a Vec because its destructor is important
        let mut recordings = bumpalo::vec![in &self.scratch];
        let mut vk_infos = bumpalo::vec![in &self.scratch];
        for info in infos.iter_mut() {
            let mut commands = bumpalo::vec![in &self.scratch];
            for c in info.commands.iter_mut() {
                recordings
                    .push(c.lock_resources().ok_or(Error::InvalidArgument)?);
                commands.push(c.borrow_mut()?);
            }
            let wait_semaphores = self.scratch.alloc_slice_fill_iter(
                info.wait.iter().map(|(sem, _)| sem.borrow()),
            );
            let wait_stage_masks = self.scratch.alloc_slice_fill_iter(
                info.wait.iter().map(|(_, mask)| *mask), //
            );
            let signal_semaphores = self.scratch.alloc_slice_fill_iter(
                info.signal.iter().map(|sem| sem.borrow()),
            );
            vk_infos.push(VkSubmitInfo {
                wait_semaphores: wait_semaphores.into(),
                wait_stage_masks: Array::from_slice(wait_stage_masks),
                command_buffers: commands.into_bump_slice().into(),
                signal_semaphores: signal_semaphores.into(),
                ..Default::default()
            });
        }

        unsafe {
            (self.device.fun.queue_submit)(
                self.handle.borrow_mut(),
                vk_infos.len() as u32,
                Array::from_slice(&vk_infos),
                Some(fence.borrow_mut()),
            )?;
        }
        drop(vk_infos);

        // Everything fallible is done, mark resources as in use
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
