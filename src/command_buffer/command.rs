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

impl<'a> CommandRecording<'a> {
    pub fn clear_color_image(
        &mut self,
        image: Arc<Image>,
        layout: ImageLayout,
        color: ClearColor,
        ranges: &[ImageSubresourceRange],
    ) -> Result<()> {
        let array = Array::from_slice(ranges).ok_or(Error::InvalidArgument)?;
        unsafe {
            (self.pool.res.device.fun.cmd_clear_color_image)(
                self.buffer.borrow_mut(),
                image.borrow(),
                layout,
                color.as_union(),
                ranges.len() as u32,
                array,
            )
        }

        self.add_resource(image);

        Ok(())
    }
}
