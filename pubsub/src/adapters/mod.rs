mod mock;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use datastore::{datapoints::Datapoint, topics::TopicKeyHandle};

use crate::client::PubSubClientHandle;

pub trait PubSubAdapter {
    fn collect_clients(&mut self) -> Vec<PubSubClientHandle>;
}

pub trait PubSubAdapterOffspring {
    fn read(&mut self) -> BTreeMap<TopicKeyHandle, Vec<Datapoint>>;
    fn write(&mut self, data: BTreeMap<TopicKeyHandle, Vec<Datapoint>>);
}

pub type PubSubAdapterHandle = Arc<Mutex<dyn PubSubAdapter>>;
pub type PubSubAdapterOffspringHandle = Arc<Mutex<dyn PubSubAdapterOffspring>>;
