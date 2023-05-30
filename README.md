# TodoList

add more tests

custom delimiters for sequences

cacheable env vars

encrypted env vars

deserialize aws parameter store (using rusoto) credentials

custom syntax to support deserialization into structs to allow nested
env vars
example:

```
key=struct_name{field: value, field: value, field: struct_name{field: value, field: value}, field: enum[A,B,C]}
```

It should be as close to Rust syntax as possible.

Structs would be: `struct_name{ .. fields .. }`

Unit structs could be: `struct_name(field)`

Enums would be: `enum[A,B,C]`

Enums with newtype fields could be: `enum[A(field), B(field), C(field)]`

Enum with struct variants could be: enum[A{field: value}, B{field: value}, C{field: value}]

and a mix of all of the above.

Where key is the env var key, name is the name of the struct
and the rest are `field: value` pairs.
