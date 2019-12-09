use rocket_contrib::json::Json;
use serde::Serialize;
use super::{db, model, serde_fixed, decode_fixed};

struct WrappedStreamId {
  value: model::StreamId
}

serde_fixed!(SerdeArray8Base64, 8);

impl<'r> rocket::request::FromParam<'r> for WrappedStreamId {
  type Error = ();

  fn from_param(param: &'r rocket::http::RawStr) -> Result<Self, Self::Error> {
      decode_fixed!(param.url_decode_lossy().as_str(), 8).map(|value| WrappedStreamId{value}).map_err(|_| ())
  }
}

#[derive(Serialize)]
struct GetMessagesResponse {
  #[serde(with = "SerdeArray8Base64")] id: model::StreamId,
  messages: Vec<model::Message>
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
  let messages = db::get_messages(&mut con, &stream_id, offset, limit).unwrap();

  Json(GetMessagesResponse { id: stream_id, messages } )
}

// Handlers
#[post("/streams/<w_stream_id>/messages", data="<message>")]
fn create_message(w_stream_id: WrappedStreamId, message: Json<model::Message>) -> () {
  let stream_id = w_stream_id.value;

  // TODO: Make it return a 500 instead of just panic
  // TODO: Use r2d2 pool as a middleware
  let client = redis::Client::open("redis://127.0.0.1/").unwrap();
  let mut con = client.get_connection().unwrap();

  // TODO: 500 instead of panic
  db::add_message(&mut con, &stream_id, message.into_inner()).unwrap();
}

pub fn run() {
  rocket::ignite().mount("/", routes![get_messages, create_message]).launch();
}