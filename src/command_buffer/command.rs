use crate::buffer::Buffer;
use crate::descriptor_set::DescriptorSet;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::ffi::Array;
use crate::image::Image;
use crate::pipeline::{Pipeline, PipelineLayout};
use crate::subobject::Owner;
use crate::types::*;

use super::{
    Bindings, CommandRecording, ExternalRenderPassRecording,
    RenderPassRecording, SecondaryCommandBuffer, SecondaryCommandRecording,
};

impl<'a> CommandRecording<'a> {
    /// Offset and size are rounded down to the nearest multiple of 4.
    pub fn fill_buffer(
        &mut self,
        dst: &Arc<Buffer>,
        offset: u64,
        size: Option<u64>,
        data: u32,
    ) -> Result<()> {
        let offset = offset & !3;
        let size = match size {
            Some(size) => {
                if !dst.bounds_check(offset, size) {
                    return Err(Error::OutOfBounds);
                }
                size & !3
            }
            None => {
                if !dst.bounds_check(offset, 0) {
                    return Err(Error::OutOfBounds);
                }
                u64::MAX
            }
        };
        self.add_resource(dst.clone());
        unsafe {
            (self.pool.device.fun.cmd_fill_buffer)(
                self.buffer.handle.borrow_mut(),
                dst.handle(),
                offset,
                size,
                data,
            );
        }
        Ok(())
    }

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
            (self.pool.device.fun.cmd_copy_buffer)(
                self.buffer.handle.borrow_mut(),
                src.handle(),
                dst.handle(),
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
            (self.pool.device.fun.cmd_copy_buffer_to_image)(
                self.buffer.handle.borrow_mut(),
                src.handle(),
                dst.handle(),
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
            (self.pool.device.fun.cmd_blit_image)(
                self.buffer.handle.borrow_mut(),
                src.handle(),
                src_layout,
                dst.handle(),
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
            (self.pool.device.fun.cmd_clear_color_image)(
                self.buffer.handle.borrow_mut(),
                image.handle(),
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
            buffer: self.buffer.handle(),
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
            image: self.image.handle(),
            subresource_range: self.subresource_range,
        }
    }
}

impl<'a> RenderPassRecording<'a> {
    pub fn pipeline_barrier(
        &mut self,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        dependency_flags: DependencyFlags,
        memory_barriers: &[MemoryBarrier],
        buffer_memory_barriers: &[BufferMemoryBarrier],
        image_memory_barriers: &[ImageMemoryBarrier],
    ) {
        self.rec.pipeline_barrier(
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
        self.rec.memory_barrier(
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
        self.rec.image_barrier(
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

impl<'a> SecondaryCommandRecording<'a> {
    pub fn pipeline_barrier(
        &mut self,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        dependency_flags: DependencyFlags,
        memory_barriers: &[MemoryBarrier],
        buffer_memory_barriers: &[BufferMemoryBarrier],
        image_memory_barriers: &[ImageMemoryBarrier],
    ) {
        self.rec.pipeline_barrier(
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
        self.rec.memory_barrier(
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
        self.rec.image_barrier(
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
        let vk_buffer_barriers = self.scratch.alloc_slice_fill_iter(
            buffer_memory_barriers.iter().map(|b| b.vk()),
        );
        let vk_image_barriers = self.scratch.alloc_slice_fill_iter(
            image_memory_barriers.iter().map(|b| b.vk()),
        );

        unsafe {
            (self.pool.device.fun.cmd_pipeline_barrier)(
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
            (self.pool.device.fun.cmd_pipeline_barrier)(
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
                image: image.handle(),
                subresource_range: Default::default(),
            };
            (self.pool.device.fun.cmd_pipeline_barrier)(
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

impl<'a> RenderPassRecording<'a> {
    pub fn bind_pipeline(
        &mut self,
        bind_point: PipelineBindPoint,
        pipeline: &Arc<Pipeline>,
    ) {
        self.rec.bind_pipeline(bind_point, pipeline)
    }
}

impl<'a> SecondaryCommandRecording<'a> {
    pub fn bind_pipeline(
        &mut self,
        bind_point: PipelineBindPoint,
        pipeline: &Arc<Pipeline>,
    ) {
        self.rec.bind_pipeline(bind_point, pipeline)
    }
}

impl<'a> CommandRecording<'a> {
    pub fn bind_pipeline(
        &mut self,
        bind_point: PipelineBindPoint,
        pipeline: &Arc<Pipeline>,
    ) {
        if bind_point == PipelineBindPoint::GRAPHICS {
            self.graphics.pipeline = Some(pipeline.clone());
        } else {
            self.compute.pipeline = Some(pipeline.clone());
        }
        self.add_resource(pipeline.clone());
        unsafe {
            (self.pool.device.fun.cmd_bind_pipeline)(
                self.buffer.handle.borrow_mut(),
                bind_point,
                pipeline.handle(),
            )
        }
    }
}

impl<'a> RenderPassRecording<'a> {
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        self.rec.set_viewport(viewport)
    }
}
impl<'a> SecondaryCommandRecording<'a> {
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        self.rec.set_viewport(viewport)
    }
}
impl<'a> CommandRecording<'a> {
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        unsafe {
            (self.pool.device.fun.cmd_set_viewport)(
                self.buffer.handle.borrow_mut(),
                0,
                1,
                std::array::from_ref(viewport).into(),
            )
        }
    }
}

impl<'a> RenderPassRecording<'a> {
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        self.rec.set_scissor(scissor)
    }
}
impl<'a> SecondaryCommandRecording<'a> {
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        self.rec.set_scissor(scissor)
    }
}
impl<'a> CommandRecording<'a> {
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        unsafe {
            (self.pool.device.fun.cmd_set_scissor)(
                self.buffer.handle.borrow_mut(),
                0,
                1,
                std::array::from_ref(scissor).into(),
            )
        }
    }
}

impl<'a> RenderPassRecording<'a> {
    pub fn bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers_offsets: &[(&Arc<Buffer>, u64)],
    ) -> Result<()> {
        self.rec.bind_vertex_buffers(first_binding, buffers_offsets)
    }
    pub fn bind_index_buffer(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        index_type: IndexType,
    ) {
        self.rec.bind_index_buffer(buffer, offset, index_type)
    }
}
impl<'a> SecondaryCommandRecording<'a> {
    pub fn bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers_offsets: &[(&Arc<Buffer>, u64)],
    ) -> Result<()> {
        self.rec.bind_vertex_buffers(first_binding, buffers_offsets)
    }
    pub fn bind_index_buffer(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        index_type: IndexType,
    ) {
        self.rec.bind_index_buffer(buffer, offset, index_type)
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
        let buffers = self.scratch.alloc_slice_fill_iter(
            buffers_offsets.iter().map(|&(b, _)| b.handle()),
        );
        let offsets = self.scratch.alloc_slice_fill_iter(
            buffers_offsets.iter().map(|&(_, o)| o), //
        );

        unsafe {
            (self.pool.device.fun.cmd_bind_vertex_buffers)(
                self.buffer.handle.borrow_mut(),
                first_binding,
                buffers.len() as u32,
                Array::from_slice(&buffers).ok_or(Error::InvalidArgument)?,
                Array::from_slice(&offsets).ok_or(Error::InvalidArgument)?,
            )
        }
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
            (self.pool.device.fun.cmd_bind_index_buffer)(
                self.buffer.handle.borrow_mut(),
                buffer.handle(),
                offset,
                index_type,
            )
        }
    }
}

impl<'a> RenderPassRecording<'a> {
    pub fn bind_descriptor_sets(
        &mut self,
        pipeline_bind_point: PipelineBindPoint,
        layout: &PipelineLayout,
        first_set: u32,
        sets: &[&Arc<DescriptorSet>],
        dynamic_offsets: &[u32],
    ) -> Result<()> {
        self.rec.bind_descriptor_sets(
            pipeline_bind_point,
            layout,
            first_set,
            sets,
            dynamic_offsets,
        )
    }
}
impl<'a> SecondaryCommandRecording<'a> {
    pub fn bind_descriptor_sets(
        &mut self,
        pipeline_bind_point: PipelineBindPoint,
        layout: &PipelineLayout,
        first_set: u32,
        sets: &[&Arc<DescriptorSet>],
        dynamic_offsets: &[u32],
    ) -> Result<()> {
        self.rec.bind_descriptor_sets(
            pipeline_bind_point,
            layout,
            first_set,
            sets,
            dynamic_offsets,
        )
    }
}

impl<'a> Bindings<'a> {
    fn bind_descriptor_sets(
        &mut self,
        layout: &PipelineLayout,
        begin: usize,
        sets: usize,
    ) {
        let end = begin + sets;
        let layouts = &layout.layouts()[0..end];
        let i = self
            .layout
            .iter()
            .zip(layouts.iter())
            .position(|(a, b)| a != b)
            .unwrap_or(self.layout.len().min(layouts.len()));
        if i < end {
            // Some bindings were invalidated
            self.layout.clear();
            self.layout.extend(layouts.iter().cloned());
            self.inited.resize(i, false);
            self.inited.resize(begin, false);
            self.inited.resize(end, true);
        } else {
            self.inited.resize(self.inited.len().max(end), false);
            self.inited[begin..end].fill(true);
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
        if sets.iter().map(|s| s.layout()).ne(layout
            .layouts()
            .iter()
            .skip(first_set as usize)
            .take(sets.len()))
            || sets
                .iter()
                .map(|s| s.layout().num_dynamic_offsets())
                .sum::<u32>()
                != dynamic_offsets.len() as u32
        {
            return Err(Error::InvalidArgument);
        }
        if pipeline_bind_point == PipelineBindPoint::GRAPHICS {
            self.graphics.bind_descriptor_sets(
                layout,
                first_set as usize,
                sets.len(),
            );
        } else {
            self.compute.bind_descriptor_sets(
                layout,
                first_set as usize,
                sets.len(),
            );
        }

        for &set in sets {
            self.add_resource(set.clone());
        }
        let sets =
            self.scratch.alloc_slice_fill_iter(sets.iter().map(|s| s.handle()));
        unsafe {
            (self.pool.device.fun.cmd_bind_descriptor_sets)(
                self.buffer.handle.borrow_mut(),
                pipeline_bind_point,
                layout.handle(),
                first_set,
                sets.len() as u32,
                Array::from_slice(sets),
                dynamic_offsets.len() as u32,
                Array::from_slice(dynamic_offsets),
            )
        }

        Ok(())
    }
}

impl<'a> RenderPassRecording<'a> {
    pub fn push_constants(
        &mut self,
        layout: &PipelineLayout,
        stage_flags: ShaderStageFlags,
        offset: u32,
        data: &[u8],
    ) -> Result<()> {
        self.rec.push_constants(layout, stage_flags, offset, data)
    }
}
impl<'a> SecondaryCommandRecording<'a> {
    pub fn push_constants(
        &mut self,
        layout: &PipelineLayout,
        stage_flags: ShaderStageFlags,
        offset: u32,
        data: &[u8],
    ) -> Result<()> {
        self.rec.push_constants(layout, stage_flags, offset, data)
    }
}
impl<'a> CommandRecording<'a> {
    pub fn push_constants(
        &mut self,
        layout: &PipelineLayout,
        stage_flags: ShaderStageFlags,
        offset: u32,
        data: &[u8],
    ) -> Result<()> {
        if !layout.bounds_check_push_constants(
            stage_flags,
            offset,
            data.len() as u32,
        ) {
            return Err(Error::OutOfBounds);
        }
        unsafe {
            (self.pool.device.fun.cmd_push_constants)(
                self.buffer.handle.borrow_mut(),
                layout.handle(),
                stage_flags,
                offset,
                data.len() as u32,
                Array::from_slice(data).ok_or(Error::InvalidArgument)?,
            );
        }
        Ok(())
    }
}

impl<'a> Bindings<'a> {
    fn check(&self) -> Result<()> {
        if let Some(pipeline) = self.pipeline.as_ref() {
            let layouts = &pipeline.layout().layouts();
            if self.layout.get(0..layouts.len()) == Some(layouts)
                && self.inited.iter().take_while(|b| **b).count()
                    >= layouts.len()
            {
                return Ok(());
            }
        }
        return Err(Error::InvalidState);
    }
}

impl<'a> RenderPassRecording<'a> {
    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        self.rec.draw(
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        )
    }
    pub fn draw_indirect(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.draw_indirect(buffer, offset, draw_count, stride)
    }
    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> Result<()> {
        self.rec.draw_indexed(
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        )
    }
    pub fn draw_indexed_indirect(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.draw_indexed_indirect(buffer, offset, draw_count, stride)
    }
}
impl<'a> SecondaryCommandRecording<'a> {
    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        self.rec.draw(
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        )
    }
    pub fn draw_indirect(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.draw_indirect(buffer, offset, draw_count, stride)
    }
    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> Result<()> {
        self.rec.draw_indexed(
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        )
    }
    pub fn draw_indexed_indirect(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.draw_indexed_indirect(buffer, offset, draw_count, stride)
    }
}
impl<'a> CommandRecording<'a> {
    fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        self.graphics.check()?;
        unsafe {
            (self.pool.device.fun.cmd_draw)(
                self.buffer.handle.borrow_mut(),
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        }
        Ok(())
    }
    fn draw_indirect(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.graphics.check()?;
        self.add_resource(buffer.clone());
        unsafe {
            (self.pool.device.fun.cmd_draw_indirect)(
                self.buffer.handle.borrow_mut(),
                buffer.handle(),
                offset,
                draw_count,
                stride,
            )
        }
        Ok(())
    }
    fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> Result<()> {
        self.graphics.check()?;
        unsafe {
            (self.pool.device.fun.cmd_draw_indexed)(
                self.buffer.handle.borrow_mut(),
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            )
        }
        Ok(())
    }
    fn draw_indexed_indirect(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.graphics.check()?;
        self.add_resource(buffer.clone());
        unsafe {
            (self.pool.device.fun.cmd_draw_indexed_indirect)(
                self.buffer.handle.borrow_mut(),
                buffer.handle(),
                offset,
                draw_count,
                stride,
            )
        }
        Ok(())
    }
}

impl<'a> CommandRecording<'a> {
    pub fn dispatch(
        &mut self,
        group_count_x: u32,
        group_count_y: u32,
        group_count_z: u32,
    ) -> Result<()> {
        self.compute.check()?;
        unsafe {
            (self.pool.device.fun.cmd_dispatch)(
                self.buffer.handle.borrow_mut(),
                group_count_x,
                group_count_y,
                group_count_z,
            );
        }
        Ok(())
    }
    pub fn dispatch_indirect(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
    ) -> Result<()> {
        self.compute.check()?;
        self.add_resource(buffer.clone());
        unsafe {
            (self.pool.device.fun.cmd_dispatch_indirect)(
                self.buffer.handle.borrow_mut(),
                buffer.handle(),
                offset,
            );
        }
        Ok(())
    }
}

impl<'a> ExternalRenderPassRecording<'a> {
    pub fn execute_commands(
        &mut self,
        commands: &mut [&mut SecondaryCommandBuffer],
    ) -> Result<()> {
        let mut resources = bumpalo::vec![in self.rec.scratch];
        let mut handles = bumpalo::vec![in self.rec.scratch];
        for command in commands {
            // Check that the buffer is recorded.
            let res = command.lock_resources().ok_or(Error::InvalidArgument)?;
            // Require that this pool be reset before the other pool.
            if !Owner::ptr_eq(self.rec.pool, &command.0.pool) {
                resources.push(res as Arc<_>);
            }
            // Check that the buffer is not in use.
            handles.push(command.handle_mut()?);
        }

        unsafe {
            (self.rec.pool.device.fun.cmd_execute_commands)(
                self.rec.buffer.handle.borrow_mut(),
                handles.len() as u32,
                Array::from_slice(&handles).ok_or(Error::InvalidArgument)?,
            )
        }

        self.rec.pool.resources.extend(resources);
        Ok(())
    }
}

// TODO: Less use *
// TODO: Building/distributing instructions
// TODO(maybe): Other kinds of command buffers
