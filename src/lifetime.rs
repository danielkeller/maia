use crate::load::{DeviceFn, InstanceFn};
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::Arc;

pub struct InstanceResource {
    pub handle: NonNull<c_void>,
    pub fun: InstanceFn,
}

impl Drop for InstanceResource {
    fn drop(&mut self) {
        unsafe { (self.fun.destroy_instance)(self.handle, None) }
    }
}

impl std::fmt::Debug for InstanceResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

pub struct DeviceResource {
    pub handle: NonNull<c_void>,
    pub fun: DeviceFn,
    pub instance: Arc<InstanceResource>,
}

impl Drop for DeviceResource {
    fn drop(&mut self) {
        unsafe { (self.fun.destroy_device)(self.handle, None) }
    }
}

impl std::fmt::Debug for DeviceResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}
