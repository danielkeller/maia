use crate::buffer::Buffer;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::ffi::Array;
use crate::image::Image;
use crate::pipeline::Pipeline;
use crate::types::*;

use super::{CommandRecording, RenderPassRecording};

impl<'a> CommandRecording<'a> {
    pub fn copy_buffer(
        &mut self,
        src: &Arc<Buffer>,
        dst: &Arc<Buffer>,
        regions: &[BufferCopy],
    ) -> Result<()> {
        unsafe {
            (self.pool.res.device.fun.cmd_copy_buffer)(
                self.buffer.handle.borrow_mut(),
                src.borrow(),
                dst.borrow(),
                regions.len() as u32,
                Array::from_slice(regions).ok_or(Error::InvalidArgument)?,
            );
        }
        self.add_resource(src.clone());
        self.add_resource(dst.clone());
        Ok(())
    }
}

impl<'a> CommandRecording<'a> {
    pub fn clear_color_image(
        &mut self,
        image: &Arc<Image>,
        layout: ImageLayout,
        color: ClearColorValue,
        ranges: &[ImageSubresourceRange],
    ) -> Result<()> {
        let array = Array::from_slice(ranges).ok_or(Error::InvalidArgument)?;
        unsafe {
            (self.pool.res.device.fun.cmd_clear_color_image)(
                self.buffer.handle.borrow_mut(),
                image.borrow(),
                layout,
                &color,
                ranges.len() as u32,
                array,
            )
        }

        self.add_resource(image.clone());

        Ok(())
    }
}

pub struct BufferMemoryBarrier {
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub buffer: Arc<Buffer>,
    pub offset: u64,
    pub size: u64,
}
impl BufferMemoryBarrier {
    fn vk(&self) -> VkBufferMemoryBarrier {
        VkBufferMemoryBarrier {
            stype: Default::default(),
            next: Default::default(),
            src_access_mask: self.src_access_mask,
            dst_access_mask: self.dst_access_mask,
            src_queue_family_index: self.src_queue_family_index,
            dst_queue_family_index: self.dst_queue_family_index,
            buffer: self.buffer.borrow(),
            offset: self.offset,
            size: self.size,
        }
    }
}
pub struct ImageMemoryBarrier {
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub image: Arc<Image>,
    pub subresource_range: ImageSubresourceRange,
}
impl ImageMemoryBarrier {
    fn vk(&self) -> VkImageMemoryBarrier {
        VkImageMemoryBarrier {
            stype: Default::default(),
            next: Default::default(),
            src_access_mask: self.src_access_mask,
            dst_access_mask: self.dst_access_mask,
            old_layout: self.old_layout,
            new_layout: self.new_layout,
            src_queue_family_index: self.src_queue_family_index,
            dst_queue_family_index: self.dst_queue_family_index,
            image: self.image.borrow(),
            subresource_range: self.subresource_range,
        }
    }
}

impl<'a, 'rec> RenderPassRecording<'a, 'rec> {
    pub fn pipeline_barrier(
        &mut self,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        dependency_flags: DependencyFlags,
        memory_barriers: &[MemoryBarrier],
        buffer_memory_barriers: &[BufferMemoryBarrier],
        image_memory_barriers: &[ImageMemoryBarrier],
    ) {
        self.0.pipeline_barrier(
            src_stage_mask,
            dst_stage_mask,
            dependency_flags,
            memory_barriers,
            buffer_memory_barriers,
            image_memory_barriers,
        )
    }
    pub fn memory_barrier(
        &mut self,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags,
    ) {
        self.0.memory_barrier(
            src_stage_mask,
            dst_stage_mask,
            src_access_mask,
            dst_access_mask,
        )
    }
}

