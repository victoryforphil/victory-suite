use datastore::topics::TopicKeyHandle;

use crate::MutexType;

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

pub type PubSubClientHandle = MutexType<PubSubClient>;
