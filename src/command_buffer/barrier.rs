use crate::buffer::Buffer;
use crate::enums::*;
use crate::ffi::Array;
use crate::image::Image;
use crate::types::*;

use super::{CommandRecording, RenderPassRecording, SecondaryCommandRecording};

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
