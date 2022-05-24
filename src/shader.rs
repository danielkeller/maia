use crate::device::Device;
use crate::error::{Error, Result};
use crate::types::*;

pub struct ShaderModule {
    handle: Handle<VkShaderModule>,
    _device: Arc<Device>,
}

impl Device {
    pub fn create_shader_module(
        self: &Arc<Self>,
        code: &[u32],
    ) -> Result<ShaderModule> {
        if code.is_empty() {
            return Err(Error::InvalidArgument);
        }
        let mut handle = None;
        unsafe {
            (self.fun.create_shader_module)(
                self.borrow(),
                &VkShaderModuleCreateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: Default::default(),
                    code: code.into(),
                },
                None,
                &mut handle,
            )?;
        }
        Ok(ShaderModule { handle: handle.unwrap(), _device: self.clone() })
    }
}

impl ShaderModule {
    pub fn borrow(&self) -> Ref<VkShaderModule> {
        self.handle.borrow()
    }
}
