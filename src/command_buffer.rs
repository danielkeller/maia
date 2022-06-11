use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::sync::Weak;

use crate::device::Device;
use crate::enums::*;
use crate::error::{Error, ErrorAndSelf, Result, ResultAndSelf};
use crate::exclusive::Exclusive;
use crate::framebuffer::Framebuffer;
use crate::render_pass::RenderPass;
use crate::subobject::{Owner, Subobject};
use crate::types::*;

pub mod command;

pub struct CommandPool {
    recording: Option<Arc<RecordedCommands>>,
    res: Owner<CommandPoolLifetime>,
    scratch: Exclusive<bumpalo::Bump>,
}

#[must_use = "Leaks pool resources if not freed"]
#[derive(Debug)]
// Theoretically the arc could be on the outside, but this would encourage users
// to store copies of it, causing confusing errors when they try to record only
// to find it locked.
pub struct CommandBuffer(pub(crate) Arc<CommandBufferLifetime>);

pub struct CommandRecording<'a> {
    pool: &'a mut CommandPool,
    buffer: CommandBufferLifetime,
}

pub struct RenderPassRecording<'a, 'rec>(&'a mut CommandRecording<'rec>);

#[derive(Debug)]
pub(crate) struct CommandBufferLifetime {
    handle: Handle<VkCommandBuffer>,
    pool: Subobject<CommandPoolLifetime>,
    /// For buffers in the executable state, it will give an Arc. Otherwise the
    /// buffer is in the initial state.
    recording: Weak<RecordedCommands>,
}

#[derive(Debug)]
struct CommandPoolLifetime {
    handle: Handle<VkCommandPool>,
    resources: Vec<Arc<dyn Send + Sync + Debug>>,
    device: Arc<Device>,
}

#[derive(Debug)]
struct RecordedCommands(Subobject<CommandPoolLifetime>);

impl std::fmt::Debug for CommandPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.res.handle.fmt(f)
    }
}

impl Device {
    // Don't take a create info, the different types of command pools need
    // different interfaces and thus different constructors
    pub fn create_command_pool(
        self: &Arc<Self>,
        queue_family_index: u32,
    ) -> Result<CommandPool> {
        let i = queue_family_index as usize;
        if i > self.queues.len() || self.queues[i] < 1 {
            return Err(Error::InvalidArgument);
        }
        let mut handle = None;
        unsafe {
            (self.fun.create_command_pool)(
                self.borrow(),
                &CommandPoolCreateInfo {
                    queue_family_index,
                    ..Default::default()
                },
                None,
                &mut handle,
            )?;
        }
        let handle = handle.unwrap();

        let res = Owner::new(CommandPoolLifetime {
            handle,
            resources: vec![],
            device: self.clone(),
        });
        let _res = Subobject::new(&res);
        Ok(CommandPool {
            res,
            recording: Some(Arc::new(RecordedCommands(_res))),
            scratch: Exclusive::new(bumpalo::Bump::new()),
        })
    }
}

impl Drop for CommandPoolLifetime {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_command_pool)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl CommandPool {
    pub fn borrow_mut(&mut self) -> Mut<VkCommandPool> {
        self.res.handle.borrow_mut()
    }

    /// Return SynchronizationError if any command buffers are pending.
    pub fn reset(&mut self, flags: CommandPoolResetFlags) -> Result<()> {
        match Arc::try_unwrap(self.recording.take().unwrap()) {
            // Buffer in pending state
            Err(arc) => {
                self.recording = Some(arc);
                Err(Error::SynchronizationError)
            }
            Ok(_) => {
                let res = &mut *self.res;
                unsafe {
                    (res.device.fun.reset_command_pool)(
                        res.device.borrow(),
                        res.handle.borrow_mut(),
                        flags,
                    )?;
                }
                self.recording =
                    Some(Arc::new(RecordedCommands(Subobject::new(&self.res))));
                self.res.resources.clear();
                Ok(())
            }
        }
    }

