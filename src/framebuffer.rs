use crate::enums::*;
use crate::error::Result;
use crate::image::ImageView;
use crate::render_pass::RenderPass;
use crate::types::*;

#[derive(Debug)]
pub struct Framebuffer {
    handle: Handle<VkFramebuffer>,
    _attachments: Vec<Arc<ImageView>>,
    render_pass: Arc<RenderPass>,
}

impl RenderPass {
    pub fn create_framebuffer(
        self: &Arc<Self>,
        flags: FramebufferCreateFlags,
        attachments: Vec<Arc<ImageView>>,
        size: Extent3D,
    ) -> Result<Arc<Framebuffer>> {
        for iv in &attachments {
            assert_eq!(iv.device(), self.device());
        }
        let vk_attachments: Vec<_> =
            attachments.iter().map(|iv| iv.handle()).collect();
        let vk_create_info = VkFramebufferCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags,
            render_pass: self.handle(),
            attachments: (&vk_attachments).into(),
            width: size.width,
            height: size.height,
            layers: size.depth,
        };
        let mut handle = None;
        unsafe {
            (self.device().fun.create_framebuffer)(
                self.device().handle(),
                &vk_create_info,
                None,
                &mut handle,
            )?;
        }
        Ok(Arc::new(Framebuffer {
            handle: handle.unwrap(),
            _attachments: attachments,
            render_pass: self.clone(),
        }))
    }
}

impl Framebuffer {
    pub fn handle(&self) -> Ref<VkFramebuffer> {
        self.handle.borrow()
    }
    pub fn is_compatible_with(&self, pass: &RenderPass) -> bool {
        self.render_pass.compatible(pass)
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            (self.render_pass.device.fun.destroy_framebuffer)(
                self.render_pass.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}
