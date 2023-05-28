#[macro_export]
macro_rules! forward_parsed_values {
    ($($typ:ident => $method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value>
                where V: de::Visitor<'de>
            {
                match self.0.parse::<$typ>() {
                    Ok(val) => val.into_deserializer().$method(visitor),
                    Err(e) => Err(de::Error::custom(format_args!("{} while parsing value '{}'", e, self.0)))
                }
            }
        )*
    }
}
