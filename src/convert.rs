use std::env;

use serde::de::{self};

use crate::{de::EnvVarDeserializer, sanitize::is_quote_or_whitespace, Error, Result};

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Deserialize some type `T` from a [`str`]
///
/// The `(key, value)` pairs will have the following [`char`]s stripped
/// from the beginning and end of the strings:
///
/// * ' (single quote)
/// * " (double quote)
/// * \s  (whitespace)
///
/// [`from_str`] expects a blob of str with newline `(\n)` or
/// carriage return newline `(\r\n)` delimited lines,
/// where the key value pairs can look like any of the following:
///
/// ```text
/// key=value
/// KEY=value
/// "KEY"=VALUE
/// KEY="VALUE"
/// KEY="   VALUE     "
/// KEY='VALUE'
/// ```
///
/// Note that the values will **not** be lowercased, but **will** be trimmed,
/// removing the afformentioned prefixes and suffixes. Another thing to note is that
/// if you define a [`String`] in your `struct`, but the input is `key=`, then
/// your result will be an empty [`String`]. This means an allocation, so unless
/// you want this behaviour, you're encouraged to instead define it as an `Option<String>`
///
/// # Example
///
/// ```
/// use renvar::from_str;
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize, PartialEq, Eq)]
/// struct CustomStruct {
///     key: String,
///     maybe: Option<String>,
/// }
///
/// let input = r#"
/// key="I'm a VALUE"
/// maybe=
/// "#;
///
/// let custom_struct = from_str::<CustomStruct>(input).unwrap();
///
/// assert_eq!(
///     custom_struct,
///     CustomStruct {
///         key: "I'm a VALUE".to_owned(),
///         maybe: None
///     }
/// );
///
/// // With `maybe` being a `String`:
///
/// #[derive(Debug, Deserialize, PartialEq, Eq)]
/// struct AnotherCustomStruct {
///     key: String,
///     maybe: String,
///     something_else: String,
/// }
///
/// let input = r#"
/// key="I'm a VALUE"
/// maybe=
/// something_else=
/// "#;
///
/// let custom_struct = from_str::<AnotherCustomStruct>(input).unwrap();
///
/// assert_eq!(
///     custom_struct,
///     AnotherCustomStruct {
///         key: "I'm a VALUE".to_owned(),
///         maybe: "".to_owned(),
///         something_else: "".to_owned()
///     }
/// );
/// ```
pub fn from_str<'de, T>(input: &str) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    let iter = input
        .lines()
        .filter_map(|line| {
            line.split_once('=').map(|(key, value)| {
                (
                    String::from(key.trim_matches(is_quote_or_whitespace)),
                    String::from(value.trim_matches(is_quote_or_whitespace)),
                )
            })
        })
        .collect::<Vec<_>>();

    T::deserialize(EnvVarDeserializer::new(iter.into_iter()))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Deserialize some type `T` from an iterator of key-value pairs
///
/// Like with [`from_str`], single quotes, double quotes and whitespace will be trimmed
///
/// # Example
///
/// ```
/// use renvar::from_iter;
/// use renvar::Error;
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize, PartialEq, Eq)]
/// struct CustomStruct {
///     key1: String,
///     key2: String,
/// }
///
/// let vars = vec![
///     ("KEY1".to_owned(), "value1  ".to_owned()),
///     ("KEY2".to_owned(), "value2".to_owned()),
/// ];
///
/// let custom_struct: CustomStruct = from_iter(vars).unwrap();
///
/// assert_eq!(
///     custom_struct,
///     CustomStruct {
///         key1: "value1".to_owned(),
///         key2: "value2".to_owned()
///     }
/// )
/// ```
pub fn from_iter<T, Iter>(iter: Iter) -> Result<T>
where
    Iter: IntoIterator<Item = (String, String)>,
    T: de::DeserializeOwned,
{
    T::deserialize(EnvVarDeserializer::new(iter.into_iter().map(
        |(key, value)| {
            (
                String::from(key.trim_matches(is_quote_or_whitespace)),
                String::from(value.trim_matches(is_quote_or_whitespace)),
            )
        },
    )))
}

#[cfg(feature = "with_trimmer")]
pub mod with_trimmer {

    use serde::de;
    use std::env;

    use crate::{de::EnvVarDeserializer, Result};

