#![feature(arbitrary_enum_discriminant)]
pub mod ext;
mod ffi;
mod instance;
mod lifetime;
mod load;
mod physical_device;
mod types;
pub mod window;

use crate::types::*;

pub fn create_instance<'a>(
    info: &'a InstanceCreateInfo<'a>,
) -> Result<Instance> {
    let mut handle = None;
    unsafe { (load::vk_create_instance())(info, None, &mut handle)? };
    Ok(Instance::new(handle.unwrap()))
}

pub fn instance_extension_properties() -> Result<Vec<ExtensionProperties>> {
    let mut len = 0;
    let mut result = Vec::new();
    unsafe {
        let fn_ptr = load::vk_enumerate_instance_extension_properties();
        fn_ptr(None, &mut len, None)?;
        result.reserve(len.try_into().unwrap());
        fn_ptr(None, &mut len, result.spare_capacity_mut().first_mut())?;
        result.set_len(len.try_into().unwrap());
    }
    Ok(result)
}

pub mod vk {
    pub use crate::create_instance;
    pub use crate::ext;
    pub use crate::ffi::*;
    pub use crate::instance_extension_properties;
    pub use crate::types::*;
}
