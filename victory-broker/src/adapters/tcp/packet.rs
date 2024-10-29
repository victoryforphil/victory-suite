use crate::{channel::PubSubChannelIDType, messages::PubSubMessage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TCPPacket {
    pub from: PubSubChannelIDType,
    pub to: PubSubChannelIDType,
    pub messages: Vec<PubSubMessage>,
}
