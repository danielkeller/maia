use crate::types::VkError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    Other,
    InvalidArgument,
    OutOfBounds,
    SynchronizationError,
    NotReady,
    Timeout,
    Incomplete,
    OutOfHostMemory,
    InitializationFailed,
    ExtensionNotPresent,
    DeviceLost,
    SurfaceLostKHR,
    OutOfPoolMemory,
    SuboptimalHKR,
    OutOfDateKHR,
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
            -3 => Self::InitializationFailed,
            -4 => Self::DeviceLost,
            -7 => Self::ExtensionNotPresent,
            -1000000000 => Self::SurfaceLostKHR,
            -1000069000 => Self::OutOfPoolMemory,
            1000001003 => Self::SuboptimalHKR,
            -1000001004 => Self::OutOfDateKHR,
            -1000255000 => Self::FullScreenExclusiveModeLostEXT,
            _ => Self::Other,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

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
impl<T> std::error::Error for ErrorAndSelf<T> {}
pub type ResultAndSelf<T, S> = std::result::Result<T, ErrorAndSelf<S>>;
