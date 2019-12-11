use redis::{
    Commands, Connection, ErrorKind, FromRedisValue, RedisError, RedisResult, ToRedisArgs, Value,
};

use rocket_contrib::databases::redis;

use super::model::{Message, StreamId};

pub fn get_messages(
    conn: &Connection,
    stream_id: &StreamId,
    offset: u32,
    limit: u8,
) -> RedisResult<Vec<Message>> {
    conn.lrange(stream_id, offset as isize, limit as isize)
}

pub fn add_message(conn: &Connection, stream_id: &StreamId, message: Message) -> RedisResult<()> {
    conn.rpush(stream_id, message)
}

#[database("redis_db")]
pub struct RocketConn(Connection);

impl FromRedisValue for Message {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        match v {
            Value::Data(data) => {
                if data.len() == 192 {
                    let (data_vec, sig_vec) = data.split_at(128);
                    let mut data = [0; 128];
                    let mut signature = [0; 64];
                    data.copy_from_slice(data_vec);
                    signature.copy_from_slice(sig_vec);

                    Ok(Message { data, signature })
                } else {
                    Err(RedisError::from((
                        ErrorKind::TypeError,
                        "Message size is incorrect",
                    )))
                }
            }
            _ => Err(RedisError::from((
                ErrorKind::TypeError,
                "Data type is incorrect for a message",
            ))),
        }
    }
}

impl ToRedisArgs for Message {
    fn write_redis_args(&self, out: &mut Vec<Vec<u8>>) {
        let mut bytes = [0; 192];
        bytes[..128].copy_from_slice(&self.data);
        bytes[128..].copy_from_slice(&self.signature);
        out.push(bytes.to_vec());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_message_parsing_and_reading() {
        let message = Message {
            data: [1; 128],
            signature: [2; 64]
        };
        let mut args = vec![];
        Message::write_redis_args(&message, &mut args);
        assert_eq!(args.len(), 1);

        let value = Value::Data(args[0].clone());
        let decoded_message = Message::from_redis_value(&value).unwrap();

        assert!(decoded_message.data.iter().eq(message.data.iter()));
        assert!(decoded_message.signature.iter().eq(message.signature.iter()));
    }

    // TODO: Add test for the actual connection.
}