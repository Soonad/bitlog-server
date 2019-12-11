#[macro_export]
macro_rules! fixed_size_byte_array {
    ($name:ident, $schema:expr, $size:expr) => {
        pub struct $name([u8; $size]);

        impl $name {
            fn as_slice(&self) -> &[u8] {
                &self.0
            }
            fn as_array(&self) -> &[u8; $size] {
                &self.0
            }
        }

        impl std::convert::Into<[u8; $size]> for $name {
            fn into(self) -> [u8; $size] { *self.as_array() }
        }

        impl std::convert::From<[u8; $size]> for $name {
            fn from(bytes: [u8; $size]) -> Self { Self(bytes) }
        }

        impl std::convert::TryFrom<Vec<u8>> for $name {
            type Error = &'static str;
            fn try_from(bytes_vec: Vec<u8>) -> Result<Self, Self::Error> { 
                if bytes_vec.len() == $size {
                    let mut bytes: [u8; $size] = [0; $size];
                    bytes.copy_from_slice(bytes_vec.as_slice());
                    Ok(Self(bytes))
                } else {
                    Err("Invalid byte size")
                }
             }
        }
        
        impl<'r> rocket::request::FromParam<'r> for $name {
            type Error = ();
        
            fn from_param(param: &'r rocket::http::RawStr) -> Result<Self, Self::Error> {
                match base64::decode_config(param.url_decode_lossy().as_str(), base64::URL_SAFE) {
                    Ok(bytes_vec) => { std::convert::TryFrom::try_from(bytes_vec).map_err(|_| ()) }
                    _ => Err(())
                }
            }
        }
        
        impl<'r> rocket_okapi::request::OpenApiFromParam<'r> for $name {
            fn path_parameter(
                _gen: &mut rocket_okapi::gen::OpenApiGenerator,
                name: String,
            ) -> Result<okapi::openapi3::Parameter, rocket_okapi::OpenApiError> {
                Ok(okapi::openapi3::Parameter {
                    name,
                    location: String::from("path"),
                    description: None,
                    required: true,
                    deprecated: false,
                    allow_empty_value: false,
                    extensions: std::collections::BTreeMap::new(),
                    value: okapi::openapi3::ParameterValue::Schema {
                        style: None,
                        explode: None,
                        allow_reserved: false,
                        schema: schemars::schema::SchemaObject::new_ref(String::from(
                            format!("#/components/schemas/{}", $schema),
                        )),
                        example: None,
                        examples: None,
                    },
                })
            }
        }

        impl serde::ser::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                serializer
                    .serialize_str(base64::encode_config(self.as_slice(), base64::URL_SAFE).as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<$name, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                struct Base64Visitor;

                impl<'de> serde::de::Visitor<'de> for Base64Visitor {
                    type Value = $name;

                    fn expecting( &self,formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        write!(formatter, "base64 ASCII text")
                    }

                    fn visit_str<E>(self, v: &str) -> ::std::result::Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        let bytes_vec = base64::decode_config(v, base64::URL_SAFE).map_err(serde::de::Error::custom)?;
                        std::convert::TryFrom::try_from(bytes_vec).map_err(serde::de::Error::custom)
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
    }
}