//! `renvar` is library to help deserialize environment variables into Rust datatypes
//!
//! # Example
//!
//! ```
//! use renvar::{from_env, from_iter, from_str};
//! use serde::Deserialize;
//! use std::env;
//!
//! let env_content = r#"
//! name=renvar
//! type=Library
//! dependencies=serde
//! "#;
//!
//! #[derive(Debug, Deserialize, PartialEq, Eq)]
//! enum CrateType {
//!     Library,
//!     Binary,
//! }
//!
//! #[derive(Debug, Deserialize, PartialEq, Eq)]
//! struct Renvar {
//!     name: String,
//!     #[serde(rename = "type")]
//!     typ: CrateType,
//!     dependencies: Vec<String>,
//! }
//!
//! let actual = Renvar {
//!     name: "renvar".to_owned(),
//!     typ: CrateType::Library,
//!     dependencies: vec!["serde".to_owned()],
//! };
//!
//! // we can read from strings
//!
//! let value = from_str::<Renvar>(env_content).unwrap();
//!
//! assert_eq!(value, actual);
//!
//! // directly from the environment
//!
//! let envs = vec![
//!     ("name".to_owned(), "renvar".to_owned()),
//!     ("type".to_owned(), "Library".to_owned()),
//!     ("dependencies".to_owned(), "serde".to_owned()),
//! ];
//!
//! for (key, value) in envs.clone().into_iter() {
//!     env::set_var(key, value);
//! }
//!
//! let value = from_env::<Renvar>().unwrap();
//!
//! assert_eq!(value, actual);
//!
//! // or from iterables
//!
//! let value = from_iter::<Renvar, _>(envs).unwrap();
//!
//! assert_eq!(value, actual);
//! ```
//!
//! # Feature flags
//!
//! Renvar has the following feature flags:
//! ## prefixed
//!
//! `prefixed` gives you the `prefixed` function, that accepts a prefix. The prefixes will be stripped away
//! before deserialization. Additionally, you'll also have access to `case_insensitive_prefixed`, where the casing
//! of the prefix doesn't matter, nor the casing of the environment variable keys.
//!
//! ## postfixed
//!
//! `postfix` is exactly the same as prefix, just with postfixes
//!
//! ## with_trimmer
//!
//! Finally, the `with_trimmer` feature flag gives you `*_with_trimmer` variants for all of the above,
//! where you can strip extraneous characters off of the beginning and env of strings with by passing a closure.

#![deny(
    missing_debug_implementations,
    missing_docs,
    clippy::missing_errors_doc,
    clippy::wrong_self_convention,
    rustdoc::invalid_rust_codeblocks
)]

#[cfg(feature = "prefixed")]
mod prefix;
#[cfg(feature = "postfixed")]
mod postfix;
mod error;
mod sanitize;
mod convert;

pub mod de;

#[doc(hidden)]
pub(crate) mod proc_macros;

////////////////////////////////////////////////////////////////////////////////////////////////////////

pub use convert::{from_env, from_iter, from_os_env, from_str};

#[cfg(feature = "prefixed")]
pub use prefix::{
    case_insensitive_prefixed, prefixed, CaseInsensitivePrefixed, Prefixed,
};

#[cfg(feature = "postfixed")]
pub use postfix::{
    case_insensitive_postfixed, postfixed, CaseInsensitivePostfixed, Postfixed,
};

#[cfg(feature = "with_trimmer")]
pub use convert::with_trimmer::{
    from_env_with_trimmer, from_iter_with_trimmer, from_os_env_with_trimmer,
};

////////////////////////////////////////////////////////////////////////////////////////////////////////

pub use error::Error;

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// `Result` type alias used by this crate
pub type Result<T> = std::result::Result<T, Error>;
