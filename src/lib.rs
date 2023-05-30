//! todo document

#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::wrong_self_convention,
    clippy::bare_urls,
    rustdoc::invalid_rust_codeblocks
)]

mod prefix;
mod error;
mod sanitize;
mod convert;
mod postfix;

pub mod de;

pub(crate) mod proc_macros;

////////////////////////////////////////////////////////////////////////////////////////////////////////

pub use convert::{from_env, from_iter, from_os_env, from_str};
pub use error::Error;

#[cfg(feature = "with_trimmer")]
pub use convert::with_trimmer::{
    from_env_with_trimmer, from_iter_with_trimmer, from_os_env_with_trimmer,
};

#[cfg(feature = "prefixed")]
pub use prefix::{prefixed, Prefixed};

#[cfg(feature = "case_insensitive_prefixed")]
pub use prefix::{case_insensitive_prefixed, CaseInsensitivePrefixed};

#[cfg(feature = "postfixed")]
pub use postfix::{postfixed, Postfixed};

#[cfg(feature = "case_insensitive_postfixed")]
pub use postfix::{case_insensitive_postfixed, CaseInsensitivePostfixed};

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// `Result` type alias used by this crate
pub type Result<T> = std::result::Result<T, Error>;
