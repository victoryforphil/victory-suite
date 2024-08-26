mod health;
mod publish;
mod register;
mod subscribe;
mod update;
pub use publish::*;
pub use subscribe::*;
pub use update::*;

use serde::{Deserialize, Serialize};

use crate::client::PubSubClientIDType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PubSubMessage {
    Register(),
    Publish(PublishMessage),
    Subscribe(SubscribeMessage),
    Update(UpdateMessage),
    Health(),
    Welcome(PubSubClientIDType),
}
