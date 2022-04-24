#![feature(arbitrary_enum_discriminant)]
mod ffi;
mod instance;
mod lifetime;
mod load;
mod physical_device;
mod types;

use crate::types::*;

pub fn create_instance<'a>(
    info: &'a InstanceCreateInfo<'a>,
) -> Result<Instance> {
    let mut handle = None;
    unsafe { (load::vk_create_instance())(info, None, &mut handle)? };
    Ok(Instance::new(handle.unwrap()))
}

pub mod vk {
    pub use crate::create_instance;
    pub use crate::ffi::*;
    pub use crate::types::*;
}
