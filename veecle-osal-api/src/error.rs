/// A result with the [`Error`] error.
pub type Result<T> = core::result::Result<T, Error>;

/// An error that may happen during Veecle OS operations.
#[derive(Debug)]
pub enum Error {
    /// Run out of memory during the action.
    OutOfMemory,
    /// Could not apply the operation due to unknown error.
    Unknown,
}

impl core::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::OutOfMemory => write!(f, "out of memory"),
            Error::Unknown => write!(f, "unknown error"),
        }
    }
}