    pub fn allocate(&mut self) -> Result<CommandBuffer> {
        let mut handle = MaybeUninit::uninit();
        let res = &mut *self.res;
        let handle = unsafe {
            (res.device.fun.allocate_command_buffers)(
                res.device.borrow(),
                &CommandBufferAllocateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    pool: res.handle.borrow_mut(),
                    level: CommandBufferLevel::PRIMARY,
                    count: 1,
                },
                std::array::from_mut(&mut handle).into(),
            )?;
            handle.assume_init()
        };
        Ok(CommandBuffer(Arc::new(CommandBufferLifetime {
            handle,
            pool: Subobject::new(&self.res),
            recording: Weak::new(),
        })))
    }

    pub fn free(&mut self, mut buffer: CommandBuffer) -> Result<()> {
        if !Owner::ptr_eq(&self.res, &buffer.0.pool) {
            return Err(Error::InvalidArgument);
        }

        let res = &mut *self.res;
        unsafe {
            (res.device.fun.free_command_buffers)(
                res.device.borrow(),
                res.handle.borrow_mut(),
                1,
                &buffer.borrow_mut()?,
            );
        }

        Ok(())
    }

    /// Returns InvalidArgument if the buffer does not belong to this pool or is
    /// not in the initial state.
    pub fn begin<'a>(
        &'a mut self,
        buffer: CommandBuffer,
    ) -> ResultAndSelf<CommandRecording<'a>, CommandBuffer> {
        if !Owner::ptr_eq(&self.res, &buffer.0.pool)
            // In executable state
            || buffer.lock_resources().is_some()
        {
            return Err(ErrorAndSelf(Error::InvalidArgument, buffer));
        }
        // In pending state
        let mut inner = Arc::try_unwrap(buffer.0).map_err(|arc| {
            ErrorAndSelf(Error::SynchronizationError, CommandBuffer(arc))
        })?;
        unsafe {
            if let Err(err) = (self.res.device.fun.begin_command_buffer)(
                inner.handle.borrow_mut(),
                &Default::default(),
            ) {
                return Err(ErrorAndSelf(
                    err.into(),
                    CommandBuffer(Arc::new(inner)),
                ));
            };
        }
        Ok(CommandRecording { pool: self, buffer: inner })
    }
}

impl CommandBuffer {
    pub fn borrow_mut(&mut self) -> Result<Mut<VkCommandBuffer>> {
        match Arc::get_mut(&mut self.0) {
            Some(inner) => Ok(inner.handle.borrow_mut()),
            None => Err(Error::SynchronizationError),
        }
    }
    /// Prevent the command pool from being cleared or destroyed until the value
    /// is dropped.
    pub(crate) fn lock_resources(
        &self,
    ) -> Option<Arc<impl Send + Sync + Debug>> {
        self.0.recording.upgrade()
    }
}

impl<'a> CommandRecording<'a> {
    fn add_resource(&mut self, value: Arc<dyn Send + Sync + Debug>) {
        self.pool.res.resources.push(value);
    }
    /// A failed call to vkEndCommandBuffer leaves the buffer in the invalid
    /// state, so it is dropped in that case.
    pub fn end(mut self) -> Result<CommandBuffer> {
        unsafe {
            (self.pool.res.device.fun.end_command_buffer)(
                self.buffer.handle.borrow_mut(),
            )?;
        }
        self.buffer.recording =
            Arc::downgrade(self.pool.recording.as_ref().unwrap());
        Ok(CommandBuffer(Arc::new(self.buffer)))
    }
}

impl<'rec> CommandRecording<'rec> {
    #[must_use = "Record render pass commands on this object"]
    pub fn begin_render_pass(
        &mut self,
        render_pass: &Arc<RenderPass>,
        framebuffer: &Arc<Framebuffer>,
        render_area: Rect2D,
        clear_values: &[ClearValue],
    ) -> RenderPassRecording<'_, 'rec> {
        self.add_resource(render_pass.clone());
        self.add_resource(framebuffer.clone());
        let info = RenderPassBeginInfo {
            stype: Default::default(),
            next: Default::default(),
            render_pass: render_pass.borrow(),
            framebuffer: framebuffer.borrow(),
            render_area,
            clear_values: clear_values.into(),
        };
        unsafe {
            (self.pool.res.device.fun.cmd_begin_render_pass)(
                self.buffer.handle.borrow_mut(),
                &info,
                SubpassContents::INLINE,
            );
        }
        RenderPassRecording(self)
    }
}

impl<'a, 'rec> RenderPassRecording<'a, 'rec> {
    pub fn end(self) {}
}

impl<'a, 'rec> Drop for RenderPassRecording<'a, 'rec> {
    fn drop(&mut self) {
        unsafe {
            (self.0.pool.res.device.fun.cmd_end_render_pass)(
                self.0.buffer.handle.borrow_mut(),
            )
        }
    }
}
