use crate::device::Device;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::image::ImageView;
use crate::render_pass::RenderPass;
use crate::types::*;

#[derive(Debug)]
pub struct Framebuffer {
    handle: Handle<VkFramebuffer>,
    _attachments: Vec<Arc<ImageView>>,
    device: Arc<Device>,
}

impl RenderPass {
    pub fn create_framebuffer(
        &self,
        flags: FramebufferCreateFlags,
        attachments: Vec<Arc<ImageView>>,
        size: Extent3D,
    ) -> Result<Arc<Framebuffer>> {
        if attachments.iter().any(|iv| iv.image.device != self.device) {
            return Err(Error::InvalidArgument);
        }
        let vk_attachments: Vec<_> =
            attachments.iter().map(|iv| iv.borrow()).collect();
        let vk_create_info = VkFramebufferCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags,
            render_pass: self.borrow(),
            attachments: (&vk_attachments).into(),
            width: size.width,
            height: size.height,
            layers: size.depth,
        };
        let mut handle = None;
        unsafe {
            (self.device.fun.create_framebuffer)(
                self.device.borrow(),
                &vk_create_info,
                None,
                &mut handle,
            )?;
        }
        Ok(Arc::new(Framebuffer {
            handle: handle.unwrap(),
            _attachments: attachments,
            device: self.device.clone(),
        }))
    }
}

impl Framebuffer {
    pub fn borrow(&self) -> Ref<VkFramebuffer> {
        self.handle.borrow()
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_framebuffer)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}
