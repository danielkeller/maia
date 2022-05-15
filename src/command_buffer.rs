use std::fmt::Debug;
use std::mem::MaybeUninit;

use crate::device::Device;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::ffi::ArrayMut;
use crate::subobject::{Owner, Subobject, WeakSubobject};
use crate::types::*;

pub mod command;

pub struct CommandPool {
    handle: Handle<VkCommandPool>,
    recorded: Option<Owner<RecordedCommands>>,
    res: Arc<CommandPoolLifetime>,
}

#[must_use = "Leaks pool resources if not freed"]
pub struct CommandBuffer {
    handle: Handle<VkCommandBuffer>,
    pool: Arc<CommandPoolLifetime>,
    /// For buffers in the initial state, upgrading this will give None. For
    /// buffers in the executable state, it will give an Arc.
    recording: WeakSubobject<RecordedCommands>,
}

pub struct CommandRecording<'a> {
    pool: &'a mut CommandPool,
    buffer: CommandBuffer,
}

#[derive(Debug)]
struct CommandPoolLifetime {
    // Safety: Used only in Drop
    _handle: Handle<VkCommandPool>,
    device: Arc<Device>,
}

#[derive(Debug)]
pub(crate) struct RecordedCommands {
    resources: Vec<Arc<dyn Send + Sync + Debug>>,
    _res: Arc<CommandPoolLifetime>,
}

impl std::fmt::Debug for CommandPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl std::fmt::Debug for CommandBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandBuffer")
            .field("handle", &self.handle)
            .field("pool", &self.pool._handle)
            .finish()
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

        let res = Arc::new(CommandPoolLifetime {
            _handle: unsafe { handle.clone() },
            device: self.clone(),
        });
        let recorded = Some(Owner::new(RecordedCommands {
            _res: res.clone(),
            resources: vec![],
        }));
        Ok(CommandPool { handle, res, recorded })
    }
}

impl Drop for CommandPoolLifetime {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_command_pool)(
                self.device.borrow(),
                self._handle.borrow_mut(),
                None,
            )
        }
    }
}

impl CommandPool {
    pub fn borrow_mut(&mut self) -> Mut<VkCommandPool> {
        self.handle.borrow_mut()
    }

    /// Return SynchronizationError if any command buffers are pending.
    pub fn reset(&mut self, flags: CommandPoolResetFlags) -> Result<()> {
        let recorded = self.recorded.take().unwrap();
        // Try to lock the Arc, disassociating any Weak pointers from executable
        // command buffers.
        match Owner::try_unwrap(recorded) {
            Err(owner) => {
                self.recorded = Some(owner);
                Err(Error::SynchronizationError)
            }
            Ok(_inner) => {
                unsafe {
                    (self.res.device.fun.reset_command_pool)(
                        self.res.device.borrow(),
                        self.handle.borrow_mut(),
                        flags,
                    )?;
                }
                let owner = Owner::new(RecordedCommands {
                    _res: self.res.clone(),
                    resources: vec![],
                });
                self.recorded = Some(owner);
                Ok(())
            }
        }
    }

    pub fn allocate(&mut self) -> Result<CommandBuffer> {
        let mut handle = MaybeUninit::uninit();
        let handle = unsafe {
            (self.res.device.fun.allocate_command_buffers)(
                self.res.device.borrow(),
                &CommandBufferAllocateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    pool: self.handle.borrow_mut(),
                    level: CommandBufferLevel::PRIMARY,
                    count: 1,
                },
                std::array::from_mut(&mut handle).into(),
            )?;
            handle.assume_init()
        };
        Ok(CommandBuffer {
            handle,
            pool: self.res.clone(),
            recording: WeakSubobject::new(),
        })
    }

    pub fn free(&mut self, mut buffer: CommandBuffer) -> Result<()> {
        if !Arc::ptr_eq(&self.res, &buffer.pool) {
            return Err(Error::InvalidArgument);
        }

        unsafe {
            (self.res.device.fun.free_command_buffers)(
                self.res.device.borrow(),
                self.handle.borrow_mut(),
                1,
                &buffer.borrow_mut(),
            );
        }

        Ok(())
    }

    /// Return InvalidArgument if the buffer does not belong to this pool or is
    /// not in the initial state.
    pub fn begin<'a>(
        &'a mut self,
        mut buffer: CommandBuffer,
    ) -> Result<CommandRecording<'a>> {
        if !Arc::ptr_eq(&self.res, &buffer.pool)
            || buffer.recording.upgrade().is_some()
        {
            return Err(Error::InvalidArgument);
        }
        unsafe {
            (self.res.device.fun.begin_command_buffer)(
                buffer.borrow_mut(),
                &Default::default(),
            )?;
        }
        Ok(CommandRecording { pool: self, buffer })
    }
}

impl CommandBuffer {
    pub fn borrow_mut(&mut self) -> Mut<VkCommandBuffer> {
        self.handle.borrow_mut()
    }
    /// Prevent the command pool from being cleared, or any bound objects
    /// from being freed, until the Arc is dropped.
    pub(crate) fn lock_resources(&self) -> Option<Subobject<RecordedCommands>> {
        self.recording.upgrade()
    }
}

impl<'a> CommandRecording<'a> {
    pub(crate) fn add_resource(&mut self, value: Arc<dyn Send + Sync + Debug>) {
        self.pool.recorded.as_mut().unwrap().resources.push(value);
    }
    pub fn end(mut self) -> Result<CommandBuffer> {
        unsafe {
            (self.pool.res.device.fun.end_command_buffer)(
                self.buffer.borrow_mut(),
            )?;
        }
        self.buffer.recording =
            Owner::downgrade(self.pool.recorded.as_ref().unwrap());
        Ok(self.buffer)
    }
}
