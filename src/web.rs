use rocket_contrib::json::Json;
use schemars::JsonSchema;
use serde::Serialize;
use super::{db, model, serde_fixed, decode_fixed};

#[derive(Debug)]
pub enum Error {
  DbError
}

#[openapi]
#[get("/streams/<stream_id>/messages?<offset>&<limit>")]
pub fn get_messages(
  conn: db::RocketConn,
  stream_id: WrappedStreamId,
  offset: Option<u32>,
  limit: Option<u8>
) -> Result<Json<GetMessagesResponse>, Error> {
  let offset: u32 = offset.unwrap_or(0);
  let limit: u8 = limit.unwrap_or(100);
  let stream_id = stream_id.value;

  // TODO: 500 instead of panic
  let messages = db::get_messages(&*conn, &stream_id, offset, limit).map_err(|_| Error::DbError)?;

  Ok(Json(GetMessagesResponse { id: stream_id, messages } ))
}

#[openapi]
#[post("/streams/<stream_id>/messages", data="<message>")]
pub fn create_message(
  conn: db::RocketConn,
  stream_id: WrappedStreamId,
  message: Json<model::Message>
) -> Result<(), Error> {
  let stream_id = stream_id.value;

  db::add_message(&*conn, &stream_id, message.into_inner()).map_err(|_| Error::DbError)
}

pub struct WrappedStreamId {
  value: model::StreamId
}

impl<'r> rocket::request::FromParam<'r> for WrappedStreamId {
  type Error = ();

  fn from_param(param: &'r rocket::http::RawStr) -> Result<Self, Self::Error> {
      decode_fixed!(param.url_decode_lossy().as_str(), 8).map(|value| WrappedStreamId{value}).map_err(|_| ())
  }
}

impl<'r> rocket_okapi::request::OpenApiFromParam<'r> for WrappedStreamId {
  fn path_parameter(_gen: &mut rocket_okapi::gen::OpenApiGenerator, name: String) -> Result<okapi::openapi3::Parameter, rocket_okapi::OpenApiError> {
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
        schema: schemars::schema::SchemaObject::new_ref(
          String::from("#/components/schemas/8BytesBase64Encoded")
        ),
        example: None,
        examples: None
      }
    })
  }
}

serde_fixed!(SerdeArray8Base64, "8BytesBase64Encoded", 8);

#[derive(Serialize, JsonSchema)]
pub struct GetMessagesResponse {
  #[serde(with = "SerdeArray8Base64")] id: model::StreamId,
  messages: Vec<model::Message>
}