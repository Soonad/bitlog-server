#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate base64;
extern crate redis;
use base64::Base64Mode;
use redis::Commands;
use rocket_contrib::json::Json;
use serde::ser::Serializer;
use serde::de::Deserializer;
use serde::Serialize;
use serde::Deserialize;

// Some macros yey
macro_rules! decode_fixed {
    ($string:expr, $size:expr) => {
        match base64::decode_mode($string, Base64Mode::UrlSafe) {
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

macro_rules! serde_fixed {
    ($name:ident, $size:expr) => {
        enum $name {}
        impl $name {
            pub fn serialize<S>(bytes: &[u8; $size], serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                serializer.serialize_str(base64::encode_mode(bytes, Base64Mode::UrlSafe).as_str())
            }

            pub fn deserialize<'de, D>(deserializer: D) -> ::std::result::Result<[u8; $size], D::Error>
            where D: Deserializer<'de> {
                struct Base64Visitor;

                impl<'de> serde::de::Visitor<'de> for Base64Visitor {
                    type Value = [u8; $size];

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        write!(formatter, "base64 ASCII text")
                    }

                    fn visit_str<E>(self, v: &str) -> ::std::result::Result<Self::Value, E> where
                            E: serde::de::Error, {
                        decode_fixed!(v, $size).map_err(serde::de::Error::custom)
                    }
                }

                deserializer.deserialize_str(Base64Visitor)
            }
        }
    }
}

serde_fixed!(SerdeArray8Base64, 8);
serde_fixed!(SerdeArray64Base64, 64);
serde_fixed!(SerdeArray128Base64, 128);

type StreamId = [u8; 8];
type MsgSignature = [u8; 64];
type MsgData = [u8; 128];

#[derive(Serialize, Deserialize)]
struct Message {
    #[serde(with = "SerdeArray64Base64")] signature: MsgSignature,
    #[serde(with = "SerdeArray128Base64")] data: MsgData
}

// DB Stuff

impl redis::FromRedisValue for Message {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            redis::Value::Data(data) => {
                if data.len() == 192 {
                    let (data_vec, sig_vec) = data.split_at(128);
                    let mut data = [0; 128];
                    let mut signature = [0; 64];
                    data.copy_from_slice(data_vec);
                    signature.copy_from_slice(sig_vec);

                    Ok(Message{data, signature})
                } else {
                    Err(redis::RedisError::from((redis::ErrorKind::TypeError, "Message size is incorrect")))
                }
            }
            _ => Err(redis::RedisError::from((redis::ErrorKind::TypeError, "Data type is incorrect for a message")))
        }
    }
}

impl redis::ToRedisArgs for Message {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where W: redis::RedisWrite,
    {
        let mut bytes = [0; 192];
        bytes[..128].copy_from_slice(&self.data);
        bytes[128..].copy_from_slice(&self.signature);
        out.write_arg(&bytes);
    }
}

fn get_messages_from_db(con: &mut redis::Connection, stream_id: &StreamId, offset: u32, limit: u8) -> redis::RedisResult<Vec<Message>> {
    con.lrange(stream_id, offset as isize, limit as isize)
}

fn add_message_to_db(con: &mut redis::Connection, stream_id: &StreamId, message: Message) -> redis::RedisResult<()> {
    con.rpush(stream_id, message)
}

// Web only stuff

struct WrappedStreamId {
    value: StreamId
}

impl<'r> rocket::request::FromParam<'r> for WrappedStreamId {
    type Error = ();

    fn from_param(param: &'r rocket::http::RawStr) -> Result<Self, Self::Error> {
        decode_fixed!(param.url_decode_lossy().as_str(), 8).map(|value| WrappedStreamId{value}).map_err(|_| ())
    }
}

#[derive(Serialize)]
struct GetMessagesResponse {
    #[serde(with = "SerdeArray8Base64")] id: StreamId,
    messages: Vec<Message>
}

// Handlers
#[get("/streams/<w_stream_id>/messages?<offset>&<limit>")]
fn get_messages(w_stream_id: WrappedStreamId, offset: Option<u32>, limit: Option<u8>) -> Json<GetMessagesResponse> {
    let offset: u32 = offset.unwrap_or(0);
    let limit: u8 = limit.unwrap_or(100);
    let stream_id = w_stream_id.value;

    // TODO: Make it return a 500 instead of just panic
    // TODO: Use r2d2 pool as a middleware
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();

    // TODO: 500 instead of panic
    let messages = get_messages_from_db(&mut con, &stream_id, offset, limit).unwrap();

    Json(GetMessagesResponse { id: stream_id, messages } )
}

// Handlers
#[post("/streams/<w_stream_id>/messages", data="<message>")]
fn create_message(w_stream_id: WrappedStreamId, message: Json<Message>) -> () {
    let stream_id = w_stream_id.value;

    // TODO: Make it return a 500 instead of just panic
    // TODO: Use r2d2 pool as a middleware
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();

    // TODO: 500 instead of panic
    add_message_to_db(&mut con, &stream_id, message.into_inner()).unwrap();
}

fn main() {
    rocket::ignite().mount("/", routes![get_messages, create_message]).launch();
}