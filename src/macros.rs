#[macro_export]
macro_rules! decode_fixed {
  ($string:expr, $size:expr) => {
      match base64::decode_mode($string, base64::Base64Mode::UrlSafe) {
          Ok(bytes_vec) =>
              if bytes_vec.len() == $size {
                  let mut bytes: [u8; $size] = [0; $size];
                  bytes.copy_from_slice(bytes_vec.as_slice());
                  Ok(bytes)
              } else {
                  Err("Invalid byte size")
              }
          _ => Err("Invalid base64")
      }
  }
}

#[macro_export]
macro_rules! serde_fixed {
    ($name:ident, $size:expr) => {
        enum $name {}
        impl $name {
            #[allow(dead_code)]
            pub fn serialize<S>(bytes: &[u8; $size], serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::ser::Serializer {
                serializer.serialize_str(base64::encode_mode(bytes, base64::Base64Mode::UrlSafe).as_str())
            }

            #[allow(dead_code)]
            pub fn deserialize<'de, D>(deserializer: D) -> ::std::result::Result<[u8; $size], D::Error>
            where D: serde::de::Deserializer<'de> {
                struct Base64Visitor;

                impl<'de> serde::de::Visitor<'de> for Base64Visitor {
                    type Value = [u8; $size];

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        write!(formatter, "base64 ASCII text")
                    }

                    fn visit_str<E>(self, v: &str) -> ::std::result::Result<Self::Value, E> where
                            E: serde::de::Error, {
                        super::decode_fixed!(v, $size).map_err(serde::de::Error::custom)
                    }
                }

                deserializer.deserialize_str(Base64Visitor)
            }
        }
    }
}