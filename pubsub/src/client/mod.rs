use std::sync::{Arc, Mutex};

use datastore::topics::TopicKeyHandle;

pub type PubSubClientIDType = u16;

pub struct PubSubClient {
    pub id: PubSubClientIDType,
    pub subscriptions: Vec<TopicKeyHandle>,
    pub publishers: Vec<TopicKeyHandle>,
}

impl PubSubClient {
    pub fn new(id: PubSubClientIDType) -> Self {
        PubSubClient {
            id,
            subscriptions: Vec::new(),
            publishers: Vec::new(),
        }
    }
}

pub type PubSubClientHandle = Arc<Mutex<PubSubClient>>;
