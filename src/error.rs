use serde::de::Error as SerdeError;
use std::{error::Error as StdError, fmt};

////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    MissingValue(String),
    Custom(String),
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
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
