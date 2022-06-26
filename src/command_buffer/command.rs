use crate::buffer::Buffer;
use crate::descriptor_set::DescriptorSet;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::ffi::Array;
use crate::image::Image;
use crate::pipeline::{Pipeline, PipelineLayout};
use crate::render_pass::RenderPass;
use crate::subobject::Owner;
use crate::types::*;

use super::{
    Bindings, CommandRecording, ExternalRenderPassRecording,
    RenderPassRecording, SecondaryCommandBuffer, SecondaryCommandRecording,
};

impl<'a> CommandRecording<'a> {
    /// The reference count of 'dst' is incremented. Offset and size are rounded
    /// down to the nearest multiple of 4. Returns [Error::OutOfBounds] if they
    /// are out of bounds.
    #[doc = crate::man_link!(vkCmdFillBuffer)]
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

    /// The reference counts of 'src' and 'dst' are incremented.
    /// Returns [Error::OutOfBounds] if a region is out of bounds.
    #[doc = crate::man_link!(vkCmdCopyBuffer)]
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

    /// The reference counts of 'src' and 'dst' are incremented.
    /// Returns [Error::OutOfBounds] if a region is out of bounds. Returns
    /// [Error::InvalidArgument] if 'regions' is empty.
    #[doc = crate::man_link!(vkCmdCopyBufferToImage)]
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

    /// The reference counts of 'src' and 'dst' are incremented.
    /// Returns [Error::OutOfBounds] if a region is out of bounds. Returns
    /// [Error::InvalidArgument] if 'regions' is empty.
    #[doc = crate::man_link!(vkCmdBlitImage)]
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

    /// The reference count of 'image' is incremented. Returns
    /// [Error::InvalidArgument] if 'ranges' is empty.
    #[doc = crate::man_link!(vkCmdClearColorImage)]
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

#[doc = crate::man_link!(VkBufferMemoryBarrier)]
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

#[doc = crate::man_link!(VkImageMemoryBarrier)]
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
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
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
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
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
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
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
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
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
    /// A shortcut for simple memory barriers.
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
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
    /// A shortcut for simple image barriers.
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
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
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
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

    /// A shortcut for simple memory barriers.
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
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

    /// A shortcut for simple image barriers.
    #[doc = crate::man_link!(vkCmdPipelineBarrier)]
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
    /// Binds the pipeline to the appropriate bind point. The reference count of
    /// 'pipeline' is incremented.
    #[doc = crate::man_link!(vkCmdBindPipeline)]
    pub fn bind_pipeline(&mut self, pipeline: &Arc<Pipeline>) {
        self.rec.bind_pipeline(pipeline)
    }
}

impl<'a> SecondaryCommandRecording<'a> {
    /// Binds the pipeline to the appropriate bind point. The reference count of
    /// 'pipeline' is incremented.
    #[doc = crate::man_link!(vkCmdBindPipeline)]
    pub fn bind_pipeline(&mut self, pipeline: &Arc<Pipeline>) {
        self.rec.bind_pipeline(pipeline)
    }
}

