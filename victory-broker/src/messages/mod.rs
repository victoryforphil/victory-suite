mod health;
mod publish;
mod register;
mod subscribe;
pub use publish::*;
use serde::{Deserialize, Serialize};
pub use subscribe::*;

use crate::channel::PubSubChannelIDType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PubSubMessage {
    Register(),
    Publish(PublishMessage),
    Subscribe(SubscribeMessage),
    Health(),
    Welcome(PubSubChannelIDType),
}
