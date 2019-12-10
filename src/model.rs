use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use super::serde_fixed;

pub type StreamId = [u8; 8];
pub type MsgSignature = [u8; 64];
pub type MsgData = [u8; 128];

serde_fixed!(SerdeArray64Base64, "Bytes64Base64Encoded", 64);
serde_fixed!(SerdeArray128Base64, "Bytes128Base64Encoded", 128);

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Message {
    #[serde(with = "SerdeArray64Base64")] pub signature: MsgSignature,
    #[serde(with = "SerdeArray128Base64")] pub data: MsgData
}