impl<'a> CommandRecording<'a> {
    /// Binds the pipeline to the appropriate bind point. The reference count of
    /// 'pipeline' is incremented.
    #[doc = crate::man_link!(vkCmdBindPipeline)]
    pub fn bind_pipeline(&mut self, pipeline: &Arc<Pipeline>) {
        if pipeline.render_pass().is_some() {
            self.graphics.pipeline = Some(pipeline.clone());
        } else {
            self.compute.pipeline = Some(pipeline.clone());
        }
        let bind_point = if pipeline.render_pass().is_some() {
            PipelineBindPoint::GRAPHICS
        } else {
            PipelineBindPoint::COMPUTE
        };
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
    #[doc = crate::man_link!(vkCmdSetViewport)]
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        self.rec.set_viewport(viewport)
    }
}
impl<'a> SecondaryCommandRecording<'a> {
    #[doc = crate::man_link!(vkCmdSetViewport)]
    pub fn set_viewport(&mut self, viewport: &Viewport) {
        self.rec.set_viewport(viewport)
    }
}
impl<'a> CommandRecording<'a> {
    #[doc = crate::man_link!(vkCmdSetViewport)]
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
    #[doc = crate::man_link!(vkCmdSetScissor)]
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        self.rec.set_scissor(scissor)
    }
}
impl<'a> SecondaryCommandRecording<'a> {
    #[doc = crate::man_link!(vkCmdSetScissor)]
    pub fn set_scissor(&mut self, scissor: &Rect2D) {
        self.rec.set_scissor(scissor)
    }
}
impl<'a> CommandRecording<'a> {
    #[doc = crate::man_link!(vkCmdSetScissor)]
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
    /// Reference counts of buffers are incremented. Returns
    /// [Error::InvalidArgument] if 'buffers_offsets' is empty.
    #[doc = crate::man_link!(vkCmdBindVertexBuffers)]
    pub fn bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers_offsets: &[(&Arc<Buffer>, u64)],
    ) -> Result<()> {
        self.rec.bind_vertex_buffers(first_binding, buffers_offsets)
    }
    /// Reference count of 'buffer' is incremented. Returns
    /// [Error::InvalidArgument] if 'buffers_offsets' is empty.
    #[doc = crate::man_link!(vkCmdBindIndexBuffer)]
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
    /// Reference counts of buffers are incremented. Returns
    /// [Error::InvalidArgument] if 'buffers_offsets' is empty.
    #[doc = crate::man_link!(vkCmdBindVertexBuffers)]
    pub fn bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers_offsets: &[(&Arc<Buffer>, u64)],
    ) -> Result<()> {
        self.rec.bind_vertex_buffers(first_binding, buffers_offsets)
    }
    /// Reference count of 'buffer' is incremented. Returns
    /// [Error::InvalidArgument] if 'buffers_offsets' is empty.
    #[doc = crate::man_link!(vkCmdBindIndexBuffer)]
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
    /// Reference counts of buffers are incremented. Returns
    /// [Error::InvalidArgument] if 'buffers_offsets' is empty.
    #[doc = crate::man_link!(vkCmdBindVertexBuffers)]
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
                Array::from_slice(buffers).ok_or(Error::InvalidArgument)?,
                Array::from_slice(offsets).ok_or(Error::InvalidArgument)?,
            )
        }
        Ok(())
    }
    /// Reference count of 'buffer' is incremented. Returns
    /// [Error::InvalidArgument] if 'buffers_offsets' is empty.
    #[doc = crate::man_link!(vkCmdBindIndexBuffer)]
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
    /// Returns [Error::InvalidArgument] if a member of 'sets' is not compatible
    /// with the corresponding member of 'layout', if the length of
    /// 'dynamic_offsets' is not correct for 'layout', or if any binding in any
    /// of 'sets' is not initialized.
    ///
    /// If the value of the binding will not be used, create a dummy object of
    /// the appropriate type and bind it.
    ///
    /// The reference count of each member of 'sets' is incremented.
    ///
    #[doc = crate::man_link!(vkCmdBindDescriptorSets)]
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
    /// Returns [Error::InvalidArgument] if a member of 'sets' is not compatible
    /// with the corresponding member of 'layout', if the length of
    /// 'dynamic_offsets' is not correct for 'layout', or if any binding in any
    /// of 'sets' is not initialized.
    ///
    /// If the value of the binding will not be used, create a dummy object of
    /// the appropriate type and bind it.
    ///
    /// The reference count of each member of 'sets' is incremented.
    ///
    #[doc = crate::man_link!(vkCmdBindDescriptorSets)]
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
            .unwrap_or_else(|| self.layout.len().min(layouts.len()));
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
    /// Returns [Error::InvalidArgument] if a member of 'sets' is not compatible
    /// with the corresponding member of 'layout', if the length of
    /// 'dynamic_offsets' is not correct for 'layout', or if any binding in any
    /// of 'sets' is not [initialized](DescriptorSet::is_initialized).
    ///
    /// If the value of the binding will not be used, create a dummy object of
    /// the appropriate type and bind it.
    ///
    /// The reference count of each member of 'sets' is incremented.
    ///
    #[doc = crate::man_link!(vkCmdBindDescriptorSets)]
    pub fn bind_descriptor_sets(
        &mut self,
        pipeline_bind_point: PipelineBindPoint,
        layout: &PipelineLayout,
        first_set: u32,
        sets: &[&Arc<DescriptorSet>],
        dynamic_offsets: &[u32],
    ) -> Result<()> {
        // Max binding is already checked by the layout
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
            || sets.iter().any(|s| !s.is_initialized())
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
    /// Sets push constants. Returns [Error::OutOfBounds] if the data is out of
    /// bounds for push contants in 'layout' or if 'stage_flags' is incorrect.
    /// Returns [Error::InvalidArgument] if 'data' is empty.
    #[doc = crate::man_link!(vkCmdPushConstants)]
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
    /// Sets push constants. Returns [Error::OutOfBounds] if the data is out of
    /// bounds for push contants in 'layout' or if 'stage_flags' is incorrect.
    /// Returns [Error::InvalidArgument] if 'data' is empty.
    #[doc = crate::man_link!(vkCmdPushConstants)]
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
    /// Sets push constants. Returns [Error::OutOfBounds] if the data is out of
    /// bounds for push contants in 'layout' or if 'stage_flags' is incorrect.
    /// Returns [Error::InvalidArgument] if 'data' is empty.
    #[doc = crate::man_link!(vkCmdPushConstants)]
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
        Err(Error::InvalidState)
    }
    fn check_render_pass(&self, pass: &RenderPass, subpass: u32) -> Result<()> {
        if let Some(pipeline) = self.pipeline.as_ref() {
            if pipeline.is_compatible_with(pass, subpass) {
                return Ok(());
            }
        }
        Err(Error::InvalidState)
    }
}

