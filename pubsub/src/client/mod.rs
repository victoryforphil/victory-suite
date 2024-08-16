use std::sync::{Arc, Mutex};

use datastore::topics::TopicKeyHandle;

use crate::adapters::PubSubAdapterOffspringHandle;

pub struct PubSubClient {
    pub id: String,
    pub subscriptions: Vec<TopicKeyHandle>,
    pub publishers: Vec<TopicKeyHandle>,
    pub adapter_offspring: PubSubAdapterOffspringHandle,
}

impl PubSubClient {
    pub fn new(id: String, adapter_offspring: PubSubAdapterOffspringHandle) -> Self {
        PubSubClient {
            id,
            subscriptions: Vec::new(),
            publishers: Vec::new(),
            adapter_offspring,
        }
    }
}

pub type PubSubClientHandle = Arc<Mutex<PubSubClient>>;
