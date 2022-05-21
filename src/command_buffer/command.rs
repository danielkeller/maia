use crate::command_buffer::CommandRecording;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::ffi::Array;
use crate::image::Image;
use crate::types::*;

#[derive(Clone, Copy, Debug)]
pub enum ClearColor {
    F32([f32; 4]),
    I32([i32; 4]),
    U32([u32; 4]),
}
impl Default for ClearColor {
    /// Black for any format
    fn default() -> Self {
        Self::U32([0, 0, 0, 0])
    }
}

impl ClearColor {
    pub fn as_union(&self) -> &ClearColorValue {
        match self {
            ClearColor::F32(arr) => unsafe { std::mem::transmute(arr) },
            ClearColor::I32(arr) => unsafe { std::mem::transmute(arr) },
            ClearColor::U32(arr) => unsafe { std::mem::transmute(arr) },
        }
    }
}

// TODO
// pub struct BufferMemoryBarrier {}
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

impl<'a> CommandRecording<'a> {
    pub fn clear_color_image(
        &mut self,
        image: &Arc<Image>,
        layout: ImageLayout,
        color: ClearColor,
        ranges: &[ImageSubresourceRange],
    ) -> Result<()> {
        let array = Array::from_slice(ranges).ok_or(Error::InvalidArgument)?;
        unsafe {
            (self.pool.res.device.fun.cmd_clear_color_image)(
                self.buffer.handle.borrow_mut(),
                image.borrow(),
                layout,
                color.as_union(),
                ranges.len() as u32,
                array,
            )
        }

        self.add_resource(image.clone());

        Ok(())
    }

    pub fn pipeline_barrier(
        &mut self,
        src_stage_mask: PipelineStageFlags,
        dst_stage_mask: PipelineStageFlags,
        dependency_flags: DependencyFlags,
        memory_barriers: &[MemoryBarrier],
        buffer_memory_barriers: &[()],
        image_memory_barriers: &[ImageMemoryBarrier],
    ) {
        let vk_buffer_barriers: Vec<_> =
            buffer_memory_barriers.iter().map(|_| unimplemented!()).collect();
        let vk_image_barriers: Vec<_> = image_memory_barriers
            .iter()
            .map(|barrier| {
                self.add_resource(barrier.image.clone());
                VkImageMemoryBarrier {
                    stype: Default::default(),
                    next: Default::default(),
                    src_access_mask: barrier.src_access_mask,
                    dst_access_mask: barrier.dst_access_mask,
                    old_layout: barrier.old_layout,
                    new_layout: barrier.new_layout,
                    src_queue_family_index: barrier.src_queue_family_index,
                    dst_queue_family_index: barrier.dst_queue_family_index,
                    image: barrier.image.borrow(),
                    subresource_range: barrier.subresource_range,
                }
            })
            .collect();

        unsafe {
            (self.pool.res.device.fun.cmd_pipeline_barrier)(
                self.buffer.handle.borrow_mut(),
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                memory_barriers.len() as u32,
                Array::from_slice(memory_barriers),
                vk_buffer_barriers.len() as u32,
                Array::from_slice(&vk_buffer_barriers),
                vk_image_barriers.len() as u32,
                Array::from_slice(&vk_image_barriers),
            )
        }
    }
}
