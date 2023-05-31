#![doc = include_str!("docs/crate.md")]
#![deny(
    missing_debug_implementations,
    missing_docs,
    clippy::missing_errors_doc,
    clippy::wrong_self_convention,
    rustdoc::invalid_rust_codeblocks
)]
#![allow(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "prefixed")]
mod prefixed;
#[cfg(feature = "case_insensitive_prefixed")]
mod case_insensitive_prefixed;
#[cfg(feature = "postfixed")]
mod postfixed;
#[cfg(feature = "case_insensitive_postfixed")]
mod case_insensitive_postfixed;
mod error;
mod sanitize;
mod convert;

pub mod de;

pub(crate) mod proc_macros;

////////////////////////////////////////////////////////////////////////////////////////////////////////

pub use convert::{from_env, from_iter, from_os_env, from_str};

#[cfg(feature = "prefixed")]
pub use prefixed::{prefixed, Prefixed};

#[cfg(feature = "case_insensitive_prefixed")]
pub use case_insensitive_prefixed::{
    case_insensitive_prefixed, CaseInsensitivePrefixed,
};
#[cfg(feature = "postfixed")]
pub use postfixed::{postfixed, Postfixed};

#[cfg(feature = "case_insensitive_prefixed")]
pub use case_insensitive_postfixed::{
    case_insensitive_postfixed, CaseInsensitivePostfixed,
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
