use crate::types::VkError;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Other,
    InvalidArgument,
    SynchronizationError,
    NotReady,
    Timeout,
    OutOfHostMemory,
    InitializationFailed,
    ExtensionNotPresent,
    DeviceLost,
    SurfaceLostKHR,
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
            -1 => Self::OutOfHostMemory,
            -3 => Self::InitializationFailed,
            -4 => Self::DeviceLost,
            -7 => Self::ExtensionNotPresent,
            -1000000000 => Self::SurfaceLostKHR,
            1000001003 => Self::SuboptimalHKR,
            -1000001004 => Self::OutOfDateKHR,
            -1000255000 => Self::FullScreenExclusiveModeLostEXT,
            _ => Self::Other,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;