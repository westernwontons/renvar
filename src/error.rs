use serde::de::Error as SerdeError;
use std::{error::Error as StdError, ffi::OsString, fmt};

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Crate level `Error` type
///
/// As per the serde convention, crates that use it
/// to create a Serializer and/or Deserializer are encouraged
/// to provide their own error type and [`crate::Result`] type alias
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Raised when any of the `from_os_env` functions/methods
    /// encounter invalid unicode in environment variables
    InvalidUnicode(OsString),

    /// Same purpose as [`serde::de::Error::missing_field`],
    MissingValue(String),

    /// Same purpose as [`serde::de::Error::custom`]
    Custom(String),
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidUnicode(invalid) => {
                write!(
                    fmt,
                    "invalid unicode found in string: {}",
                    invalid.to_string_lossy()
                )
            }
            Error::MissingValue(field) => write!(fmt, "missing value for {}", &field),
            Error::Custom(msg) => write!(fmt, "{}", msg),
        }
    }
}

impl SerdeError for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(format!("{}", msg))
    }

    fn missing_field(field: &'static str) -> Error {
        Error::MissingValue(field.into())
    }
}
