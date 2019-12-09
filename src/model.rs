use serde::Serialize;
use serde::Deserialize;
use super::serde_fixed;

pub type StreamId = [u8; 8];
pub type MsgSignature = [u8; 64];
pub type MsgData = [u8; 128];

serde_fixed!(SerdeArray64Base64, 64);
serde_fixed!(SerdeArray128Base64, 128);

#[derive(Serialize, Deserialize)]
pub struct Message {
    #[serde(with = "SerdeArray64Base64")] pub signature: MsgSignature,
    #[serde(with = "SerdeArray128Base64")] pub data: MsgData
}