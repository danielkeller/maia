// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::types::VkError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
/// An error either from Vulkan or Maia.
#[doc = crate::man_link!(VkResult)]
pub enum Error {
    /// Unknown Vulkan error.
    Other,
    /// The arguments provided to the function were incorrect.
    InvalidArgument,
    /// The object is misconfigured for the requested operation.
    InvalidState,
    /// The given index was out of bounds of the object.
    OutOfBounds,
    /// A required operation has not yet completed.
    SynchronizationError,
    /// The arguments exceed the limits of the device.
    LimitExceeded,
    #[doc = crate::man_link!(VkResult)]
    NotReady,
    #[doc = crate::man_link!(VkResult)]
    Timeout,
    #[doc = crate::man_link!(VkResult)]
    Incomplete,
    #[doc = crate::man_link!(VkResult)]
    OutOfHostMemory,
    #[doc = crate::man_link!(VkResult)]
    OutOfDeviceMemory,
    #[doc = crate::man_link!(VkResult)]
    InitializationFailed,
    #[doc = crate::man_link!(VkResult)]
    ExtensionNotPresent,
    #[doc = crate::man_link!(VkResult)]
    FeatureNotPresent,
    #[doc = crate::man_link!(VkResult)]
    IncompatibleDriver,
    #[doc = crate::man_link!(VkResult)]
    DeviceLost,
    #[doc = crate::man_link!(VkResult)]
    SurfaceLostKHR,
    #[doc = crate::man_link!(VkResult)]
    OutOfPoolMemory,
    #[doc = crate::man_link!(VkResult)]
    SuboptimalHKR,
    #[doc = crate::man_link!(VkResult)]
    OutOfDateKHR,
    #[doc = crate::man_link!(VkResult)]
    FullScreenExclusiveModeLostEXT,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for Error {}

impl From<VkError> for Error {
    fn from(err: VkError) -> Self {
        match err.0.get() {
            1 => Self::NotReady,
            2 => Self::Timeout,
            5 => Self::Incomplete,
            -1 => Self::OutOfHostMemory,
            -2 => Self::OutOfDeviceMemory,
            -3 => Self::InitializationFailed,
            -4 => Self::DeviceLost,
            -7 => Self::ExtensionNotPresent,
            -8 => Self::FeatureNotPresent,
            -9 => Self::IncompatibleDriver,
            -1000000000 => Self::SurfaceLostKHR,
            -1000069000 => Self::OutOfPoolMemory,
            1000001003 => Self::SuboptimalHKR,
            -1000001004 => Self::OutOfDateKHR,
            -1000255000 => Self::FullScreenExclusiveModeLostEXT,
            _ => Self::Other,
        }
    }
}

/// An error either from Vulkan or Maia.
pub type Result<T> = std::result::Result<T, Error>;

/// For functions that take an argument by value and need to return it in case
/// of an error.
pub struct ErrorAndSelf<T>(pub Error, pub T);

impl<T> std::fmt::Debug for ErrorAndSelf<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}
impl<T> std::fmt::Display for ErrorAndSelf<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}
impl<T> From<ErrorAndSelf<T>> for Error {
    fn from(ErrorAndSelf(err, _): ErrorAndSelf<T>) -> Self {
        err
    }
}

impl<T> std::error::Error for ErrorAndSelf<T> {}
/// For functions that take an argument by value and need to return it in case
/// of an error.
pub type ResultAndSelf<T, S> = std::result::Result<T, ErrorAndSelf<S>>;
