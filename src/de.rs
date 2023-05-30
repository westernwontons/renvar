//! Module that provides an `EnvVarDeserializer`, that does exactly as it says:
//! deserializes environment variables into Rust structs
//!
//! Most of the work is based on [envy](https://docs.rs/envy/0.4.2/envy/), but
//! with some slight differences.
//!
//! Do note that even though this module is public, users should prefer to use
//! the public functions exposed by this crate,
//! such as [`crate::from_iter`], [`crate::from_env`] or [`crate::from_str`]
//!
//! If those functions don't cut it, you can use the [`EnvVarDeserializer`] directly.
//!
//! # Example
//!
//! ```
//! use renvar::de::EnvVarDeserializer;
//! use serde::Deserialize;
//! use std::env;
//!
//! #[derive(Deserialize, Debug, PartialEq, Eq)]
//! struct CustomStruct {
//!     field: String,
//! }
//!
//! let iter = vec![(String::from("field"), String::from("value"))];
//!
//! let de = EnvVarDeserializer::new(iter.into_iter());
//!
//! let custom_struct = CustomStruct::deserialize(de).unwrap();
//!
//! assert_eq!(
//!     custom_struct,
//!     CustomStruct {
//!         field: String::from("value")
//!     }
//! );
//!
//! // or from actual env vars, not simulated ones
//!
//! let iter = vec![(String::from("field"), String::from("value"))];
//!
//! for (key, value) in iter.into_iter() {
//!     env::set_var(key, value);
//! }
//!
//! let de = EnvVarDeserializer::new(env::vars());
//!
//! let custom_struct = CustomStruct::deserialize(de).unwrap();
//!
//! assert_eq!(
//!     custom_struct,
//!     CustomStruct {
//!         field: String::from("value")
//!     }
//! )
//! ```

use std::iter::empty;

use serde::de::value::{MapDeserializer, SeqDeserializer};
use serde::{
    de::{self, IntoDeserializer},
    Deserialize,
};

use crate::{forward_parsed_values, sanitize::is_quote_or_whitespace, Error, Result};

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents the value of an environment variable
///
/// In other words, everything *after* `=`
#[derive(Debug)]
struct EnvVarValue(String);

impl<'de> de::IntoDeserializer<'de, Error> for EnvVarValue {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for EnvVarValue {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.0
            .into_deserializer()
            .deserialize_any(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.0.is_empty() || self.0.chars().all(is_quote_or_whitespace) {
            SeqDeserializer::new(empty::<Self>()).deserialize_seq(visitor)
        } else {
            let values = self.0.split(',').map(|value| {
                Self(
                    value
                        .trim_matches(is_quote_or_whitespace)
                        .to_owned(),
                )
            });
            SeqDeserializer::new(values).deserialize_seq(visitor)
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let _ = name;
        let _ = variants;

        visitor.visit_enum(self.0.into_deserializer())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.0.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self.0.into_deserializer())
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let _ = name;

        visitor.visit_newtype_struct(self)
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        String::deserialize(self.0.into_deserializer()).and_then(|unit_name| {
            if unit_name == name {
                visitor.visit_unit()
            } else {
                Err(Error::Custom(format!(
                    "expected unit struct with name '{}', found '{}'",
                    name, unit_name
                )))
            }
        })
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_parsed_values! {
        bool => deserialize_bool,
        u8 => deserialize_u8,
        u16 => deserialize_u16,
        u32 => deserialize_u32,
        u64 => deserialize_u64,
        i8 => deserialize_i8,
        i16 => deserialize_i16,
        i32 => deserialize_i32,
        i64 => deserialize_i64,
        f32 => deserialize_f32,
        f64 => deserialize_f64,
    }

    serde::forward_to_deserialize_any! {
        char str string bytes byte_buf
        map tuple tuple_struct
        struct identifier ignored_any
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// An iterator over environment variables of `(key, value)` pairs
///
/// Note: Calling [`Iterator::next`] will lowercase all keys
/// before returning them
#[derive(Debug)]
struct EnvVars<Iter>(Iter)
where
    Iter: IntoIterator<Item = (String, String)>;

impl<Iter> Iterator for EnvVars<Iter>
where
    Iter: Iterator<Item = (String, String)>,
{
    type Item = (String, EnvVarValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|(key, value)| (key.to_lowercase(), EnvVarValue(value)))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Deserializer for environment variables
///
/// Can be constructred from a type that implements [`Iterator`]
/// over `(String, String)` tuples
///
/// Alternatively, can be constructed from a [`str`] using [`crate::from_str`]
#[derive(Debug)]
pub struct EnvVarDeserializer<'de, Iter>
where
    Iter: Iterator<Item = (String, String)>,
{
    inner: MapDeserializer<'de, EnvVars<Iter>, Error>,
}

impl<'de, Iter> EnvVarDeserializer<'de, Iter>
where
    Iter: Iterator<Item = (String, String)>,
{
    /// Construct an [`EnvVarDeserializer`] from an [`Iterator`] over tuples of [`String`]s
    pub fn new(iter: Iter) -> Self {
        Self {
            inner: MapDeserializer::new(EnvVars(iter)),
        }
    }
}

impl<'de, Iter> de::Deserializer<'de> for EnvVarDeserializer<'de, Iter>
where
    Iter: Iterator<Item = (String, String)>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self.inner)
    }

    serde::forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        bytes byte_buf unit_struct tuple_struct
        identifier tuple ignored_any option newtype_struct enum
        struct
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::from_iter;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Unit;

    #[derive(Debug, Deserialize, PartialEq)]
    struct NewType(u64);

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
    }

    #[test]
    fn test_from_iter() {
        let iter = vec![
            (String::from("string_field"), String::from("hello")),
            (String::from("empty_string_field"), String::from("")),
            (String::from("sequence"), String::from("first,second,third")),
            (
                String::from("empty_sequence_doublequote"),
                String::from("\"\""),
            ),
            (
                String::from("empty_sequence_singlequote"),
                String::from("\'\'"),
            ),
            (String::from("empty_sequence_whitespace"), String::from(" ")),
            (String::from("unit"), String::from("Unit")),
            (String::from("newtype"), String::from("62875")),
            (String::from("optional_field"), String::from("")),
        ];

        let test_struct = from_iter::<Test, _>(iter.into_iter()).unwrap();

        assert_eq!(
            test_struct,
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
                newtype: NewType(62875),
                optional_field: None
            }
        );
    }
}
