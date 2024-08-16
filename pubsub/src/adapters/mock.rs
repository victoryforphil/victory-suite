use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use datastore::{datapoints::Datapoint, topics::TopicKeyHandle};
use log::info;

use crate::client::{PubSubClient, PubSubClientHandle};

use super::{PubSubAdapter, PubSubAdapterOffspring, PubSubAdapterOffspringHandle};

pub struct MockPubSubOffpspring {
    pub data: BTreeMap<TopicKeyHandle, Vec<Datapoint>>,
}

impl PubSubAdapterOffspring for MockPubSubOffpspring {
    fn read(&mut self) -> BTreeMap<TopicKeyHandle, Vec<Datapoint>> {
        // Drain read
        let data = self.data.clone();
        self.data.clear();
        data
    }

    fn write(&mut self, data: BTreeMap<TopicKeyHandle, Vec<Datapoint>>) {
        self.data = data;
    }
}

pub struct MockPubSubClient {
    pub id: String,
    pub subscriptions: Vec<TopicKeyHandle>,
    pub publishers: Vec<TopicKeyHandle>,
    pub adapter_offspring: PubSubAdapterOffspringHandle,
}

pub struct MockPubSubAdapter {
    pub clients: Vec<PubSubClientHandle>,
}

impl MockPubSubAdapter {
    pub fn new() -> Self {
        MockPubSubAdapter {
            clients: Vec::new(),
        }
    }

    pub fn register_client(&mut self, id: String) {
        info!("Creating new MockPubSubClient: {}", id);
        let offspring = Arc::new(Mutex::new(MockPubSubOffpspring {
            data: BTreeMap::new(),
        }));
        let client = Arc::new(Mutex::new(PubSubClient::new(id, offspring)));
        self.clients.push(client);
    }
}

impl PubSubAdapter for MockPubSubAdapter {
    fn collect_clients(&mut self) -> Vec<PubSubClientHandle> {
        // Drain the clients and return them
        let clients = self.clients.clone();
        self.clients.clear();
        clients
    }
}

#[cfg(test)]
mod tests {
    use datastore::{
        primitives::timestamp::VicInstant,
        topics::{TopicKey, TopicKeyProvider},
    };

    use super::*;

    #[test]
    fn test_mock_pubsub_adapter_new_client() {
        let mut adapter = MockPubSubAdapter::new();
        adapter.register_client("test".to_string());
        let clients = adapter.collect_clients();
        assert_eq!(clients.len(), 1);
    }

    #[test]
    fn test_mock_pubsub_adapter_offspring() {
        let mut adapter = MockPubSubAdapter::new();
        adapter.register_client("test".to_string());
        let clients = adapter.collect_clients();
        assert_eq!(clients.len(), 1);

        let client = clients[0].lock().unwrap();
        let mut offspring = client.adapter_offspring.lock().unwrap();

        let mut mock_data = BTreeMap::new();
        let mock_topic = TopicKey::from_str("test").handle();
        mock_data.insert(
            mock_topic.clone(),
            vec![Datapoint::new(
                &mock_topic,
                VicInstant::now().handle(),
                1.into(),
            )],
        );
        offspring.write(mock_data);

        let data = offspring.read();
        assert_eq!(data.len(), 1);
    }
}
