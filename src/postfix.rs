#[cfg(feature = "postfixed")]
pub mod postfixed {
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
    /// // Note that `postfixed` is behind the `postfixed` feature flag
    /// ```
    #[derive(Debug)]
    pub struct Postfixed<'a>(&'a str);

    impl<'a> Postfixed<'a> {
        /// Deserialize some type `T` from a snapshot of the currently
        /// running process's environment variables at invocation time.
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
        ///
        /// // Note that `postfixed` is behind the `postfixed` feature flag
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
        ///
        /// let envs = vec![("KEY_APP".to_owned(), "value".to_owned()))];
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
        /// );
        ///
        /// // Note that `postfixed` is behind the `postfixed` feature flag
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
        /// # Example
        ///
        /// ```
        /// use renvar::{postfixed, Postfixed};
        ///
        /// #[derive(Debug, Deserialize, PartialEq, Eq)]
        /// struct CustomStruct {
        ///     key1: String,
        ///     key2: String,
        ///     key3: Option<String>,
        /// }
        ///
        /// let vars = vec![
        ///     ("KEY1_SUFFIX".to_owned(), "value1".to_owned()),
        ///     ("KEY2_SOME_SUFFIX".to_owned(), "value2".to_owned()),
        ///     ("KEY3_SUFFIX".to_owned(), "value3".to_owned()),
        /// ];
        ///
        /// let with_postfix: Postfixed = postfixed("_SUFFIX");
        /// let custom_struct: CustomStruct = with_postfix.from_iter(vars).unwrap();
        ///
        /// assert_eq!(
        ///     custom_struct,
        ///     CustomStruct {
        ///         key1: "value1".to_owned(),
        ///         key2: "value2".to_owned(),
        ///         key3: "value3".to_owned()
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
    /// // Note that `postfixed` is behind the `postfixed` feature flag
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
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "case_insensitive_postfixed")]
pub mod case_insensitive_postfixed {

    use crate::{convert::maybe_invalid_unicode_vars_os, from_iter, Result};
    use serde::de;
    use std::env;

