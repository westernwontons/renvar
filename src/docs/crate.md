`renvar` is library to help deserialize environment variables into Rust data structure

# Example

```rust
use renvar::{from_env, from_iter, from_str};
use serde::Deserialize;
use std::env;

let env_content = r#"
name=renvar
type=Library
dependencies=serde
"#;

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum CrateType {
    Library,
    Binary,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Renvar {
    name: String,
    #[serde(rename = "type")]
    typ: CrateType,
    dependencies: Vec<String>,
}

let actual = Renvar {
    name: "renvar".to_owned(),
    typ: CrateType::Library,
    dependencies: vec!["serde".to_owned()],
};

// we can read from strings

let value = from_str::<Renvar>(env_content).unwrap();

assert_eq!(value, actual);

// directly from the environment

let envs = vec![
    ("name".to_owned(), "renvar".to_owned()),
    ("type".to_owned(), "Library".to_owned()),
    ("dependencies".to_owned(), "serde".to_owned()),
];

for (key, value) in envs.clone().into_iter() {
    env::set_var(key, value);
}

let value = from_env::<Renvar>().unwrap();

assert_eq!(value, actual);

// or from iterables

let value = from_iter::<Renvar, _>(envs).unwrap();

assert_eq!(value, actual);
```

# Feature flags

Renvar has the following feature flags:

## prefixed

`prefixed` gives you the `prefixed` function, that accepts a prefix. The prefixes will be stripped away
before deserialization.

## postfixed

`postfix` is exactly the same as prefix, just with postfixes

# case_insensitive_prefixed

Case insensitive variant of `prefixed`

# case_insensitive_postfixed

Case insensitive variant of `postfixed`

## with_trimmer

Finally, the `with_trimmer` feature flag gives you `*_with_trimmer` variants for all of the above,
where you can strip extraneous characters off of the beginning and end of envrironment varaibles
by passing a closure.

# Supported datatypes

- `Strings` and `str`s
- `enums`
- `sequences`
- `Unit structs`
