use std::collections::{BTreeMap, HashMap};


use log::debug;
use sub_callback::SubCallbackHandle;
use victory_data_store::{datapoints::{Datapoint, DatapointMap}, topics::{TopicKeyHandle, TopicKeyProvider}};

use crate::{
    adapters::{PubSubAdapter, PubSubAdapterHandle},
    client::PubSubClientIDType,
    messages::{PubSubMessage, PublishMessage},
};
pub mod sub_callback;

pub struct Node {
    pub id: PubSubClientIDType,
    pub name: String,
    pub adapter: PubSubAdapterHandle,
    pub sub_callbacks: BTreeMap<TopicKeyHandle, SubCallbackHandle>,
    publish_queue: Vec<PublishMessage>,
}

impl Node {
    pub fn new(id: PubSubClientIDType, name: String, adapter: PubSubAdapterHandle) -> Self {
        debug!(
            "Node::new: Creating new node with id {} and name {}",
            id, name
        );
        Node {
            id,
            name,
            adapter,
            sub_callbacks: BTreeMap::new(),
            publish_queue: Vec::new(),
        }
    }

    pub fn add_sub_callback<T: TopicKeyProvider>(&mut self, topic: T, callback: SubCallbackHandle) {
        debug!(
            "Node::add_sub_callback: Adding callback for topic {:?}",
            topic.key()
        );
        self.sub_callbacks.insert(topic.handle(), callback);
    }

    pub fn publish_message(&mut self, message: PublishMessage) {
        debug!("Node::publish_message: Publishing message {:?}", message);
        self.publish_queue.push(message);
    }

    pub fn publish_datapoint(&mut self, datapoint: &Datapoint) {
        let message = PublishMessage::new(&datapoint.topic, vec![datapoint.clone()]);
        self.publish_message(message);
    }

    pub fn tick(&mut self) {
        let mut adapter = self.adapter.try_lock().unwrap();

        let mut sub_map: DatapointMap = BTreeMap::new();
        let read_message = adapter.read();
        let read_message: Vec<&PubSubMessage> =
            read_message.iter().map(|(_, v)| v).flatten().collect();
        debug!("Node::tick: Read {} messages", read_message.len());
        for msg in read_message.iter() {
            match msg {
                PubSubMessage::Update(message) => {
                    sub_map.insert(message.topic.clone(), message.messages.clone());
                }
                _ => {}
            }
        }

        for (topic, datapoint) in sub_map.iter() {
            if let Some(callback) = self.sub_callbacks.get(topic) {
                let mut callback = callback.try_lock().unwrap();
                callback.on_update(&sub_map);
            }
        }

        debug!(
            "Node::tick: Publishing {} messages",
            self.publish_queue.len()
        );
        for message in self.publish_queue.drain(..) {
            let mut to_send = HashMap::new();
            for datapoint in message.messages.iter() {
                let topic = datapoint.topic.clone();
                to_send.insert(
                    self.id,
                    vec![PubSubMessage::Publish(PublishMessage::new(
                        &topic,
                        vec![datapoint.clone()],
                    ))],
                );
            }
            adapter.write(to_send);
        }
    }

    pub fn get_queue_len(&self) -> usize {
        self.publish_queue.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::adapters::mock::MockPubSubAdapter;
    use crate::messages::UpdateMessage;

    use super::*;

    use log::info;
    use tokio::sync::Mutex;
    use victory_data_store::topics::TopicKey;
    use victory_time_rs::Timepoint;

    use std::sync::mpsc::{channel, Sender};
    use std::sync::Arc;

    use std::vec;

    struct TestSubCallback {
        sender: Sender<DatapointMap>,
    }

    impl sub_callback::SubCallback for TestSubCallback {
        fn on_update(&mut self, datapoints: &DatapointMap) {
            info!("TestSubCallback::on_update: Received {:?}", datapoints);
            self.sender.send(datapoints.clone()).unwrap();
        }
    }

    #[test]
    fn test_node_new() {
        let adapter = Arc::new(Mutex::new(MockPubSubAdapter::new()));
        let node = Node::new(1, "test".to_string(), adapter);
        assert_eq!(node.id, 1);
        assert_eq!(node.name, "test");
        assert_eq!(node.sub_callbacks.len(), 0);
        assert_eq!(node.get_queue_len(), 0);
    }

    #[test]
    fn test_node_publish() {
        pretty_env_logger::init();
        let adapter = Arc::new(Mutex::new(MockPubSubAdapter::new()));
        let mut node = Node::new(1, "test".to_string(), adapter.clone());

        let topic: TopicKey = "test/topic".into();
        let (tx, rx) = channel();
        let callback = TestSubCallback { sender: tx };
        let datapoint = Datapoint::new(
            &topic.clone(),
            Timepoint::new_secs(1.0),
            "test".into(),
        );
        node.publish_datapoint(&datapoint);

        node.tick();
        let mut adapter = adapter.try_lock().unwrap();
        let read_result = adapter.client_read(1);
        info!("Clients read: {:?}", adapter.client_ids());
        assert_eq!(read_result.len(), 1);
    }

    #[test]
    fn test_node_sub_callback() {
        let adapter = Arc::new(Mutex::new(MockPubSubAdapter::new()));
        let mut node = Node::new(1, "test".to_string(), adapter.clone());

        let topic = TopicKey::from_str("test/topic");
        let (tx, rx) = channel();
        let callback = TestSubCallback { sender: tx };
        let callback = Arc::new(Mutex::new(callback));
        node.add_sub_callback(topic.handle().clone(), callback.clone());

        let datapoint = Datapoint::new(
            &topic.clone(),
            Timepoint::new_secs(2.0),
            "test".into(),
        );
        let message = UpdateMessage::new(datapoint.clone());

        {
            adapter
                .try_lock()
                .unwrap()
                .client_write(1, vec![PubSubMessage::Update(message.clone())]);
        }

        {
            node.tick();
        }
        {
            let read_result = adapter.try_lock().unwrap().client_read(1);
            assert_eq!(read_result.len(), 0);
            let received = rx.try_recv().unwrap();
            assert_eq!(received.len(), 1);
        }
    }
}
