use rocket_contrib::json::Json;
use serde::Serialize;
use super::{db, model, serde_fixed, decode_fixed};

#[get("/streams/<w_stream_id>/messages?<offset>&<limit>")]
pub fn get_messages(conn: db::RocketConn, w_stream_id: WrappedStreamId, offset: Option<u32>, limit: Option<u8>) -> Json<GetMessagesResponse> {
  let offset: u32 = offset.unwrap_or(0);
  let limit: u8 = limit.unwrap_or(100);
  let stream_id = w_stream_id.value;

  // TODO: 500 instead of panic
  let messages = db::get_messages(&*conn, &stream_id, offset, limit).unwrap();

  Json(GetMessagesResponse { id: stream_id, messages } )
}

#[post("/streams/<w_stream_id>/messages", data="<message>")]
pub fn create_message(conn: db::RocketConn, w_stream_id: WrappedStreamId, message: Json<model::Message>) -> () {
  let stream_id = w_stream_id.value;

  // TODO: 500 instead of panic
  db::add_message(&*conn, &stream_id, message.into_inner()).unwrap();
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

serde_fixed!(SerdeArray8Base64, 8);

#[derive(Serialize)]
pub struct GetMessagesResponse {
  #[serde(with = "SerdeArray8Base64")] id: model::StreamId,
  messages: Vec<model::Message>
}