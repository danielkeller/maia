use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::sync::Weak;

use crate::descriptor_set::DescriptorSetLayout;
use crate::device::Device;
use crate::enums::*;
use crate::error::{Error, ErrorAndSelf, Result, ResultAndSelf};
use crate::exclusive::Exclusive;
use crate::framebuffer::Framebuffer;
use crate::pipeline::Pipeline;
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
pub struct CommandBuffer(Arc<CommandBufferLifetime>);

#[must_use = "Leaks pool resources if not freed"]
#[derive(Debug)]
pub struct SecondaryCommandBuffer(Arc<CommandBufferLifetime>, u32);

struct Bindings<'a> {
    layout: bumpalo::collections::Vec<'a, Arc<DescriptorSetLayout>>,
    inited: bumpalo::collections::Vec<'a, bool>,
    pipeline: Option<Arc<Pipeline>>,
}

pub struct CommandRecording<'a> {
    pool: &'a mut Owner<CommandPoolLifetime>,
    recording: &'a Arc<RecordedCommands>,
    scratch: &'a bumpalo::Bump,
    graphics: Bindings<'a>,
    compute: Bindings<'a>,
    buffer: CommandBufferLifetime,
}

#[must_use = "Record render pass commands on this object"]
pub struct RenderPassRecording<'a> {
    rec: CommandRecording<'a>,
    num_subpasses: u32,
    subpass: u32,
}

#[must_use = "Record secondary command buffers on this object"]
pub struct ExternalRenderPassRecording<'a> {
    rec: CommandRecording<'a>,
    num_subpasses: u32,
    subpass: u32,
}

pub struct SecondaryCommandRecording<'a> {
    rec: CommandRecording<'a>,
    subpass: u32,
}

