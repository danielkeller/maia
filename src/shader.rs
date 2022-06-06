use crate::device::Device;
use crate::error::{Error, Result};
use crate::types::*;

pub struct ShaderModule {
    handle: Handle<VkShaderModule>,
    device: Arc<Device>,
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
        Ok(ShaderModule { handle: handle.unwrap(), device: self.clone() })
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_shader_module)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl ShaderModule {
    pub fn borrow(&self) -> Ref<VkShaderModule> {
        self.handle.borrow()
    }
}
