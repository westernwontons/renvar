// todo: remove extra quotes from input string
// todo: split tests up
// todo: write more tests
// todo: divide this file up

#![allow(
    unused_variables,
    unused_mut,
    unused_imports,
    dead_code,
    unused_assignments
)]

use std::iter::{empty, FilterMap};
use std::str::{FromStr, Split};
use std::string::String;
use std::vec::{self, IntoIter};

use serde::de::value::{
    Error as SerdeValueError, MapDeserializer, SeqDeserializer, StringDeserializer,
};
use serde::de::{self, StdError};
use serde::de::{DeserializeOwned, IntoDeserializer};
use serde::{forward_to_deserialize_any, Deserialize, Deserializer};

use crate::{error::Error, forward_parsed_values};

////////////////////////////////////////////////////////////////////////////////////////////////////////

pub type Result<T> = std::result::Result<T, Error>;

////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct EnvVars<Iter>(Iter)
where
    Iter: IntoIterator<Item = (String, String)>;

impl<Iter> Iterator for EnvVars<Iter>
where
    Iter: Iterator<Item = (String, String)>,
{
    type Item = (EnvVarKey, EnvVarValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|(key, value)| (EnvVarKey(key.to_lowercase()), EnvVarValue(value)))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents the value of an environment variable
///
/// In other words, everything *before* `=`
#[derive(Debug)]
struct EnvVarKey(String);

impl<'de> IntoDeserializer<'de, Error> for EnvVarKey {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for EnvVarKey {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0
            .into_deserializer()
            .deserialize_any(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Custom(String::from(
            "Environment variable keys must be present",
        )))
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64
        char str string bytes byte_buf map seq enum
        unit unit_struct tuple tuple_struct newtype_struct
        struct identifier ignored_any
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents the value of an environment variable
///
/// In other words, everything *after* `=`
#[derive(Debug)]
struct EnvVarValue(String);

impl<'de> IntoDeserializer<'de, Error> for EnvVarValue {
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
        if self.0.is_empty() {
            SeqDeserializer::new(empty::<Self>()).deserialize_seq(visitor)
        } else {
            let values = self
                .0
                .split(',')
                .map(|value| Self(value.trim().to_owned()));
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
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let _ = name;

        visitor.visit_newtype_struct(self)
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

    forward_to_deserialize_any! {
        char str string bytes byte_buf map
        unit unit_struct tuple tuple_struct
        struct identifier ignored_any
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Deserializer for environment variables
///
/// Can be constructred from a type that implements `Iterator`
/// over `(String, String)` tuples
///
/// Alternatively, can be constructed from a [`String`] or [`str`] reference
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
    fn from_iter(iter: Iter) -> Self {
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

/// Construct a `T` from an input &[`str`]
pub fn from_str<T>(input: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let iter = input
        .split("\n")
        .filter_map(|line| {
            line.split_once("=")
                .map(|(key, value)| (String::from(key), String::from(value)))
        })
        .collect::<Vec<_>>();

    let deserializer = EnvVarDeserializer::from_iter(iter.into_iter());

    T::deserialize(deserializer)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_from_iter() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Test {
            string_field: String,
            sequence: Vec<String>,
        }

        let iter = vec![
            (String::from("string_field"), String::from("hello")),
            (String::from("sequence"), String::from("first,second,third")),
        ];

        let test_struct =
            Test::deserialize(EnvVarDeserializer::from_iter(iter.into_iter())).unwrap();

        assert_eq!(
            test_struct,
            Test {
                string_field: String::from("hello"),
                sequence: vec![
                    String::from("first"),
                    String::from("second"),
                    String::from("third")
                ]
            }
        );

        let input_str = "string_field=hello\nsequence=first,second,third";

        let test_struct = from_str::<Test>(input_str).unwrap();

        assert_eq!(
            test_struct,
            Test {
                string_field: String::from("hello"),
                sequence: vec![
                    String::from("first"),
                    String::from("second"),
                    String::from("third")
                ]
            }
        )
    }
}
