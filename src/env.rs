#![allow(
    unused_variables,
    unused_mut,
    unused_imports,
    dead_code,
    unused_assignments
)]

use serde::de::value::{Error as SerdeValueError, MapDeserializer, StringDeserializer};
use serde::de::IntoDeserializer;
use serde::de::{self, StdError};

use crate::{error::Error, forward_parsed_values};

////////////////////////////////////////////////////////////////////////////////////////////////////////

pub type Result<T> = std::result::Result<T, SerdeValueError>;

////////////////////////////////////////////////////////////////////////////////////////////////////////
