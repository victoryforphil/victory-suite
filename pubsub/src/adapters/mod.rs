pub mod mock;

use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};

use datastore::{datapoints::Datapoint, topics::TopicKeyHandle};

use crate::{
    client::{PubSubClientHandle, PubSubClientIDType},
    messages::PubSubMessage,
};

pub trait PubSubAdapter {
    fn read(&mut self) -> HashMap<PubSubClientIDType, Vec<PubSubMessage>>;
    fn write(&mut self, to_send: HashMap<PubSubClientIDType, Vec<PubSubMessage>>);
}
pub type PubSubAdapterHandle = Arc<Mutex<dyn PubSubAdapter>>;