macro_rules! draw_state {
    () => {
        "Returns [Error::InvalidState] if the bound pipeline is not compatible 
        with the current render pass and subpass, if the bound descriptor sets 
        and bound graphics pipeline do not have a compatible layout, or if a 
        descriptor set mentioned in the pipeline's layout is not bound."
    }
}

impl<'a> RenderPassRecording<'a> {
    #[doc = draw_state!()]
    ///
    #[doc = crate::man_link!(vkCmdDraw)]
    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw(
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        )
    }
    #[doc = draw_state!()]
    ///
    /// The reference count of 'buffer' is incremented.
    ///
    #[doc = crate::man_link!(vkCmdDrawIndirect)]
    pub fn draw_indirect(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indirect(buffer, offset, draw_count, stride)
    }
    #[doc = draw_state!()]
    ///
    #[doc = crate::man_link!(vkCmdDrawIndexed)]
    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indexed(
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        )
    }
    #[doc = draw_state!()]
    ///
    /// The reference count of 'buffer' is incremented.
    ///
    #[doc = crate::man_link!(vkCmdDrawIndexedIndirect)]
    pub fn draw_indexed_indirect(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indexed_indirect(buffer, offset, draw_count, stride)
    }
}
impl<'a> SecondaryCommandRecording<'a> {
    #[doc = draw_state!()]
    ///
    #[doc = crate::man_link!(vkCmdDraw)]
    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw(
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        )
    }
    #[doc = draw_state!()]
    ///
    /// The reference count of 'buffer' is incremented.
    ///
    #[doc = crate::man_link!(vkCmdDrawIndirect)]
    pub fn draw_indirect(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indirect(buffer, offset, draw_count, stride)
    }
    #[doc = draw_state!()]
    ///
    #[doc = crate::man_link!(vkCmdDrawIndexed)]
    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
        self.rec.draw_indexed(
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        )
    }
    #[doc = draw_state!()]
    ///
    /// The reference count of 'buffer' is incremented.
    ///
    #[doc = crate::man_link!(vkCmdDrawIndexedIndirect)]
    pub fn draw_indexed_indirect(
        &mut self,
        buffer: &Arc<Buffer>,
        offset: u64,
        draw_count: u32,
        stride: u32,
    ) -> Result<()> {
        self.rec.graphics.check_render_pass(&self.pass, self.subpass)?;
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
    /// Returns [Error::InvalidArgument] if 'commands' is empty, if a member of
    /// 'commands' is not in the executable state, or if a member of 'commands'
    /// is not compatible with the current pass and subpass. Returns
    /// [Error::SynchronizationError] if a member of 'commands' is currently
    /// recorded to another command buffer.
    ///
    /// If a command was recorded from another pool, increments the reference
    /// count of that pool. That is, this pool must be reset before the other
    /// one can be. Note that this **does not** check for cycles of length
    /// greater than one: Adding a secondary command buffer from pool A to a
    /// primary from pool B, and a secondary from pool B to a primary from pool
    /// A will leak both pools.
    ///
    #[doc = crate::man_link!(vkCmdExecuteCommands)]
    pub fn execute_commands(
        &mut self,
        commands: &mut [&mut SecondaryCommandBuffer],
    ) -> Result<()> {
        let mut resources = bumpalo::vec![in self.rec.scratch];
        let mut handles = bumpalo::vec![in self.rec.scratch];
        for command in commands.iter_mut() {
            if !self.pass.compatible(command.pass.as_deref().unwrap())
                || self.subpass != command.subpass
            {
                return Err(Error::InvalidArgument);
            }
            // Check that the buffer is recorded.
            let res = command.lock_resources().ok_or(Error::InvalidArgument)?;
            // Require that this pool be reset before the other pool.
            if !Owner::ptr_eq(self.rec.pool, &command.buf.pool) {
                resources.push(res as Arc<_>);
            }
            // Check that the buffer is not in use.
            handles.push(command.mut_handle()?);
        }

        unsafe {
            (self.rec.pool.device.fun.cmd_execute_commands)(
                self.rec.buffer.handle.borrow_mut(),
                handles.len() as u32,
                Array::from_slice(&handles).ok_or(Error::InvalidArgument)?,
            )
        }

        drop(handles);
        self.rec.pool.resources.extend(resources);
        for command in commands {
            // Prevent this buffer from being reused.
            self.rec.pool.resources.push(command.lock_self());
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::vk;
    use std::sync::Arc;

    #[test]
    fn bounds_check() -> vk::Result<()> {
        let (dev, _) = crate::test_device()?;
        let buf = vk::BufferWithoutMemory::new(
            &dev,
            &vk::BufferCreateInfo { size: 1024, ..Default::default() },
        )?
        .allocate_memory(0)?;
        let img = vk::ImageWithoutMemory::new(
            &dev,
            &vk::ImageCreateInfo {
                extent: vk::Extent3D { width: 512, height: 512, depth: 1 },
                format: vk::Format::R8G8B8A8_SRGB,
                mip_levels: 10,
                ..Default::default()
            },
        )?
        .allocate_memory(0)?;
        let mut pool = vk::CommandPool::new(&dev, 0)?;
        let cmd = pool.allocate()?;
        let mut rec = pool.begin(cmd)?;
        assert!(rec.fill_buffer(&buf, 100, Some(1024), 42).is_err());
        assert!(rec.fill_buffer(&buf, 2000, None, 42).is_err());
        assert!(rec
            .copy_buffer(
                &buf,
                &buf,
                &[vk::BufferCopy {
                    size: 1024,
                    src_offset: 0,
                    dst_offset: 100
                }]
            )
            .is_err());
        assert!(rec
            .copy_buffer(
                &buf,
                &buf,
                &[vk::BufferCopy {
                    size: 1024,
                    src_offset: 100,
                    dst_offset: 0
                }]
            )
            .is_err());
        assert!(rec
            .copy_buffer_to_image(
                &buf,
                &img,
                vk::ImageLayout::GENERAL,
                &[vk::BufferImageCopy {
                    image_offset: vk::Offset3D { x: 5, y: 0, z: 0 },
                    image_extent: vk::Extent3D {
                        width: 512,
                        height: 512,
                        depth: 1
                    },
                    ..Default::default()
                }]
            )
            .is_err());
        assert!(rec
            .copy_buffer_to_image(
                &buf,
                &img,
                vk::ImageLayout::GENERAL,
                &[vk::BufferImageCopy {
                    image_extent: vk::Extent3D {
                        width: 512,
                        height: 512,
                        depth: 1
                    },
                    image_subresource: vk::ImageSubresourceLayers {
                        layer_count: 4,
                        ..Default::default()
                    },
                    ..Default::default()
                }]
            )
            .is_err());
        assert!(rec
            .copy_buffer_to_image(
                &buf,
                &img,
                vk::ImageLayout::GENERAL,
                &[vk::BufferImageCopy {
                    image_extent: vk::Extent3D {
                        width: 512,
                        height: 512,
                        depth: 1
                    },
                    ..Default::default()
                }]
            )
            .is_err());

        Ok(())
    }

    const SPV: &[u32] = &[
        0x07230203, 0x00010000, 0x000d000a, 0x00000006, 0x00000000, 0x00020011,
        0x00000001, 0x0006000b, 0x00000001, 0x4c534c47, 0x6474732e, 0x3035342e,
        0x00000000, 0x0003000e, 0x00000000, 0x00000001, 0x0005000f, 0x00000005,
        0x00000004, 0x6e69616d, 0x00000000, 0x00060010, 0x00000004, 0x00000011,
        0x00000001, 0x00000001, 0x00000001, 0x00030003, 0x00000002, 0x000001c2,
        0x000a0004, 0x475f4c47, 0x4c474f4f, 0x70635f45, 0x74735f70, 0x5f656c79,
        0x656e696c, 0x7269645f, 0x69746365, 0x00006576, 0x00080004, 0x475f4c47,
        0x4c474f4f, 0x6e695f45, 0x64756c63, 0x69645f65, 0x74636572, 0x00657669,
        0x00040005, 0x00000004, 0x6e69616d, 0x00000000, 0x00020013, 0x00000002,
        0x00030021, 0x00000003, 0x00000002, 0x00050036, 0x00000002, 0x00000004,
        0x00000000, 0x00000003, 0x000200f8, 0x00000005, 0x000100fd, 0x00010038,
    ];

    #[test]
    fn descriptor_set_typecheck() -> vk::Result<()> {
        let (dev, _) = crate::test_device()?;
        let mut cmd_pool = vk::CommandPool::new(&dev, 0)?;

        let ds_layout1 = vk::DescriptorSetLayout::new(
            &dev,
            vec![vk::DescriptorSetLayoutBinding {
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                immutable_samplers: vec![vk::Sampler::new(
                    &dev,
                    &Default::default(),
                )?],
            }],
        )?;

        let ds_layout2 = vk::DescriptorSetLayout::new(
            &dev,
            vec![vk::DescriptorSetLayoutBinding {
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                immutable_samplers: vec![],
            }],
        )?;

        let pipe_layout1 = vk::PipelineLayout::new(
            &dev,
            Default::default(),
            vec![ds_layout1.clone()],
            vec![],
        )?;
        let pipe_layout2 = vk::PipelineLayout::new(
            &dev,
            Default::default(),
            vec![ds_layout1.clone(), ds_layout2.clone()],
            vec![],
        )?;
        let pipe_layout3 = vk::PipelineLayout::new(
            &dev,
            Default::default(),
            vec![ds_layout2.clone(), ds_layout2.clone()],
            vec![],
        )?;
        let pipe = vk::Pipeline::new_compute(
            vk::PipelineShaderStageCreateInfo::compute(
                &vk::ShaderModule::new(&dev, SPV).unwrap(),
            ),
            &pipe_layout2,
            None,
        )
        .unwrap();

        let buf = vk::BufferWithoutMemory::new(
            &dev,
            &vk::BufferCreateInfo { size: 1024, ..Default::default() },
        )?
        .allocate_memory(0)?;
        let img = vk::ImageWithoutMemory::new(
            &dev,
            &vk::ImageCreateInfo {
                extent: vk::Extent3D { width: 512, height: 512, depth: 1 },
                format: vk::Format::R8G8B8A8_SRGB,
                mip_levels: 10,
                ..Default::default()
            },
        )?
        .allocate_memory(0)?;
        let img = vk::ImageView::new(&img, &Default::default())?;
        let mut desc_pool = vk::DescriptorPool::new(
            &dev,
            4,
            &[
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 4,
                },
                vk::DescriptorPoolSize {
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 4,
                },
            ],
        )?;

        let desc_set1 =
            Arc::new(vk::DescriptorSet::new(&mut desc_pool, &ds_layout1)?);

        let cmd = cmd_pool.allocate()?;
        let mut rec = cmd_pool.begin(cmd)?;

        // Can't bind uninitialized set
        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout1,
                0,
                &[&desc_set1],
                &[]
            )
            .is_err());

        let mut desc_set1 =
            vk::DescriptorSet::new(&mut desc_pool, &ds_layout1)?;
        let mut desc_set2 =
            vk::DescriptorSet::new(&mut desc_pool, &ds_layout2)?;

        vk::DescriptorSetUpdateBuilder::new(&dev)
            .begin()
            .dst_set(&mut desc_set1)
            .combined_image_samplers(0, 0, &[(&img, Default::default())])?
            .dst_set(&mut desc_set2)
            .uniform_buffers(
                0,
                0,
                &[vk::DescriptorBufferInfo {
                    buffer: &buf,
                    offset: 0,
                    range: 1024,
                }],
            )?
            .end();

        let desc_set1 = Arc::new(desc_set1);
        let desc_set2 = Arc::new(desc_set2);

        // Wrong layout
        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout2,
                1,
                &[&desc_set1],
                &[]
            )
            .is_err());

        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout1,
                0,
                &[&desc_set1],
                &[]
            )
            .is_ok());

        rec.bind_pipeline(&pipe);

        // Not everything bound
        assert!(rec.dispatch(1, 1, 1).is_err());

        // Invalidates earlier binding
        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout3,
                1,
                &[&desc_set2],
                &[]
            )
            .is_ok());
        assert!(rec.dispatch(1, 1, 1).is_err());

        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout1,
                0,
                &[&desc_set1],
                &[]
            )
            .is_ok());
        // Keeps earlier binding
        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout2,
                1,
                &[&desc_set2],
                &[]
            )
            .is_ok());
        assert!(rec.dispatch(1, 1, 1).is_ok());

        // Invalidate
        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout3,
                1,
                &[&desc_set2],
                &[]
            )
            .is_ok());

        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout2,
                1,
                &[&desc_set2],
                &[]
            )
            .is_ok());
        // Keeps later binding
        assert!(rec
            .bind_descriptor_sets(
                vk::PipelineBindPoint::COMPUTE,
                &pipe_layout1,
                0,
                &[&desc_set1],
                &[]
            )
            .is_ok());
        assert!(rec.dispatch(1, 1, 1).is_ok());

        Ok(())
    }
}
