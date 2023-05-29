use std::env;

use serde::de::{self};

use crate::{de::EnvVarDeserializer, sanitize::is_quote_or_whitespace, Error, Result};

/// Construct a `T` from an [`str`]
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

/// Deserialize a `T` from an iterator
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

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Deserialize a `T` from environment variables
pub fn from_env<T>() -> Result<T>
where
    T: de::DeserializeOwned,
{
    from_iter(env::vars())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Deserialize a `T` from environment variables
///
/// The function will check whether the environment variables contain
/// valid unicode and as such, uses [`std::env::vars_os`] to avoid panics.
///
/// For a panicky alternative use [`from_env`]
pub fn from_os_env<T>() -> Result<T>
where
    T: de::DeserializeOwned,
{
    T::deserialize(EnvVarDeserializer::new(maybe_invalid_unicode_vars_os()?))
}

/// Return an iterator of `(String, String)` from [`std::env::vars_os`]
///
/// This function will error if the env vars *don't* contain valid Unicode
pub(crate) fn maybe_invalid_unicode_vars_os(
) -> Result<impl Iterator<Item = (String, String)>> {
    let vars = env::vars_os()
        .map(|(k, v)| (k.into_string(), v.into_string()))
        .collect::<Vec<_>>();

    for (key, value) in vars.iter() {
        if let Err(key_error) = key {
            return Err(Error::Custom(format!(
                "key of env var contains invalid unicode: {}",
                key_error.to_string_lossy()
            )));
        }

        if let Err(value_error) = value {
            return Err(Error::Custom(format!(
                "value of env var contains invalid unicode: {}",
                value_error.to_string_lossy()
            )));
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