#[derive(Debug)]
struct CommandBufferLifetime {
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
        if !self.has_queue(queue_family_index, 1) {
            return Err(Error::OutOfBounds);
        }
        let mut handle = None;
        unsafe {
            (self.fun.create_command_pool)(
                self.handle(),
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
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl CommandPool {
    pub fn handle_mut(&mut self) -> Mut<VkCommandPool> {
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
                        res.device.handle(),
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
        Ok(CommandBuffer(self.allocate_impl(CommandBufferLevel::PRIMARY)?))
    }

    pub fn allocate_secondary(&mut self) -> Result<SecondaryCommandBuffer> {
        Ok(SecondaryCommandBuffer(
            self.allocate_impl(CommandBufferLevel::SECONDARY)?,
            0,
        ))
    }

    fn allocate_impl(
        &mut self,
        level: CommandBufferLevel,
    ) -> Result<Arc<CommandBufferLifetime>> {
        let mut handle = MaybeUninit::uninit();
        let res = &mut *self.res;
        let handle = unsafe {
            (res.device.fun.allocate_command_buffers)(
                res.device.handle(),
                &CommandBufferAllocateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    pool: res.handle.borrow_mut(),
                    level,
                    count: 1,
                },
                std::array::from_mut(&mut handle).into(),
            )?;
            handle.assume_init()
        };
        Ok(Arc::new(CommandBufferLifetime {
            handle,
            pool: Subobject::new(&self.res),
            recording: Weak::new(),
        }))
    }

    pub fn free(&mut self, mut buffer: CommandBuffer) -> Result<()> {
        if !Owner::ptr_eq(&self.res, &buffer.0.pool) {
            return Err(Error::InvalidArgument);
        }
        Ok(self.free_impl(buffer.handle_mut()?))
    }

    pub fn free_secondary(
        &mut self,
        mut buffer: SecondaryCommandBuffer,
    ) -> Result<()> {
        if !Owner::ptr_eq(&self.res, &buffer.0.pool) {
            return Err(Error::InvalidArgument);
        }
        Ok(self.free_impl(buffer.handle_mut()?))
    }

    fn free_impl(&mut self, buffer: Mut<VkCommandBuffer>) {
        let res = &mut *self.res;
        unsafe {
            (res.device.fun.free_command_buffers)(
                res.device.handle(),
                res.handle.borrow_mut(),
                1,
                &buffer,
            );
        }
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
        let scratch = self.scratch.get_mut();
        scratch.reset();
        Ok(CommandRecording {
            pool: &mut self.res,
            recording: self.recording.as_ref().unwrap(),
            graphics: Bindings::new(scratch),
            compute: Bindings::new(scratch),
            scratch,
            buffer: inner,
        })
    }

    /// Returns InvalidArgument if the buffer does not belong to this pool or is
    /// not in the initial state.
    pub fn begin_secondary<'a>(
        &'a mut self,
        buffer: SecondaryCommandBuffer,
        render_pass: &RenderPass,
        subpass: u32,
    ) -> ResultAndSelf<SecondaryCommandRecording<'a>, SecondaryCommandBuffer>
    {
        if subpass >= render_pass.num_subpasses() {
            return Err(ErrorAndSelf(Error::InvalidArgument, buffer));
        }
        if !Owner::ptr_eq(&self.res, &buffer.0.pool)
            // In executable state
            || buffer.lock_resources().is_some()
        {
            return Err(ErrorAndSelf(Error::InvalidArgument, buffer));
        }
        // In pending state
        let mut inner = Arc::try_unwrap(buffer.0).map_err(|arc| {
            ErrorAndSelf(
                Error::SynchronizationError,
                SecondaryCommandBuffer(arc, 0),
            )
        })?;
        unsafe {
            if let Err(err) = (self.res.device.fun.begin_command_buffer)(
                inner.handle.borrow_mut(),
                &CommandBufferBeginInfo {
                    flags: CommandBufferUsageFlags::RENDER_PASS_CONTINUE,
                    inheritance_info: Some(&CommandBufferInheritanceInfo {
                        stype: Default::default(),
                        next: Default::default(),
                        render_pass: render_pass.handle(),
                        subpass,
                        framebuffer: Default::default(),
                        occlusion_query_enable: Default::default(),
                        query_flags: Default::default(),
                        pipeline_statistics: Default::default(),
                    }),
                    ..Default::default()
                },
            ) {
                return Err(ErrorAndSelf(
                    err.into(),
                    SecondaryCommandBuffer(Arc::new(inner), 0),
                ));
            };
        }
        let scratch = self.scratch.get_mut();
        scratch.reset();
        Ok(SecondaryCommandRecording {
            rec: CommandRecording {
                pool: &mut self.res,
                recording: self.recording.as_ref().unwrap(),
                graphics: Bindings::new(scratch),
                compute: Bindings::new(scratch),
                scratch,
                buffer: inner,
            },
            subpass,
        })
    }
}

impl CommandBuffer {
    pub fn handle_mut(&mut self) -> Result<Mut<VkCommandBuffer>> {
        match Arc::get_mut(&mut self.0) {
            Some(inner) => Ok(inner.handle.borrow_mut()),
            None => Err(Error::SynchronizationError),
        }
    }
    /// Prevent the command buffer from being freed or submitted to a queue
    /// until the the value is dropped
    pub fn lock_self(&self) -> Arc<impl Send + Sync + Debug> {
        self.0.clone()
    }
    /// Prevent the command pool from being cleared or destroyed until the value
    /// is dropped.
    pub fn lock_resources(&self) -> Option<Arc<impl Send + Sync + Debug>> {
        self.0.recording.upgrade()
    }
}

impl SecondaryCommandBuffer {
    pub fn handle_mut(&mut self) -> Result<Mut<VkCommandBuffer>> {
        match Arc::get_mut(&mut self.0) {
            Some(inner) => Ok(inner.handle.borrow_mut()),
            None => Err(Error::SynchronizationError),
        }
    }
    /// Prevent the command buffer from being freed or submitted to a queue
    /// until the the value is dropped
    pub fn lock_self(&self) -> Arc<impl Send + Sync + Debug> {
        self.0.clone()
    }
    /// Prevent the command pool from being cleared or destroyed until the value
    /// is dropped.
    pub fn lock_resources(&self) -> Option<Arc<impl Send + Sync + Debug>> {
        self.0.recording.upgrade()
    }
}

impl<'a> Bindings<'a> {
    fn new(scratch: &'a bumpalo::Bump) -> Self {
        Self {
            layout: bumpalo::vec![in scratch],
            inited: bumpalo::vec![in scratch],
            pipeline: None,
        }
    }
}

impl<'a> CommandRecording<'a> {
    fn add_resource(&mut self, value: Arc<dyn Send + Sync + Debug>) {
        self.pool.resources.push(value);
    }
    /// A failed call to vkEndCommandBuffer leaves the buffer in the invalid
    /// state, so it is dropped in that case.
    pub fn end(mut self) -> Result<CommandBuffer> {
        unsafe {
            (self.pool.device.fun.end_command_buffer)(
                self.buffer.handle.borrow_mut(),
            )?;
        }
        self.buffer.recording = Arc::downgrade(self.recording);
        Ok(CommandBuffer(Arc::new(self.buffer)))
    }
}

impl<'a> SecondaryCommandRecording<'a> {
    /// A failed call to vkEndCommandBuffer leaves the buffer in the invalid
    /// state, so it is dropped in that case.
    pub fn end(mut self) -> Result<SecondaryCommandBuffer> {
        unsafe {
            (self.rec.pool.device.fun.end_command_buffer)(
                self.rec.buffer.handle.borrow_mut(),
            )?;
        }
        self.rec.buffer.recording = Arc::downgrade(self.rec.recording);
        Ok(SecondaryCommandBuffer(Arc::new(self.rec.buffer), self.subpass))
    }
}

impl<'a> CommandRecording<'a> {
    pub fn begin_render_pass(
        mut self,
        render_pass: &Arc<RenderPass>,
        framebuffer: &Arc<Framebuffer>,
        render_area: Rect2D,
        clear_values: &[ClearValue],
    ) -> RenderPassRecording<'a> {
        self.begin_render_pass_impl(
            render_pass,
            framebuffer,
            render_area,
            clear_values,
            SubpassContents::INLINE,
        );
        RenderPassRecording {
            rec: self,
            num_subpasses: render_pass.num_subpasses(),
            subpass: 0,
        }
    }
    pub fn begin_render_pass_secondary(
        mut self,
        render_pass: &Arc<RenderPass>,
        framebuffer: &Arc<Framebuffer>,
        render_area: Rect2D,
        clear_values: &[ClearValue],
    ) -> ExternalRenderPassRecording<'a> {
        self.begin_render_pass_impl(
            render_pass,
            framebuffer,
            render_area,
            clear_values,
            SubpassContents::SECONDARY_COMMAND_BUFFERS,
        );
        ExternalRenderPassRecording {
            rec: self,
            num_subpasses: render_pass.num_subpasses(),
            subpass: 0,
        }
    }
    fn begin_render_pass_impl(
        &mut self,
        render_pass: &Arc<RenderPass>,
        framebuffer: &Arc<Framebuffer>,
        render_area: Rect2D,
        clear_values: &[ClearValue],
        subpass_contents: SubpassContents,
    ) {
        self.add_resource(render_pass.clone());
        self.add_resource(framebuffer.clone());
        let info = RenderPassBeginInfo {
            stype: Default::default(),
            next: Default::default(),
            render_pass: render_pass.handle(),
            framebuffer: framebuffer.handle(),
            render_area,
            clear_values: clear_values.into(),
        };
        unsafe {
            (self.pool.device.fun.cmd_begin_render_pass)(
                self.buffer.handle.borrow_mut(),
                &info,
                subpass_contents,
            );
        }
    }
}