impl<'a> CommandRecording<'a> {
    pub fn pipeline_barrier(
        &mut self,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        dependency_flags: DependencyFlags,
        memory_barriers: &[MemoryBarrier],
        buffer_memory_barriers: &[BufferMemoryBarrier],
        image_memory_barriers: &[ImageMemoryBarrier],
    ) {
        for b in buffer_memory_barriers {
            self.add_resource(b.buffer.clone());
        }
        for b in image_memory_barriers {
            self.add_resource(b.image.clone());
        }
        let vk_buffer_barriers = self.pool.scratch.alloc_slice_fill_iter(
            buffer_memory_barriers.iter().map(|b| b.vk()),
        );
        let vk_image_barriers = self.pool.scratch.alloc_slice_fill_iter(
            image_memory_barriers.iter().map(|b| b.vk()),
        );

        unsafe {
            (self.pool.res.device.fun.cmd_pipeline_barrier)(
                self.buffer.handle.borrow_mut(),
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                memory_barriers.len() as u32,
                Array::from_slice(memory_barriers),
                vk_buffer_barriers.len() as u32,
                Array::from_slice(vk_buffer_barriers),
                vk_image_barriers.len() as u32,
                Array::from_slice(vk_image_barriers),
            )
        }
        self.pool.scratch.reset();
    }

    /// A shortcut for simple memory barriers
    pub fn memory_barrier(
        &mut self,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags,
    ) {
        unsafe {
            (self.pool.res.device.fun.cmd_pipeline_barrier)(
                self.buffer.handle.borrow_mut(),
                src_stage_mask,
                dst_stage_mask,
                Default::default(),
                1,
                Some(Array::from(&[MemoryBarrier {
                    src_access_mask,
                    dst_access_mask,
                    ..Default::default()
                }])),
                0,
                None,
                0,
                None,
            )
        }
    }
}

impl<'a, 'rec> RenderPassRecording<'a, 'rec> {
    pub fn bind_pipeline(
        &mut self,
        bind_point: PipelineBindPoint,
        pipeline: &Arc<Pipeline>,
    ) {
        self.0.bind_pipeline(bind_point, pipeline)
    }
}

impl<'a> CommandRecording<'a> {
    pub fn bind_pipeline(
        &mut self,
        bind_point: PipelineBindPoint,
        pipeline: &Arc<Pipeline>,
    ) {
        self.add_resource(pipeline.clone());
        unsafe {
            (self.pool.res.device.fun.cmd_bind_pipeline)(
                self.buffer.handle.borrow_mut(),
                bind_point,
                pipeline.borrow(),
            )
        }
    }
}
impl<'a, 'rec> RenderPassRecording<'a, 'rec> {
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        self.0.set_viewport(viewport)
    }
}
impl<'a> CommandRecording<'a> {
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        unsafe {
            (self.pool.res.device.fun.cmd_set_viewport)(
                self.buffer.handle.borrow_mut(),
                0,
                1,
                std::array::from_ref(viewport).into(),
            )
        }
    }
}
impl<'a, 'rec> RenderPassRecording<'a, 'rec> {
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        self.0.set_scissor(scissor)
    }
}
impl<'a> CommandRecording<'a> {
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        unsafe {
            (self.pool.res.device.fun.cmd_set_scissor)(
                self.buffer.handle.borrow_mut(),
                0,
                1,
                std::array::from_ref(scissor).into(),
            )
        }
    }
}
impl<'a, 'rec> RenderPassRecording<'a, 'rec> {
    pub fn bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers_offsets: &[(&Arc<Buffer>, u64)],
    ) -> Result<()> {
        self.0.bind_vertex_buffers(first_binding, buffers_offsets)
    }
}
impl<'a> CommandRecording<'a> {
    pub fn bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers_offsets: &[(&Arc<Buffer>, u64)],
    ) -> Result<()> {
        for &(buffer, _) in buffers_offsets {
            self.add_resource(buffer.clone());
        }
        let buffers = self.pool.scratch.alloc_slice_fill_iter(
            buffers_offsets.iter().map(|&(b, _)| b.borrow()),
        );
        let offsets = self.pool.scratch.alloc_slice_fill_iter(
            buffers_offsets.iter().map(|&(_, o)| o), //
        );

        unsafe {
            (self.pool.res.device.fun.cmd_bind_vertex_buffers)(
                self.buffer.handle.borrow_mut(),
                first_binding,
                buffers.len() as u32,
                Array::from_slice(&buffers).ok_or(Error::InvalidArgument)?,
                Array::from_slice(&offsets).ok_or(Error::InvalidArgument)?,
            )
        }
        self.pool.scratch.reset();
        Ok(())
    }
}

impl<'a, 'rec> RenderPassRecording<'a, 'rec> {
    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            (self.0.pool.res.device.fun.cmd_draw)(
                self.0.buffer.handle.borrow_mut(),
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        }
    }
}
