use crate::buffer::Buffer;
use crate::descriptor_set::DescriptorSet;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::ffi::Array;
use crate::image::Image;
use crate::pipeline::{Pipeline, PipelineLayout};
use crate::types::*;

use super::{CommandRecording, RenderPassRecording};

impl<'a> CommandRecording<'a> {
    pub fn copy_buffer(
        &mut self,
        src: &Arc<Buffer>,
        dst: &Arc<Buffer>,
        regions: &[BufferCopy],
    ) -> Result<()> {
        for r in regions {
            if !src.bounds_check(r.src_offset, r.size)
                || !dst.bounds_check(r.dst_offset, r.size)
            {
                return Err(Error::OutOfBounds);
            }
        }
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
    pub fn copy_buffer_to_image(
        &mut self,
        src: &Arc<Buffer>,
        dst: &Arc<Image>,
        dst_layout: ImageLayout,
        regions: &[BufferImageCopy],
    ) -> Result<()> {
        for r in regions {
            let bytes = image_byte_size_3d(dst.format(), r.image_extent)
                .ok_or(Error::OutOfBounds)?
                .checked_mul(r.image_subresource.layer_count as u64)
                .ok_or(Error::OutOfBounds)?;
            if !dst.bounds_check(
                r.image_subresource.mip_level,
                r.image_offset,
                r.image_extent,
            ) || !dst.array_bounds_check(
                r.image_subresource.base_array_layer,
                r.image_subresource.layer_count,
            ) || !src.bounds_check(r.buffer_offset, bytes)
            {
                return Err(Error::OutOfBounds);
            }
        }
        unsafe {
            (self.pool.res.device.fun.cmd_copy_buffer_to_image)(
                self.buffer.handle.borrow_mut(),
                src.borrow(),
                dst.borrow(),
                dst_layout,
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
    pub fn blit_image(
        &mut self,
        src: &Arc<Image>,
        src_layout: ImageLayout,
        dst: &Arc<Image>,
        dst_layout: ImageLayout,
        regions: &[ImageBlit],
        filter: Filter,
    ) -> Result<()> {
        for r in regions {
            if !src.array_bounds_check(
                r.src_subresource.base_array_layer,
                r.src_subresource.layer_count,
            ) || !dst.array_bounds_check(
                r.dst_subresource.base_array_layer,
                r.dst_subresource.layer_count,
            ) || !src.offset_bounds_check(
                r.src_subresource.mip_level,
                r.src_offsets[0],
            ) || !src.offset_bounds_check(
                r.src_subresource.mip_level,
                r.src_offsets[1],
            ) || !dst.offset_bounds_check(
                r.dst_subresource.mip_level,
                r.dst_offsets[0],
            ) || !dst.offset_bounds_check(
                r.dst_subresource.mip_level,
                r.dst_offsets[1],
            ) {
                return Err(Error::OutOfBounds);
            }
        }
        unsafe {
            (self.pool.res.device.fun.cmd_blit_image)(
                self.buffer.handle.borrow_mut(),
                src.borrow(),
                src_layout,
                dst.borrow(),
                dst_layout,
                regions.len() as u32,
                Array::from_slice(regions).ok_or(Error::InvalidArgument)?,
                filter,
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

pub struct BufferMemoryBarrier<'a> {
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub buffer: &'a Arc<Buffer>,
    pub offset: u64,
    pub size: u64,
}
impl<'a> BufferMemoryBarrier<'a> {
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
pub struct ImageMemoryBarrier<'a> {
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub image: &'a Arc<Image>,
    pub subresource_range: ImageSubresourceRange,
}
impl<'a> ImageMemoryBarrier<'a> {
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
    /// A shortcut for simple memory barriers
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
    /// A shortcut for simple image barriers
    pub fn image_barrier(
        &mut self,
        image: &Arc<Image>,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
    ) {
        self.0.image_barrier(
            image,
            src_stage_mask,
            dst_stage_mask,
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
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
        let scratch = self.pool.scratch.get_mut();
        let vk_buffer_barriers = scratch.alloc_slice_fill_iter(
            buffer_memory_barriers.iter().map(|b| b.vk()),
        );
        let vk_image_barriers = scratch.alloc_slice_fill_iter(
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
        scratch.reset();
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

    /// A shortcut for simple image barriers
    pub fn image_barrier(
        &mut self,
        image: &Arc<Image>,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        src_access_mask: AccessFlags,
        dst_access_mask: AccessFlags,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
    ) {
        self.add_resource(image.clone());
        unsafe {
            let barrier = VkImageMemoryBarrier {
                stype: Default::default(),
                next: Default::default(),
                src_access_mask,
                dst_access_mask,
                old_layout,
                new_layout,
                src_queue_family_index: Default::default(),
                dst_queue_family_index: Default::default(),
                image: image.borrow(),
                subresource_range: Default::default(),
            };
            (self.pool.res.device.fun.cmd_pipeline_barrier)(
                self.buffer.handle.borrow_mut(),
                src_stage_mask,
                dst_stage_mask,
                Default::default(),
                0,
                None,
                0,
                None,
                1,
                Array::from_slice(&[barrier]),
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
    pub fn bind_index_buffer(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        index_type: IndexType,
    ) {
        self.0.bind_index_buffer(buffer, offset, index_type)
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
        let scratch = self.pool.scratch.get_mut();
        let buffers = scratch.alloc_slice_fill_iter(
            buffers_offsets.iter().map(|&(b, _)| b.borrow()),
        );
        let offsets = scratch.alloc_slice_fill_iter(
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
        scratch.reset();
        Ok(())
    }
    pub fn bind_index_buffer(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        index_type: IndexType,
    ) {
        self.add_resource(buffer.clone());
        unsafe {
            (self.pool.res.device.fun.cmd_bind_index_buffer)(
                self.buffer.handle.borrow_mut(),
                buffer.borrow(),
                offset,
                index_type,
            )
        }
    }
}

impl<'a> CommandRecording<'a> {
    pub fn bind_descriptor_sets(
        &mut self,
        pipeline_bind_point: PipelineBindPoint,
        layout: &PipelineLayout,
        first_set: u32,
        sets: &[&Arc<DescriptorSet>],
        dynamic_offsets: &[u32],
    ) -> Result<()> {
        // Unfortunately, typechecking this against the pipeline is a bit of
        // an issue, since only descriptors that are statically used are
        // required to typecheck, which requires inspecting the spir-v. I'm
        // inclined to say that this is out of scope for this crate, since the
        // result of incompatible bindings is the shader reading garbage.
        let mut num_dyn_offsets = 0;
        for (i, set) in sets.iter().enumerate() {
            if layout.layout(i as u32 + first_set) != Some(set.layout()) {
                return Err(Error::InvalidArgument);
            }
            num_dyn_offsets += set.layout().num_dynamic_offsets();
        }
        if dynamic_offsets.len() != num_dyn_offsets as usize {
            return Err(Error::InvalidArgument);
        }
        for &set in sets {
            self.add_resource(set.clone());
        }
        let scratch = self.pool.scratch.get_mut();
        let sets =
            scratch.alloc_slice_fill_iter(sets.iter().map(|s| s.borrow()));
        unsafe {
            (self.pool.res.device.fun.cmd_bind_descriptor_sets)(
                self.buffer.handle.borrow_mut(),
                pipeline_bind_point,
                layout.borrow(),
                first_set,
                sets.len() as u32,
                Array::from_slice(sets),
                num_dyn_offsets,
                Array::from_slice(dynamic_offsets),
            )
        }
        scratch.reset();

        Ok(())
    }
}
impl<'a, 'rec> RenderPassRecording<'a, 'rec> {
    pub fn bind_descriptor_sets(
        &mut self,
        pipeline_bind_point: PipelineBindPoint,
        layout: &PipelineLayout,
        first_set: u32,
        sets: &[&Arc<DescriptorSet>],
        dynamic_offsets: &[u32],
    ) -> Result<()> {
        self.0.bind_descriptor_sets(
            pipeline_bind_point,
            layout,
            first_set,
            sets,
            dynamic_offsets,
        )
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
    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            (self.0.pool.res.device.fun.cmd_draw_indexed)(
                self.0.buffer.handle.borrow_mut(),
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            )
        }
    }
}
