// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
/// An error either from Vulkan or Maia.
#[doc = crate::man_link!(VkResult)]
pub enum ErrorKind {
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

#[derive(Debug, Clone)]
struct ErrorInner {
    kind: ErrorKind,
    message: String,
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Error(Box<ErrorInner>);

/// An error either from Vulkan or Maia.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
struct ErrorAndInner<S> {
    error: Box<ErrorInner>,
    value: S,
}

/// For functions that take an argument by value and need to return it in case
/// of an error.
#[derive(Clone)]
#[repr(transparent)]
pub struct ErrorAnd<S>(Box<ErrorAndInner<S>>);

/// For functions that take an argument by value and need to return it in case
/// of an error.
pub type ResultAnd<T, S> = std::result::Result<T, ErrorAnd<S>>;

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self(Box::new(ErrorInner { kind, message: message.into() }))
    }
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidArgument, message)
    }
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidState, message)
    }
    pub fn out_of_bounds(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::OutOfBounds, message)
    }
    pub fn synchronization(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::SynchronizationError, message)
    }
    pub fn limit_exceeded(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::LimitExceeded, message)
    }
    pub fn and<S>(self, value: S) -> ErrorAnd<S> {
        ErrorAnd(Box::new(ErrorAndInner { error: self.0, value }))
    }
    pub fn kind(&self) -> ErrorKind {
        self.0.kind
    }
}

impl<S> ErrorAnd<S> {
    pub fn new(kind: ErrorKind, message: impl Into<String>, value: S) -> Self {
        Self(Box::new(ErrorAndInner {
            error: Box::new(ErrorInner { kind, message: message.into() }),
            value,
        }))
    }
    pub fn invalid_argument(message: impl Into<String>, value: S) -> Self {
        Self::new(ErrorKind::InvalidArgument, message, value)
    }
    pub fn invalid_state(message: impl Into<String>, value: S) -> Self {
        Self::new(ErrorKind::InvalidState, message, value)
    }
    pub fn out_of_bounds(message: impl Into<String>, value: S) -> Self {
        Self::new(ErrorKind::OutOfBounds, message, value)
    }
    pub fn synchronization(message: impl Into<String>, value: S) -> Self {
        Self::new(ErrorKind::SynchronizationError, message, value)
    }
    pub fn limit_exceeded(message: impl Into<String>, value: S) -> Self {
        Self::new(ErrorKind::LimitExceeded, message, value)
    }
    pub fn kind(&self) -> ErrorKind {
        self.0.error.kind
    }
    pub fn into_parts(self) -> (Error, S) {
        (Error(self.0.error), self.0.value)
    }
}

impl<S> From<ErrorAnd<S>> for Error {
    fn from(other: ErrorAnd<S>) -> Self {
        other.into_parts().0
    }
}

impl Display for ErrorInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}: {}", self.kind, self.message))
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
impl<T> Display for ErrorAnd<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0.error, f)
    }
}
impl<T> Debug for ErrorAnd<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorAnd")
            .field("error", &self.0.error)
            .finish_non_exhaustive()
    }
}

impl<T> std::error::Error for ErrorAnd<T> {}

#[doc = crate::man_link!(VkResult)]
#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct VkResult(i32);

impl ErrorKind {
    fn from_vk(err: VkResult) -> Option<Self> {
        match err.0 {
            0 => None,
            1 => Some(Self::NotReady),
            2 => Some(Self::Timeout),
            5 => Some(Self::Incomplete),
            -1 => Some(Self::OutOfHostMemory),
            -2 => Some(Self::OutOfDeviceMemory),
            -3 => Some(Self::InitializationFailed),
            -4 => Some(Self::DeviceLost),
            -7 => Some(Self::ExtensionNotPresent),
            -8 => Some(Self::FeatureNotPresent),
            -9 => Some(Self::IncompatibleDriver),
            -1000000000 => Some(Self::SurfaceLostKHR),
            -1000069000 => Some(Self::OutOfPoolMemory),
            1000001003 => Some(Self::SuboptimalHKR),
            -1000001004 => Some(Self::OutOfDateKHR),
            -1000255000 => Some(Self::FullScreenExclusiveModeLostEXT),
            _ => Some(Self::Other),
        }
    }
}

impl VkResult {
    pub fn context(self, func: &str) -> Result<()> {
        match ErrorKind::from_vk(self) {
            None => Ok(()),
            Some(kind) => Err(Error::new(kind, format!("In call to {}", func))),
        }
    }
}