    use super::maybe_invalid_unicode_vars_os;

    // todo: replace Fn with Pattern once it's stable
    //
    /// Deserialize some type `T` from an iterator over `(String, String)`
    /// `(key, value)` pairs, where you can specify custom trimming logic
    /// with a closure.
    ///
    /// The environment variable values might have some unneeded prefix or suffixes.
    /// If this is the case, users are encouraged to use this function, which allows
    /// passing a closure that receives a [`char`] and returns a [`bool`].
    ///
    /// For each `char`, returning `true` will have it removed.
    ///
    /// # Panics
    /// If the environment variable contains invalid unicode. If you'd like to avoid this,
    /// use [`from_os_env_with_trimmer`]
    ///
    /// # Example
    ///
    /// ```
    /// use renvar::from_iter_with_trimmer;
    /// use serde::Deserialize;
    ///
    /// #[derive(Debug, Deserialize, PartialEq, Eq)]
    /// struct CustomStruct {
    ///     key: String,
    /// }
    ///
    /// let iter = vec![(String::from("key"), String::from("value   ;"))];
    ///
    /// let trimmer = |c: char| c == ';' || c == ' ';
    ///
    /// let custom_struct: CustomStruct = from_iter_with_trimmer(iter, trimmer).unwrap();
    ///
    /// assert_eq!(
    ///     custom_struct,
    ///     CustomStruct {
    ///         key: String::from("value")
    ///     }
    /// )
    /// ```
    pub fn from_iter_with_trimmer<T, Iter, Trimmer>(
        iter: Iter,
        trimmer: Trimmer,
    ) -> Result<T>
    where
        Iter: IntoIterator<Item = (String, String)>,
        T: de::DeserializeOwned,
        Trimmer: Fn(char) -> bool + Copy,
    {
        T::deserialize(EnvVarDeserializer::new(iter.into_iter().map(
            |(key, value)| {
                (
                    String::from(key.trim_matches(trimmer)),
                    String::from(value.trim_matches(trimmer)),
                )
            },
        )))
    }

