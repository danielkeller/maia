use crate::device::Device;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::types::*;

/// A
#[doc = crate::spec_link!("render pass", "renderpass")]
///
/// Create with [Device::create_render_pass()].
#[derive(Debug)]
pub struct RenderPass {
    handle: Handle<VkRenderPass>,
    compat: RenderPassCompat,
    pub(crate) device: Arc<Device>,
}

impl Device {
    #[doc = crate::man_link!(vkCreateRenderPass)]
    pub fn create_render_pass(
        self: &Arc<Device>,
        info: &RenderPassCreateInfo,
    ) -> Result<Arc<RenderPass>> {
        let compat = RenderPassCompat::new(info)?;
        let mut handle = None;
        unsafe {
            (self.fun.create_render_pass)(
                self.handle(),
                info,
                None,
                &mut handle,
            )?;
        }
        let handle = handle.unwrap();
        Ok(Arc::new(RenderPass { handle, compat, device: self.clone() }))
    }
}

impl RenderPass {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkRenderPass> {
        self.handle.borrow()
    }
    /// Returns the number of subpasses.
    pub fn num_subpasses(&self) -> u32 {
        self.compat.subpasses.len() as u32
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Device {
        &*self.device
    }
    /// Returns true if this render pass is compatible with 'other'
    pub fn compatible(&self, other: &Self) -> bool {
        std::ptr::eq(self, other) || self.compat == other.compat
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_render_pass)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct AttachmentRefCompat {
    format: Format,
    samples: SampleCount,
}

#[derive(Debug, Eq)]
struct SubpassCompat {
    input_attachments: Vec<Option<AttachmentRefCompat>>,
    color_attachments: Vec<Option<AttachmentRefCompat>>,
    resolve_attachments: Vec<Option<AttachmentRefCompat>>,
    depth_stencil_attachments: Vec<Option<AttachmentRefCompat>>,
    preserve_attachments: Vec<Option<AttachmentRefCompat>>,
}

#[derive(Debug, PartialEq, Eq)]
struct RenderPassCompat {
    subpasses: Vec<SubpassCompat>,
    dependencies: Vec<SubpassDependency>,
}

fn flatten_ref<T>(opt: Option<&Option<T>>) -> Option<&T> {
    match opt {
        Some(Some(v)) => Some(v),
        _ => None,
    }
}

fn att_ref_array_compat(
    a: &[Option<AttachmentRefCompat>],
    b: &[Option<AttachmentRefCompat>],
) -> bool {
    for i in 0..a.len().max(b.len()) {
        if flatten_ref(a.get(i)) != flatten_ref(b.get(i)) {
            return false;
        }
    }
    return true;
}

impl PartialEq for SubpassCompat {
    fn eq(&self, other: &Self) -> bool {
        att_ref_array_compat(&self.input_attachments, &other.input_attachments)
            && att_ref_array_compat(
                &self.color_attachments,
                &other.color_attachments,
            )
            && att_ref_array_compat(
                &self.resolve_attachments,
                &other.resolve_attachments,
            )
            && att_ref_array_compat(
                &self.depth_stencil_attachments,
                &other.depth_stencil_attachments,
            )
            && att_ref_array_compat(
                &self.preserve_attachments,
                &other.preserve_attachments,
            )
    }
}

impl RenderPassCompat {
    fn new(info: &RenderPassCreateInfo) -> Result<Self> {
        let att_ref = |att: &AttachmentReference| {
            if att.attachment == u32::MAX {
                Ok(None)
            } else if let Some(desc) =
                info.attachments.as_slice().get(att.attachment as usize)
            {
                Ok(Some(AttachmentRefCompat {
                    format: desc.format,
                    samples: desc.samples,
                }))
            } else {
                Err(Error::OutOfBounds)
            }
        };
        let mut subpasses = vec![];
        for subpass in info.subpasses {
            subpasses.push(SubpassCompat {
                input_attachments: subpass
                    .input_attachments
                    .into_iter()
                    .map(att_ref)
                    .collect::<Result<_>>()?,
                preserve_attachments: subpass
                    .color_attachments
                    .into_iter()
                    .map(att_ref)
                    .collect::<Result<_>>()?,
                color_attachments: subpass
                    .color_attachments
                    .into_iter()
                    .map(att_ref)
                    .collect::<Result<_>>()?,
                resolve_attachments: subpass
                    .resolve_attachments
                    .map_or(Default::default(), |a| unsafe {
                        a.as_slice(subpass.color_attachments.len())
                    })
                    .iter()
                    .map(att_ref)
                    .collect::<Result<_>>()?,
                depth_stencil_attachments: subpass
                    .depth_stencil_attachments
                    .map_or(Default::default(), |a| unsafe {
                        a.as_slice(subpass.color_attachments.len())
                    })
                    .iter()
                    .map(att_ref)
                    .collect::<Result<_>>()?,
            });
        }

        Ok(Self {
            subpasses,
            dependencies: info.dependencies.into_iter().cloned().collect(),
        })
    }
}
