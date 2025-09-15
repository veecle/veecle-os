use core::error::Error;
use core::fmt::Display;

#[derive(Debug)]
#[non_exhaustive]
/// Error decoding a CAN frame.
pub enum CanDecodeError {
    /// The frame given to deserialize from had the wrong id.
    IncorrectId,

    /// The frame given to deserialize from had the wrong amount of data.
    IncorrectBufferSize,

    /// Value was out of range.
    OutOfRange {
        /// Name of the field.
        name: &'static str,
        /// Type of the field.
        ty: &'static str,
        /// Additional details (including the expected range).
        message: &'static str,
    },

    /// Validation failure.
    Invalid {
        /// Additional details about what was invalid.
        message: &'static str,
    },
}

impl CanDecodeError {
    /// Create an instance of [`Self::Invalid`].
    pub fn invalid(message: &'static str) -> Self {
        Self::Invalid { message }
    }
}

impl Display for CanDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            CanDecodeError::IncorrectId => write!(f, "incorrect CAN frame id"),
            CanDecodeError::IncorrectBufferSize => write!(f, "incorrect CAN frame size"),
            CanDecodeError::OutOfRange { name, ty, message } => {
                write!(f, "field {name}:{ty}: {message}")
            }
            CanDecodeError::Invalid { message } => write!(f, "validation failure: {message}"),
        }
    }
}

impl Error for CanDecodeError {}