    /// Represents a case-insensitive postfix used to filter environment variables during deserialization.
    ///
    /// It is used to specify a case-insensitive postfix that is used to filter environment variables during deserialization.
    /// To create an instance of [`CaseInsensitivePostfixed`], you can use the [`case_insensitive_postfixed`] function:
    ///
    /// # Example
    ///
    /// ```
    /// // Creates a new instance of `CaseInsensitivePostfixed` with the specified case-insensitive postfix.
    ///
    ///
    /// use renvar::{case_insensitive_postfixed, CaseInsensitivePostfixed};
    ///
    /// let with_postfix: CaseInsensitivePostfixed = case_insensitive_postfixed("_suffix");
    /// // or
    /// let with_postfix = case_insensitive_postfixed("_SUFFIX");
    /// // or
    /// // (please don't do this)
    /// let with_postfix = case_insensitive_postfixed("_sUfFiX");
    /// // but since it's case insensitive, it doesn't matter, as long as it's valid unicode
    /// ```
    #[derive(Debug)]
    pub struct CaseInsensitivePostfixed<'a>(&'a str);

    impl<'a> CaseInsensitivePostfixed<'a> {
        /// Deserialize some type `T` from a snapshot of environment
        /// variables, filtering only the variables that end with the
        /// specified postfix.
        ///
        /// # Example
        ///
        /// ```
        /// use renvar::Error;
        /// use renvar::{case_insensitive_postfixed, CaseInsensitivePostfixed};
        /// use serde::Deserialize;
        /// use std::env;
        ///
        /// #[derive(Deserialize, Debug, PartialEq, Eq)]
        /// struct CustomStruct {
        ///     field: String,
        ///     other_field: Option<String>,
        /// }
        ///
        /// let envs = vec![
        ///     ("FIELD_SUFFix".to_owned(), "value".to_owned()),
        ///     ("OTHER_FIELD_SUFFIX".to_owned(), "other_value".to_owned()),
        /// ];
        ///
        /// for (key, value) in envs.into_iter() {
        ///     env::set_var(key, value);
        /// }
        ///
        /// let with_postfix: CaseInsensitivePostfixed = case_insensitive_postfixed("_SUFFIX");
        /// let custom_struct: CustomStruct = with_postfix.from_env().unwrap();
        ///
        /// assert_eq!(
        ///     custom_struct,
        ///     CustomStruct {
        ///         field: "value".to_owned(),
        ///         other_field: Some("other_value".to_owned())
        ///     }
        /// )
        /// ```
        pub fn from_env<T>(&self) -> Result<T>
        where
            T: de::DeserializeOwned,
        {
            self.from_iter(env::vars())
        }

        /// Deserialize some type `T` from a snapshot of environment variables,
        /// filtering only the variables that end with the specified postfix.
        /// This method handles environment variables with potentially invalid Unicode.
        ///
        /// # Example
        ///
        /// ```
        /// use renvar::Error;
        /// use renvar::{case_insensitive_postfixed, CaseInsensitivePostfixed};
        /// use serde::Deserialize;
        /// use std::env;
        ///
        /// #[derive(Deserialize, Debug, PartialEq)]
        /// struct CustomStruct {
        ///     field: String,
        ///     other_field: Option<String>,
        /// }
        ///
        /// let envs = vec![
        ///     ("field_suffix".to_owned(), "field_value".to_owned()),
        ///     ("other_field_suffix".to_owned(), "other_value".to_owned()),
        /// ];
        ///
        /// for (key, value) in envs.into_iter() {
        ///     env::set_var(key, value);
        /// }
        ///
        /// let with_postfix: CaseInsensitivePostfixed = case_insensitive_postfixed("_SUFFIX");
        /// let custom_struct: CustomStruct = with_postfix.from_os_env().unwrap();
        ///
        /// assert_eq!(
        ///     custom_struct,
        ///     CustomStruct {
        ///         field: "field_value".to_owned(),
        ///         other_field: Some("other_value".to_owned())
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
        /// filtering only the pairs where the key ends with the specified postfix.
        ///
        /// # Example
        ///
        /// ```
        /// use renvar::{case_insensitive_postfixed, CaseInsensitivePostfixed};
        /// use serde::Deserialize;
        ///
        /// #[derive(Debug, Deserialize, PartialEq, Eq)]
        /// struct CustomStruct {
        ///     key1: String,
        ///     key2: String,
        ///     key3: String,
        /// }
        ///
        /// let with_postfix: CaseInsensitivePostfixed = case_insensitive_postfixed("_SUFfix");
        /// let vars = vec![
        ///     ("KEY1_SUFFiX".to_owned(), "value1".to_owned()),
        ///     ("KEY2_SUffIX".to_owned(), "value2".to_owned()),
        ///     ("KEY3_suFFIX".to_owned(), "value3".to_owned()),
        /// ];
        ///
        /// let custom_struct: CustomStruct = with_postfix.from_iter(vars).unwrap();
        ///
        /// assert_eq!(
        ///     custom_struct,
        ///     CustomStruct {
        ///         key1: "value1".to_owned(),
        ///         key2: "value2".to_owned(),
        ///         key3: "value3".to_owned(),
        ///     }
        /// )
        /// ```
        pub fn from_iter<T, Iter>(&self, iter: Iter) -> Result<T>
        where
            T: de::DeserializeOwned,
            Iter: IntoIterator<Item = (String, String)>,
        {
            from_iter(iter.into_iter().filter_map(|(key, value)| {
                let (lowercase_postfix, lowercase_key) =
                    (self.0.to_lowercase(), key.to_lowercase());

                if lowercase_key.ends_with(&lowercase_postfix) {
                    Some((
                        lowercase_key
                            .trim_end_matches(&lowercase_postfix)
                            .to_owned(),
                        value,
                    ))
                } else {
                    None
                }
            }))
        }

        /// Retrieve the postfix specified at the time
        /// of constructing an instance of [`CaseInsensitivePostfixed`]
        pub fn postfix(&self) -> &str {
            self.0
        }
    }

    /// Aids in deserializing some type `T` from environment variables,
    /// where the keys are postfixed. Users are meant to obtain a [`CaseInsensitivePostfixed`]
    /// struct by calling [`case_insensitive_postfixed`].
    ///
    /// As the name suggests, the casing of the keys for the environment variables
    /// does not matter (everything will be lowercased)
    ///
    /// # Example
    ///
    /// ```
    /// use renvar::{case_insensitive_postfixed, CaseInsensitivePostfixed};
    ///
    /// let with_postfix: CaseInsensitivePostfixed = case_insensitive_postfixed("_app");
    ///
    /// // Note that `case_insensitive_postfixed` is behind the `case_insensitive_postfixed` feature flag
    /// ```
    pub fn case_insensitive_postfixed(postfix: &str) -> CaseInsensitivePostfixed<'_> {
        CaseInsensitivePostfixed(postfix)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use serde::Deserialize;

        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Test {
            key: String,
        }

        #[test]
        fn test_case_insensitive_postfixed() {
            env::set_var("KEY_APP", "value");
            let postfixed = case_insensitive_postfixed("_app")
                .from_env::<Test>()
                .unwrap();

            assert_eq!(
                postfixed,
                Test {
                    key: String::from("value")
                }
            )
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////
