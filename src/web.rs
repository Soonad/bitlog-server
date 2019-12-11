use super::{db, model, fixed_size_byte_array};
use rocket_contrib::json::Json;
use schemars::JsonSchema;
use serde::Serialize;
use serde::Deserialize;

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
    data: MessageData
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
        
    let messages: Vec<Message> = messages.iter().map(|message| Message {
        data: MessageData::from(message.data),
        signature: MessageSignature::from(message.signature)
    }).collect();

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
            signature: *message.signature.as_array()
        };

        db::add_message(&*conn, m_stream_id, m_message).map_err(|_| Error::DbError)
}
