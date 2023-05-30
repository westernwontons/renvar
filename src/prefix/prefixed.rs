#![cfg(feature = "prefixed")]

use std::{env, string::String};

use serde::de;

use crate::convert::maybe_invalid_unicode_vars_os;
use crate::{from_iter, Result};

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Aids in deserializing some type `T` from environment variables,
/// where the keys are prefixed. Users are meant to obtain this struct
/// by calling [`prefixed`].
///
/// # Example
///
/// ```
/// use renvar::{prefixed, Prefixed};
///
/// let with_prefix: Prefixed = prefixed("APP_");
///
/// assert_eq!(with_prefix.prefix(), "APP_")
/// ```
#[derive(Debug)]
pub struct Prefixed<'a>(&'a str);

impl<'a> Prefixed<'a> {
    /// Deserialize some type `T` from a snapshot of the currently
    /// running process's environment variables at invocation time.
    ///
    /// # Panics
    /// if any of the environment variables contain invalid unicode
    ///
    /// # Example
    ///
    /// ```
    /// use renvar::{prefixed, Prefixed};
    /// use serde::Deserialize;
    /// use std::env;
    ///
    /// #[derive(Debug, Deserialize, PartialEq, Eq)]
    /// struct CustomStruct {
    ///     key: String,
    /// }
    ///
    /// let with_prefix: Prefixed = prefixed("APP_");
    ///
    /// let envs = vec![(String::from("APP_KEY"), String::from("value"))];
    ///
    /// for (key, value) in envs.into_iter() {
    ///     env::set_var(key, value);
    /// }
    ///
    /// let custom_struct: CustomStruct = with_prefix.from_env().unwrap();
    ///
    /// assert_eq!(
    ///     custom_struct,
    ///     CustomStruct {
    ///         key: String::from("value")
    ///     }
    /// )
    /// ```
    pub fn from_env<T>(&self) -> Result<T>
    where
        T: de::DeserializeOwned,
    {
        self.from_iter(env::vars())
    }

    /// Deserialize some type `T` from a snapshot of the currently
    /// running process's environment variables at invocation time, but doesn't panic
    /// if any of the environment variables contain invalid unicode, instead returns
    /// an error.
    ///
    /// # Example
    ///
    /// ```
    /// use renvar::{prefixed, Prefixed};
    /// use serde::Deserialize;
    /// use std::env;
    /// use std::ffi::OsString;
    ///
    /// #[derive(Debug, Deserialize, PartialEq, Eq)]
    /// struct CustomStruct {
    ///     key: String,
    /// }
    ///
    /// let with_prefix: Prefixed = prefixed("APP_");
    ///
    /// let envs = vec![(String::from("APP_KEY"), String::from("value"))];
    ///
    /// for (key, value) in envs.into_iter() {
    ///     env::set_var(key, value);
    /// }
    ///
    /// let custom_struct: CustomStruct = with_prefix.from_os_env().unwrap();
    ///
    /// assert_eq!(
    ///     custom_struct,
    ///     CustomStruct {
    ///         key: String::from("value")
    ///     }
    /// );
    /// ```
    pub fn from_os_env<T>(&self) -> Result<T>
    where
        T: de::DeserializeOwned,
    {
        self.from_iter(maybe_invalid_unicode_vars_os()?)
    }

    /// Deserialize some type `T` from an iterator `Iter` that is an iterator over key-value pairs,
    /// filtering only the pairs where the key ends with the specified prefix.
    ///
    /// # Example
    ///
    /// ```
    /// use renvar::{prefixed, Prefixed};
    /// use serde::Deserialize;
    ///
    /// #[derive(Debug, Deserialize, PartialEq, Eq)]
    /// struct CustomStruct {
    ///     key1: String,
    ///     key2: String,
    ///     key3: Option<String>,
    /// }
    ///
    /// let vars = vec![
    ///     ("APP_KEY1".to_owned(), "value1".to_owned()),
    ///     ("APP_KEY2".to_owned(), "value2".to_owned()),
    ///     ("APP_KEY3".to_owned(), "value3".to_owned()),
    /// ];
    ///
    /// let with_prefix: Prefixed = prefixed("APP_");
    /// let custom_struct: CustomStruct = with_prefix.from_iter(vars).unwrap();
    ///
    /// assert_eq!(
    ///     custom_struct,
    ///     CustomStruct {
    ///         key1: String::from("value1"),
    ///         key2: String::from("value2"),
    ///         key3: Some(String::from("value3"))
    ///     }
    /// )
    /// ```
    pub fn from_iter<T, Iter>(&self, iter: Iter) -> Result<T>
    where
        T: de::DeserializeOwned,
        Iter: IntoIterator<Item = (String, String)>,
    {
        from_iter(iter.into_iter().filter_map(|(k, v)| {
            if k.starts_with(self.0) {
                Some((k.trim_start_matches(self.0).to_owned(), v))
            } else {
                None
            }
        }))
    }

    /// Retrieve the prefix specified at the time
    /// of constructing an instance of [`Prefixed`]
    pub fn prefix(&self) -> &str {
        self.0
    }
}

/// Aids in deserializing some type `T` from environment variables,
/// where the keys are prefixed. Users are meant to obtain a [`Prefixed`]
/// struct by calling [`prefixed`].
///
/// # Example
///
/// ```
/// use renvar::{prefixed, Prefixed};
///
/// let with_prefix = prefixed("APP_");
///
/// assert_eq!(with_prefix.prefix(), "APP_")
/// ```
pub fn prefixed(prefix: &str) -> Prefixed<'_> {
    Prefixed(prefix)
}

#[cfg(test)]
mod test_prefixed {
    use serde::Deserialize;
    use std::env;

    use super::prefixed;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct Test {
        key: String,
    }

    #[test]
    fn test_prefixed() {
        env::set_var("APP_KEY", "value");
        let prefixed = prefixed("APP_").from_env::<Test>().unwrap();

        assert_eq!(
            prefixed,
            Test {
                key: String::from("value")
            }
        )
    }
}
