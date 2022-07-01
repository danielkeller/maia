// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::device::Device;
use crate::error::Result;
use crate::types::*;

/// A
#[doc = crate::spec_link!("sampler", "samplers")]
#[derive(Debug, Eq)]
pub struct Sampler {
    handle: Handle<VkSampler>,
    device: Arc<Device>,
}

impl Sampler {
    #[doc = crate::man_link!(vkCreateSampler)]
    pub fn new(
        device: &Arc<Device>, info: &SamplerCreateInfo,
    ) -> Result<Arc<Self>> {
        device.increment_sampler_alloc_count()?;
        let mut handle = None;
        let result = unsafe {
            (device.fun.create_sampler)(
                device.handle(),
                info,
                None,
                &mut handle,
            )
        };
        if result.is_err() {
            device.decrement_sampler_alloc_count();
            result?
        }
        Ok(Arc::new(Self { handle: handle.unwrap(), device: device.clone() }))
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_sampler)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
        self.device.decrement_sampler_alloc_count();
    }
}

impl PartialEq for Sampler {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl Sampler {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkSampler> {
        self.handle.borrow()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}