    // todo: replace Fn with Pattern once it's stable
    //
    /// Deserialize some type `T` from a snapshot of the processes environment variables
    /// at the time of invocation, where you can specify custom trimming logic
    /// with a closure.
    ///
    /// The environment variable values might have some unneeded prefix or suffixes.
    /// If this is the case, users are encouraged to use this function, which allows
    /// passing a closure that receives a [`char`] and returns a [`bool`].
    ///
    /// For each `char`, returning `true` will have it removed.
    ///
    /// # Panics
    ///
    /// If the environment variable contains invalid unicode.
    /// If you'd like to avoid this, use [`crate::from_os_env_with_trimmer`]
    ///
    /// # Example
    ///
    /// ```
    /// use renvar::from_env_with_trimmer;
    /// use serde::Deserialize;
    /// use std::env;
    ///
    /// #[derive(Debug, Deserialize, PartialEq, Eq)]
    /// struct CustomStruct {
    ///     key: String,
    /// }
    ///
    /// let envs = vec![(String::from("key"), String::from("value. .."))];
    ///
    /// for (key, value) in envs.into_iter() {
    ///     env::set_var(key, value);
    /// }
    ///
    /// let custom_struct: CustomStruct =
    ///     from_env_with_trimmer(|c: char| c == ' ' || c == '.').unwrap();
    ///
    /// assert_eq!(
    ///     custom_struct,
    ///     CustomStruct {
    ///         key: String::from("value")
    ///     }
    /// );
    /// ```
    pub fn from_env_with_trimmer<T, Trimmer>(trimmer: Trimmer) -> Result<T>
    where
        T: de::DeserializeOwned,
        Trimmer: Fn(char) -> bool + Copy,
    {
        from_iter_with_trimmer(env::vars(), trimmer)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////

    /// Deserialize some type `T` from a snapshot of the processes environment variables
    /// at the time of invocation.
    ///
    /// The function will check whether the environment variables contain
    /// valid unicode and as such, uses [`std::env::vars_os`] to avoid panics.
    ///
    /// The environment variable values might have some unneeded prefix or suffixes.
    /// If this is the case, users are encouraged to use this function, which allows
    /// passing a closure that receives a [`char`] and returns a [`bool`].
    ///
    /// Items for which the closure returns `true` will be trimmed from keys and values of the
    /// environment variables.
    ///
    /// For a panicky alternative, use [`crate::from_env`] or [`crate::from_env_with_trimmer`]
    ///
    /// # Example
    ///
    /// ```
    /// use renvar::from_os_env_with_trimmer;
    /// use serde::Deserialize;
    /// use std::env;
    ///
    /// #[derive(Debug, Deserialize, PartialEq, Eq)]
    /// struct CustomStruct {
    ///     key: String,
    /// }
    ///
    /// let envs = vec![("key".to_owned(), "value. ..".to_owned())];
    ///
    /// for (key, value) in envs.into_iter() {
    ///     env::set_var(key, value);
    /// }
    ///
    /// let custom_struct: CustomStruct =
    ///     from_os_env_with_trimmer(|c: char| c == ' ' || c == '.').unwrap();
    ///
    /// assert_eq!(
    ///     custom_struct,
    ///     CustomStruct {
    ///         key: "value".to_owned()
    ///     }
    /// );
    /// ```
    pub fn from_os_env_with_trimmer<T, Trimmer>(trimmer: Trimmer) -> Result<T>
    where
        T: de::DeserializeOwned,
        Trimmer: Fn(char) -> bool + Copy,
    {
        T::deserialize(EnvVarDeserializer::new(
            maybe_invalid_unicode_vars_os()?.map(|(key, value)| {
                (
                    String::from(key.trim_matches(trimmer)),
                    String::from(value.trim_matches(trimmer)),
                )
            }),
        ))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Deserialize some type `T` from a snapshot of the processes environment variables
/// at the time of invocation.
///
/// The environment variable values might have some unneeded prefix or suffixes.
/// If this is the case, users are encouraged to use this function, which allows
/// passing a closure that receives a [`char`] and returns a [`bool`].
///
/// Items for which the closure returns `true` will be trimmed from keys and values of the
/// environment variables.
///
/// Note that if the environment variables contain potentionally invalid unicode, this function will panic.
///
/// For a non-panicky alternative, use [`crate::from_os_env`] or [`crate::from_os_env_with_trimmer`]
///
/// ```
/// use renvar::from_env;
/// use serde::Deserialize;
/// use std::env;
///
/// #[derive(Debug, Deserialize, PartialEq, Eq)]
/// struct CustomStruct {
///     key: String,
/// }
///
/// let envs = vec![("key".to_owned(), "value".to_owned())];
///
/// for (key, value) in envs.into_iter() {
///     env::set_var(key, value);
/// }
///
/// let custom_struct: CustomStruct = from_env().unwrap();
///
/// assert_eq!(
///     custom_struct,
///     CustomStruct {
///         key: "value".to_owned()
///     }
/// );
/// ```
pub fn from_env<T>() -> Result<T>
where
    T: de::DeserializeOwned,
{
    from_iter(env::vars())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Deserialize some type `T` from a snapshot of the processes environment variables
/// at the time of invocation.
///
/// The function will check whether the environment variables contain
/// valid unicode and as such, uses [`std::env::vars_os`] to avoid panics.
///
/// For a panicky alternative, use [`crate::from_env`] or [`crate::from_env_with_trimmer`],
///
/// Note: [`crate::from_env_with_trimmer`] is behind the `with_trimmer` feature flag
///
/// ```
/// use renvar::from_os_env;
/// use serde::Deserialize;
/// use std::env;
///
/// #[derive(Debug, Deserialize, PartialEq, Eq)]
/// struct CustomStruct {
///     key: String,
/// }
///
/// let envs = vec![("key".to_owned(), "value".to_owned())];
///
/// for (key, value) in envs.into_iter() {
///     env::set_var(key, value);
/// }
///
/// let custom_struct: CustomStruct = from_os_env().unwrap();
///
/// assert_eq!(
///     custom_struct,
///     CustomStruct {
///         key: "value".to_owned()
///     }
/// );
/// ```
pub fn from_os_env<T>() -> Result<T>
where
    T: de::DeserializeOwned,
{
    T::deserialize(EnvVarDeserializer::new(maybe_invalid_unicode_vars_os()?))
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Return an iterator of `(String, String)` from [`std::env::vars_os`]
///
/// This function will error if the env vars contain invalid Unicode
pub(crate) fn maybe_invalid_unicode_vars_os(
) -> Result<impl Iterator<Item = (String, String)>> {
    let vars = env::vars_os()
        .map(|(k, v)| (k.into_string(), v.into_string()))
        .collect::<Vec<_>>();

    // we don't expect a lot of errors to happen, so it's better to just clone,
    // instead of putting a lifetime bound with a Cow or OsStr on renvar::Error
    for (key, value) in vars.iter() {
        if let Err(key_error) = key {
            return Err(Error::InvalidUnicode(key_error.to_owned()));
        }

        if let Err(value_error) = value {
            return Err(Error::InvalidUnicode(value_error.to_owned()));
        }
    }

    Ok(vars
        .into_iter()
        .map(|(k, v)| (k.unwrap(), v.unwrap())))
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Unit;

    #[derive(Debug, Deserialize, PartialEq)]
    struct NewType(u64);

    #[derive(Debug, Deserialize, PartialEq)]
    enum Enumeration {
        A,
        B,
        C,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Test {
        string_field: String,
        empty_string_field: String,
        sequence: Vec<String>,
        empty_sequence_doublequote: Vec<String>,
        empty_sequence_singlequote: Vec<String>,
        empty_sequence_whitespace: Vec<String>,
        unit: Unit,
        newtype: NewType,
        optional_field: Option<String>,
        enumeration: Enumeration,
    }

    #[test]
    fn test_from_str() {
        let input_str = r#"
        string_field=hello
        empty_string_field=
        sequence=first,second,third
        empty_sequence_doublequote=""
        empty_sequence_singlequote=''
        empty_sequence_whitespace=" "
        unit=Unit
        newtype=123
        optional_field=
        enumeration=A
        "#;

        let actual = from_str::<Test>(input_str).unwrap();

        assert_eq!(
            actual,
            Test {
                string_field: String::from("hello"),
                empty_string_field: String::from(""),
                sequence: vec![
                    String::from("first"),
                    String::from("second"),
                    String::from("third")
                ],
                empty_sequence_doublequote: vec![],
                empty_sequence_singlequote: vec![],
                empty_sequence_whitespace: vec![],
                unit: Unit,
                newtype: NewType(123),
                optional_field: None,
                enumeration: Enumeration::A,
            }
        )
    }

    #[test]
    fn test_from_str_with_extra_quotes() {
        let input_str = r#"
        string_field="hello"
        empty_string_field=
        sequence="first,second,third"
        empty_sequence_doublequote=""
        empty_sequence_singlequote=''
        empty_sequence_whitespace=" "
        unit=Unit
        newtype=123
        optional_field=
        enumeration=A
        "#;

        let actual = from_str::<Test>(input_str).unwrap();

        assert_eq!(
            actual,
            Test {
                string_field: String::from("hello"),
                empty_string_field: String::from(""),
                sequence: vec![
                    String::from("first"),
                    String::from("second"),
                    String::from("third")
                ],
                empty_sequence_doublequote: vec![],
                empty_sequence_singlequote: vec![],
                empty_sequence_whitespace: vec![],
                unit: Unit,
                newtype: NewType(123),
                optional_field: None,
                enumeration: Enumeration::A,
            }
        )
    }

    #[test]
    fn test_from_env() {
        let input_str = r#"
        string_field=hello
        empty_string_field=""
        sequence=first,second,third
        empty_sequence_doublequote=""
        empty_sequence_singlequote=''
        empty_sequence_whitespace=" "
        unit=Unit
        newtype=123
        optional_field=""
        enumeration=A
        "#;

        for line in input_str.lines().filter_map(|l| {
            let line = l.trim();
            if !line.is_empty() {
                Some(line)
            } else {
                None
            }
        }) {
            let (key, value) = line.split_once('=').unwrap();
            env::set_var(key, value);
        }

        let actual = from_env::<Test>().unwrap();

        assert_eq!(
            actual,
            Test {
                string_field: String::from("hello"),
                empty_string_field: String::from(""),
                sequence: vec![
                    String::from("first"),
                    String::from("second"),
                    String::from("third")
                ],
                empty_sequence_doublequote: vec![],
                empty_sequence_singlequote: vec![],
                empty_sequence_whitespace: vec![],
                unit: Unit,
                newtype: NewType(123),
                optional_field: None,
                enumeration: Enumeration::A,
            }
        );
    }
}
