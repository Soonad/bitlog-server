use super::{db, fixed_size_byte_array, model};
use rocket_contrib::json::Json;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug)]
pub enum Error {
    DbError,
}

fixed_size_byte_array!(StreamId, "StreamId", 8);
fixed_size_byte_array!(MessageSignature, "MessageSignature", 64);
fixed_size_byte_array!(MessageData, "MessageData", 128);

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Message {
    signature: MessageSignature,
    data: MessageData,
}

#[derive(Serialize, JsonSchema)]
pub struct GetMessagesResponse {
    id: StreamId,
    messages: Vec<Message>,
}

#[openapi]
#[get("/streams/<stream_id>/messages?<offset>&<limit>")]
pub fn get_messages(
    conn: db::RocketConn,
    stream_id: StreamId,
    offset: Option<u32>,
    limit: Option<u8>,
) -> Result<Json<GetMessagesResponse>, Error> {
    let offset: u32 = offset.unwrap_or(0);
    let limit: u8 = limit.unwrap_or(100);
    let m_stream_id = stream_id.as_array();

    let messages =
        db::get_messages(&*conn, m_stream_id, offset, limit).map_err(|_| Error::DbError)?;

    let messages: Vec<Message> = messages
        .iter()
        .map(|message| Message {
            data: MessageData::from(message.data),
            signature: MessageSignature::from(message.signature),
        })
        .collect();

    Ok(Json(GetMessagesResponse {
        id: stream_id,
        messages,
    }))
}

#[openapi]
#[post("/streams/<stream_id>/messages", data = "<message>")]
pub fn create_message(
    conn: db::RocketConn,
    stream_id: StreamId,
    message: Json<Message>,
) -> Result<(), Error> {
    let message = message.into_inner();
    let m_stream_id = stream_id.as_array();
    let m_message = model::Message {
        data: *message.data.as_array(),
        signature: *message.signature.as_array(),
    };

    db::add_message(&*conn, m_stream_id, m_message).map_err(|_| Error::DbError)
}

// RFC 7807 Error Message
#[derive(Serialize, JsonSchema)]
pub struct ErrorResponse {
    _type: &'static str,
    title: &'static str,
    status: u16,
    detail: &'static str
}

#[catch(404)]
pub fn not_found(_req: &rocket::Request) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        _type: "urn:bitlog:errors:not_found",
        title: "Resource Not Found",
        status: 404,
        detail: concat!(
            "The requested URL and method combination is not a valid one.",
            "Please refer to the OAS Spec on https://bitlog.fm/openapi.json"
        )
    })
}

#[catch(422)]
pub fn unprocessable_entity(_req: &rocket::Request) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        _type: "urn:bitlog:errors:unprocessable_entity",
        title: "Unprocessable Entity",
        status: 422,
        detail: concat!(
            "The request was well formed but the content one or more fields is incorrect.",
            "Please refer to the OAS Spec on https://bitlog.fm/openapi.json"
        )
    })
}

#[catch(500)]
pub fn server_error(_req: &rocket::Request) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        _type: "urn:bitlog:errors:internal_server_error",
        title: "Internal Server Error",
        status: 500,
        detail: "Something unexpected happened. This is our fault. We are sorry about that. Please try again later"
    })
}

pub fn rocket() -> rocket::Rocket {
    let cors_options: rocket_cors::CorsOptions = rocket_cors::CorsOptions::default();
    let cors = cors_options.to_cors().expect("Cors");

    rocket::ignite()
        .mount("/", routes_with_openapi![get_messages, create_message])
        .register(catchers![not_found, unprocessable_entity, server_error])
        .attach(db::RocketConn::fairing())
        .attach(cors)
}
