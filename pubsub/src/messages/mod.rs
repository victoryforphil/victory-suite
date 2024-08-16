mod health;
mod publish;
mod register;
mod subscribe;
mod update;
pub use health::*;
pub use publish::*;
pub use register::*;
pub use subscribe::*;
pub use update::*;
#[derive(Debug, Clone)]
pub enum PubSubMessage {
    Register(),
    Publish(),
    Subscribe(SubscribeMessage),
    Update(),
    Health(),
}
