use crate::device::Device;
use crate::error::{Error, Result};
use crate::types::*;

#[derive(Debug)]
pub struct RenderPass {
    handle: Handle<VkRenderPass>,
    num_subpasses: u32,
    pub(crate) device: Arc<Device>,
}

fn att_bounds(attachments: &[AttachmentReference], len: u32) -> Result<()> {
    if attachments.iter().any(|a| a.attachment >= len) {
        Err(Error::OutOfBounds)
    } else {
        Ok(())
    }
}

impl Device {
    pub fn create_render_pass(
        self: &Arc<Device>,
        info: &RenderPassCreateInfo,
    ) -> Result<Arc<RenderPass>> {
        let mut handle = None;
        unsafe {
            let len = info.attachments.len();
            for subpass in info.subpasses {
                att_bounds(subpass.input_attachments.as_slice(), len)?;
                att_bounds(subpass.color_attachments.as_slice(), len)?;
                att_bounds(subpass.preserve_attachments.as_slice(), len)?;
                let color_len = subpass.color_attachments.len();
                // Safety: Checked by TryFrom
                if let Some(resa) = &subpass.resolve_attachments {
                    att_bounds(resa.as_slice(color_len), len)?;
                }
                if let Some(dsa) = &subpass.depth_stencil_attachments {
                    att_bounds(dsa.as_slice(color_len), len)?;
                }
            }
            (self.fun.create_render_pass)(
                self.borrow(),
                info,
                None,
                &mut handle,
            )?;
        }
        let handle = handle.unwrap();
        Ok(Arc::new(RenderPass {
            handle,
            num_subpasses: info.subpasses.len(),
            device: self.clone(),
        }))
    }
}

impl RenderPass {
    pub fn borrow(&self) -> Ref<VkRenderPass> {
        self.handle.borrow()
    }
    pub fn num_subpasses(&self) -> u32 {
        self.num_subpasses
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_render_pass)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}
