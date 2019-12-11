pub type StreamId = [u8; 8];
pub type MsgSignature = [u8; 64];
pub type MsgData = [u8; 128];

pub struct Message {
    pub signature: MsgSignature,
    pub data: MsgData,
}
