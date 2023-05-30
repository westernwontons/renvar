use crate::convert::maybe_invalid_unicode_vars_os;
use crate::{from_iter, Result};
use serde::de;
use std::{env, string::String};

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Aids in deserializing some type `T` from environment variables,
/// where the keys are postfixed. Users are meant to obtain this struct
/// by calling [`postfixed`].
///
/// # Example
///
/// ```
/// use renvar::{postfixed, Postfixed};
///
/// let with_postfix: Postfixed = postfixed("_APP");
///
/// assert_eq!(with_postfix.postfix(), "_APP")
/// ```
#[derive(Debug)]
pub struct Postfixed<'a>(&'a str);

impl<'a> Postfixed<'a> {
    /// Deserialize some type `T` from a snapshot of the currently
    /// running process's environment variables at invocation time.
    ///
    /// # Errors
    ///
    /// Any errors that might occur during deserialization
    ///
    /// # Panics
    /// if any of the environment variables contain invalid unicode
    ///
    /// # Example
    ///
    /// ```
    /// use renvar::{postfixed, Postfixed};
    /// use serde::Deserialize;
    /// use std::env;
    ///
    /// #[derive(Debug, Deserialize, PartialEq, Eq)]
    /// struct CustomStruct {
    ///     key: String,
    /// }
    ///
    /// let envs = vec![("KEY_APP".to_owned(), "value".to_owned())];
    ///
    /// for (key, value) in envs.into_iter() {
    ///     env::set_var(key, value);
    /// }
    ///
    /// let with_postfix: Postfixed = postfixed("_APP");
    /// let custom_struct: CustomStruct = with_postfix.from_env().unwrap();
    ///
    /// assert_eq!(
    ///     custom_struct,
    ///     CustomStruct {
    ///         key: "value".to_owned()
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
    /// # Errors
    ///
    /// Any errors that might occur during deserialization
    ///
    /// # Example
    ///
    /// ```
    /// use renvar::{postfixed, Postfixed};
    /// use serde::Deserialize;
    /// use std::env;
    /// use std::ffi::OsString;
    ///
    /// #[derive(Debug, Deserialize, PartialEq, Eq)]
    /// struct CustomStruct {
    ///     key: String,
    /// }
    ///
    /// let envs = vec![("KEY_APP".to_owned(), "value".to_owned())];
    ///
    /// for (key, value) in envs.into_iter() {
    ///     env::set_var(key, value);
    /// }
    ///
    /// let with_postfix: Postfixed = postfixed("_APP");
    /// let custom_struct: CustomStruct = with_postfix.from_os_env().unwrap();
    ///
    /// assert_eq!(
    ///     custom_struct,
    ///     CustomStruct {
    ///         key: "value".to_owned()
    ///     }
    /// )
    /// ```
    pub fn from_os_env<T>(&self) -> Result<T>
    where
        T: de::DeserializeOwned,
    {
        self.from_iter(maybe_invalid_unicode_vars_os()?)
    }

    /// Deserialize some type `T` from an iterator `Iter` that is an iterator over key-value pairs,
    /// filtering only the pairs where the key ends with the specified postfix.
    ///
    /// # Errors
    ///
    /// Any errors that might occur during deserialization
    ///
    /// # Example
    ///
    /// ```
    /// use renvar::{postfixed, Postfixed};
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
    ///     ("KEY1_APP".to_owned(), "value1".to_owned()),
    ///     ("KEY2_APP".to_owned(), "value2".to_owned()),
    ///     ("KEY3_APP".to_owned(), "value3".to_owned()),
    /// ];
    ///
    /// let with_postfix: Postfixed = postfixed("_APP");
    /// let custom_struct: CustomStruct = with_postfix.from_iter(vars).unwrap();
    ///
    /// assert_eq!(
    ///     custom_struct,
    ///     CustomStruct {
    ///         key1: "value1".to_owned(),
    ///         key2: "value2".to_owned(),
    ///         key3: Some("value3".to_owned())
    ///     }
    /// )
    /// ```
    pub fn from_iter<T, Iter>(&self, iter: Iter) -> Result<T>
    where
        T: de::DeserializeOwned,
        Iter: IntoIterator<Item = (String, String)>,
    {
        from_iter(iter.into_iter().filter_map(|(k, v)| {
            if k.ends_with(self.0) {
                Some((k.trim_end_matches(self.0).to_owned(), v))
            } else {
                None
            }
        }))
    }

    /// Retrieve the postfix specified at the time
    /// of constructing an instance of [`Postfixed`]
    pub fn postfix(&self) -> &str {
        self.0
    }
}

/// Aids in deserializing some type `T` from environment variables,
/// where the keys are postfixed. Users are meant to obtain a [`Postfixed`]
/// struct by calling [`postfixed`].
///
/// # Example
///
/// ```
/// use renvar::postfixed;
///
/// let with_postfix = postfixed("_APP");
///
/// assert_eq!(with_postfix.postfix(), "_APP")
/// ```
pub fn postfixed(postfix: &str) -> Postfixed<'_> {
    Postfixed(postfix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::env;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct Test {
        key: String,
    }

    #[test]
    fn test_postfixed() {
        env::set_var("KEY_APP", "value");
        let postfixed = postfixed("_APP").from_env::<Test>().unwrap();

        assert_eq!(
            postfixed,
            Test {
                key: String::from("value")
            }
        )
    }
}
