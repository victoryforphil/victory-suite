pub mod mock;
pub mod tcp;

use std::collections::HashMap;

use crate::{channel::PubSubChannelIDType, messages::PubSubMessage, MutexType};

pub trait PubSubAdapter {
    fn get_name(&self) -> String;

    fn get_description(&self) -> String {
        format!("Adapter: {}", self.get_name())
    }

    fn get_stats(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn get_live(&self) -> bool {
        true
    }

    fn read(&mut self) -> HashMap<PubSubChannelIDType, Vec<PubSubMessage>>;
    fn write(&mut self, to_send: HashMap<PubSubChannelIDType, Vec<PubSubMessage>>);
}
pub type PubSubAdapterHandle = MutexType<dyn PubSubAdapter + Send>;
