pub mod mock;

use std::collections::HashMap;

use crate::{client::PubSubClientIDType, messages::PubSubMessage, MutexType};

pub trait PubSubAdapter {
    fn read(&mut self) -> HashMap<PubSubClientIDType, Vec<PubSubMessage>>;
    fn write(&mut self, to_send: HashMap<PubSubClientIDType, Vec<PubSubMessage>>);
}
pub type PubSubAdapterHandle = MutexType<dyn PubSubAdapter + Send>;
