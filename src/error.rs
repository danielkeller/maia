use crate::types::VkError;

#[derive(Debug)]
pub enum Error {
    Other,
    InvalidArgument,
    SynchronizationError,
    OutOfHostMemory,
    InitializationFailed,
    ExtensionNotPresent,
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
            -1 => Self::OutOfHostMemory,
            -3 => Self::InitializationFailed,
            -7 => Self::ExtensionNotPresent,
            _ => Self::Other,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
