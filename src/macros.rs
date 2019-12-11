#[macro_export]
macro_rules! decode_fixed {
    ($string:expr, $size:expr) => {
        match base64::decode_config($string, base64::URL_SAFE) {
            Ok(bytes_vec) => {
                if bytes_vec.len() == $size {
                    let mut bytes: [u8; $size] = [0; $size];
                    bytes.copy_from_slice(bytes_vec.as_slice());
                    Ok(bytes)
                } else {
                    Err("Invalid byte size")
                }
            }
            _ => Err("Invalid base64"),
        }
    };
}

#[macro_export]
macro_rules! serde_fixed {
    ($name:ident, $schema:tt, $size:expr) => {
        enum $name {}
        impl $name {
            #[allow(dead_code)]
            pub fn serialize<S>(bytes: &[u8; $size], serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                serializer
                    .serialize_str(base64::encode_config(&bytes[..], base64::URL_SAFE).as_str())
            }

            #[allow(dead_code)]
            pub fn deserialize<'de, D>(
                deserializer: D,
            ) -> ::std::result::Result<[u8; $size], D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                struct Base64Visitor;

                impl<'de> serde::de::Visitor<'de> for Base64Visitor {
                    type Value = [u8; $size];

                    fn expecting(
                        &self,
                        formatter: &mut ::std::fmt::Formatter,
                    ) -> ::std::fmt::Result {
                        write!(formatter, "base64 ASCII text")
                    }

                    fn visit_str<E>(self, v: &str) -> ::std::result::Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        super::decode_fixed!(v, $size).map_err(serde::de::Error::custom)
                    }
                }

                deserializer.deserialize_str(Base64Visitor)
            }
        }

        impl rocket_okapi::JsonSchema for $name {
            fn schema_name() -> String {
                String::from($schema)
            }

            fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
                // This calculates the lenghts for base64 encoded strings
                // where minimum length is without padding and maximum is with padding.
                // We accept both padded and unpadded base 64
                let min_length = (($size as f32) * 4.0 / 3.0).ceil() as u32;
                let max_length = ((($size as f32) / 3.0).ceil() * 4.0) as u32;

                // This is just the regex for padded base64url encoded strings with optional padding
                let pattern = format!(
                    "^[A-Za-z0-9_\\-]{{{}}}={{0,{}}}$",
                    min_length,
                    max_length - min_length
                );

                let mut schema = schemars::schema_for!(String).schema;
                schema.metadata = None;
                schema.format = Some(String::from("base64url"));
                schema.string = Some(Box::new(schemars::schema::StringValidation {
                    max_length: Some(max_length),
                    min_length: Some(min_length),
                    pattern: Some(pattern),
                }));
                schemars::schema::Schema::Object(schema)
            }
        }
    };
}
