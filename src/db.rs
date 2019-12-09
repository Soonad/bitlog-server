use super::model::{Message, StreamId};
use redis::{
  Commands,
  Connection,
  ErrorKind,
  FromRedisValue,
  RedisError,
  RedisResult,
  RedisWrite,
  ToRedisArgs,
  Value
};

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

                  Ok(Message{data, signature})
              } else {
                  Err(RedisError::from((ErrorKind::TypeError, "Message size is incorrect")))
              }
          }
          _ => Err(RedisError::from((ErrorKind::TypeError, "Data type is incorrect for a message")))
      }
  }
}

impl ToRedisArgs for Message {
  fn write_redis_args<W: ?Sized>(&self, out: &mut W)
  where W: RedisWrite,
  {
      let mut bytes = [0; 192];
      bytes[..128].copy_from_slice(&self.data);
      bytes[128..].copy_from_slice(&self.signature);
      out.write_arg(&bytes);
  }
}

pub fn get_messages(con: &mut Connection, stream_id: &StreamId, offset: u32, limit: u8) -> RedisResult<Vec<Message>> {
  con.lrange(stream_id, offset as isize, limit as isize)
}

pub fn add_message(con: &mut Connection, stream_id: &StreamId, message: Message) -> RedisResult<()> {
  con.rpush(stream_id, message)
}