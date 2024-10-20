use crate::{client::PubSubClientIDType, messages::PubSubMessage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TCPPacket {
    pub from: PubSubClientIDType,
    pub to: PubSubClientIDType,
    pub messages: Vec<PubSubMessage>,
}