impl<'a> RenderPassRecording<'a> {
    pub fn next_subpass(&mut self) -> Result<()> {
        if self.subpass >= self.num_subpasses - 1 {
            return Err(Error::OutOfBounds);
        }
        self.subpass += 1;
        unsafe {
            (self.rec.pool.device.fun.cmd_next_subpass)(
                self.rec.buffer.handle.borrow_mut(),
                SubpassContents::INLINE,
            )
        }
        Ok(())
    }
    pub fn next_subpass_secondary(
        mut self,
    ) -> Result<ExternalRenderPassRecording<'a>> {
        if self.subpass >= self.num_subpasses - 1 {
            return Err(Error::OutOfBounds);
        }
        unsafe {
            (self.rec.pool.device.fun.cmd_next_subpass)(
                self.rec.buffer.handle.borrow_mut(),
                SubpassContents::SECONDARY_COMMAND_BUFFERS,
            );
        }
        Ok(ExternalRenderPassRecording {
            rec: self.rec,
            num_subpasses: self.num_subpasses,
            subpass: self.subpass + 1,
        })
    }
    pub fn end(mut self) -> Result<CommandRecording<'a>> {
        if self.subpass != self.num_subpasses - 1 {
            return Err(Error::InvalidArgument);
        }
        unsafe {
            (self.rec.pool.device.fun.cmd_end_render_pass)(
                self.rec.buffer.handle.borrow_mut(),
            );
        }
        Ok(self.rec)
    }
}

impl<'a> ExternalRenderPassRecording<'a> {
    pub fn next_subpass_secondary(&mut self) -> Result<()> {
        if self.subpass >= self.num_subpasses - 1 {
            return Err(Error::OutOfBounds);
        }
        self.subpass += 1;
        unsafe {
            (self.rec.pool.device.fun.cmd_next_subpass)(
                self.rec.buffer.handle.borrow_mut(),
                SubpassContents::SECONDARY_COMMAND_BUFFERS,
            )
        }
        Ok(())
    }
    pub fn next_subpass(mut self) -> Result<RenderPassRecording<'a>> {
        if self.subpass >= self.num_subpasses - 1 {
            return Err(Error::OutOfBounds);
        }
        unsafe {
            (self.rec.pool.device.fun.cmd_next_subpass)(
                self.rec.buffer.handle.borrow_mut(),
                SubpassContents::INLINE,
            );
        }
        Ok(RenderPassRecording {
            rec: self.rec,
            num_subpasses: self.num_subpasses,
            subpass: self.subpass + 1,
        })
    }
    pub fn end(mut self) -> Result<CommandRecording<'a>> {
        if self.subpass != self.num_subpasses - 1 {
            return Err(Error::InvalidArgument);
        }
        unsafe {
            (self.rec.pool.device.fun.cmd_end_render_pass)(
                self.rec.buffer.handle.borrow_mut(),
            );
        }
        Ok(self.rec)
    }
}
