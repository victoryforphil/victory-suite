use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};

use bucket_listener::NodeBucketListener;
use log::{debug, info, trace};
use sub_callback::SubCallbackHandle;
use victory_data_store::{
    database::{Datastore, DatastoreHandle},
    datapoints::{Datapoint, DatapointMap},
    primitives::Primitives,
    topics::{TopicKeyHandle, TopicKeyProvider},
};
use victory_wtf::Timepoint;

use crate::{
    adapters::{PubSubAdapter, PubSubAdapterHandle},
    channel::{PubSubChannel, PubSubChannelHandle, PubSubChannelIDType},
    messages::{PubSubMessage, PublishMessage},
};
pub mod bucket_listener;
pub mod sub_callback;
pub struct Node {
    pub name: String,
    pub adapter: PubSubAdapterHandle,
    pub datastore: DatastoreHandle,
    pub sub_callbacks: BTreeMap<TopicKeyHandle, SubCallbackHandle>,
    pub channels: HashMap<PubSubChannelIDType, PubSubChannelHandle>,
    pub bucket_listener: Arc<Mutex<NodeBucketListener>>,
}

impl Node {
    pub fn new(name: String, adapter: PubSubAdapterHandle, datastore: DatastoreHandle) -> Self {
        debug!("Node::new: Creating new node with name {}", name);
        let bucket_listener = Arc::new(Mutex::new(NodeBucketListener::new()));
        Node {
            name,
            adapter,
            datastore,
            sub_callbacks: BTreeMap::new(),
            channels: HashMap::new(),
            bucket_listener,
        }
    }
    pub fn register(&mut self) {
        let register_message = PubSubMessage::Register();
        self.send_message(0, register_message);
    }
    pub fn add_sub_callback<T: TopicKeyProvider>(&mut self, topic: T, callback: SubCallbackHandle) {
        debug!("Adding callback for topic {:?}", topic.key());

        self.sub_callbacks.insert(topic.handle(), callback);

        for channel in self.channels.values_mut() {
            let mut channel = channel.lock().unwrap();
            channel.add_subscribe(topic.handle());
        }
    }

    pub fn tick(&mut self) {
        // 1. Read incoming messages from adapters
        let msgs = {
            let mut adapter = self.adapter.lock().unwrap();
            adapter.read()
        };
        self.process_incoming_messages(msgs);

        // 2. Check for local datastore updates and send to remote channels
        let datapoints = {
            let mut bucket_listener = self.bucket_listener.lock().unwrap();
            bucket_listener.drain()
        };
        if datapoints.len() > 0 {
            debug!("Sending {} datapoints to remote channels", datapoints.len());
        }
        for datapoint in datapoints {
            self.on_datapoint(&datapoint);
        }

        // 4. Check for recived datapoints from remote channels
        let mut msgs = Vec::new();
        for channel in self.channels.values_mut() {
            let mut channel = channel.lock().unwrap();
            msgs.extend(channel.drain_recv_queue());
        }
        if msgs.len() > 0 {
            debug!(
                "Applying and notifying {} datapoints from remote channels",
                msgs.len()
            );
        }

        self.apply_datapoints(&msgs);
        self.notify_sub_callbacks(&msgs);

        for (_, channel) in self.channels.iter_mut() {
            let mut channel = channel.lock().unwrap();
            let send_queue = channel.drain_send_queue();

            for (channel_id, messages) in send_queue {
                // Chunk messages into 8
                let mut chunks = messages.chunks(16);
                while let Some(chunk) = chunks.next() {
                    self.adapter
                        .lock()
                        .unwrap()
                        .write(HashMap::from([(channel_id, chunk.to_vec())]));
                }
            }
        }
    }
}

impl Node {
    fn process_incoming_messages(
        &mut self,
        msgs: HashMap<PubSubChannelIDType, Vec<PubSubMessage>>,
    ) {
        for (channel_id, messages) in msgs {
            for message in messages {
                debug!("Processing incoming message: {:?}", message);
                self.process_incoming_message(channel_id, message);
            }
        }
    }
    fn process_incoming_message(
        &mut self,
        channel_id: PubSubChannelIDType,
        message: PubSubMessage,
    ) {
        match message {
            PubSubMessage::Register() => {
                debug!("[process_incoming_message]: Received register message for channel");

                let new_channel = self.new_channel();
                // Update with current subscribing topics
                for topic in self.sub_callbacks.keys() {
                    debug!("Re-adding topic subscription {:?} to new channel", topic);
                    let mut new_channel = new_channel.lock().unwrap();
                    new_channel.add_subscribe(topic.clone());
                }
            }
            PubSubMessage::Publish(msg) => {
                let datapoints = msg.messages;
                self.on_publish(channel_id, &datapoints);
            }
            PubSubMessage::Subscribe(msg) => {
                self.on_subscribe(channel_id, msg.topic.handle());
            }
            PubSubMessage::Welcome(id) => {
                debug!("Received welcome message for channel {}", id);
                let new_channel = self.new_channel_with_id(id);
                // Update with current subscribing topics
                for topic in self.sub_callbacks.keys() {
                    let mut new_channel = new_channel.lock().unwrap();
                    new_channel.add_subscribe(topic.clone());
                }
            }
            _ => {}
        }
    }

    fn send_message(&mut self, channel_id: PubSubChannelIDType, message: PubSubMessage) {
        let mut adapter = self.adapter.try_lock().unwrap();
        let mut msg_map = HashMap::new();
        msg_map.insert(channel_id, vec![message]);
        adapter.write(msg_map);
    }

    fn new_channel(&mut self) -> PubSubChannelHandle {
        let channel = PubSubChannel::new();
        let handle = channel.handle();
        let id = channel.id;
        debug!("[new_channel]: Created new channel with id {}", id);
        self.channels.insert(id, handle.clone());
        handle
    }

    fn new_channel_with_id(&mut self, id: PubSubChannelIDType) -> PubSubChannelHandle {
        let channel = PubSubChannel::new_with_id(id);
        let handle = channel.handle();
        self.channels.insert(id, handle.clone());
        handle
    }

    fn on_subscribe(&mut self, channel_id: PubSubChannelIDType, topic: TopicKeyHandle) {
        trace!(
            "New remote subscription {:?} on channel {}",
            topic,
            channel_id
        );

        {
            let mut datastore = self.datastore.lock().unwrap();
            let channel = self.channels.get_mut(&channel_id).unwrap();
            let mut channel = channel.lock().unwrap();

            datastore
                .add_listener(topic.key(), self.bucket_listener.clone())
                .expect("Failed to add listener");
            channel.on_subscribe(topic.clone());
        }

        // Send initial snapshot of all topics
        let datapoints = {
            let datastore = self.datastore.lock().unwrap();
            datastore.get_latest_datapoints(&topic).unwrap()
        };
        for datapoint in datapoints {
            self.on_datapoint(&datapoint);
        }
    }

    fn on_datapoint(&mut self, datapoint: &Datapoint) {
        trace!("Received updated datapoint: {:?}", datapoint.topic);
        for channel in self.channels.values_mut() {
            let mut channel = channel.lock().unwrap();
            channel.on_datapoint(datapoint);
        }
    }

    fn on_publish(&mut self, channel_id: PubSubChannelIDType, datapoints: &Vec<Datapoint>) {
        trace!("Received publish message for channel {}", channel_id);
        let channel = self.channels.get_mut(&channel_id).unwrap();
        let mut channel = channel.lock().unwrap();
        channel.on_publish(datapoints);
    }

    fn apply_datapoints(&mut self, datapoints: &Vec<Datapoint>) {
        if datapoints.len() > 0 {
            debug!("Applying {} datapoints", datapoints.len());
            self.datastore
                .lock()
                .unwrap()
                .add_datapoints(datapoints.clone());
        }
    }

    fn notify_sub_callbacks(&mut self, datapoints: &Vec<Datapoint>) {
        // Call all sub callbacks

        if datapoints.len() == 0 {
            return;
        }
        for (topic, callback) in self.sub_callbacks.iter() {
            let mut update_map = BTreeMap::new();

            for datapoint in datapoints {
                if datapoint.topic.is_child_of(topic) {
                    update_map.insert(datapoint.topic.clone(), datapoint.clone());
                } else {
                    debug!(
                        "Datapoint {:?} is not a child of topic {:?}",
                        datapoint.topic, topic
                    );
                }
            }
            if update_map.len() > 0 {
                let mut callback = callback.lock().unwrap();
                callback.on_update(&update_map);
                debug!("Notified sub callback for topic {:?}", topic);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::adapters::mock::MockPubSubAdapter;
    use crate::messages::SubscribeMessage;
    use test_log::test;
    use victory_data_store::primitives::Primitives;

    use super::*;

    use log::info;

    use victory_data_store::topics::TopicKey;
    use victory_wtf::Timepoint;

    use std::ops::Sub;
    use std::sync::mpsc::{channel, Sender};
    use std::sync::{Arc, Mutex};

    use std::vec;
    #[derive(Debug, Default)]
    struct TestSubCallback {
        msgs: Vec<DatapointMap>,
    }

    impl sub_callback::SubCallback for TestSubCallback {
        fn on_update(&mut self, datapoints: &DatapointMap) {
            info!("TestSubCallback::on_update: Received {:?}", datapoints);
            self.msgs.push(datapoints.clone());
        }
    }

    #[test]
    fn test_node_register() {
        let datastore = Arc::new(Mutex::new(Datastore::new()));
        let adapter = Arc::new(Mutex::new(MockPubSubAdapter::new()));
        let mut node = Node::new("test".to_string(), adapter.clone(), datastore.clone());

        // Send register message

        {
            let register_message = PubSubMessage::Register();
            let mut adapter = adapter.lock().unwrap();
            adapter.channel_write(0, vec![register_message]);
        }

        // Process incoming messages
        node.tick();

        // Check that the channel was created
        let channels = node.channels;
        assert!(channels.len() == 1, "1 New channel was created");
    }

    #[test]
    fn test_node_remote_subscribe() {
        let datastore = Arc::new(Mutex::new(Datastore::new()));
        let adapter = Arc::new(Mutex::new(MockPubSubAdapter::new()));
        let mut node = Node::new("test".to_string(), adapter.clone(), datastore.clone());

        // Send register message
        {
            let register_message = PubSubMessage::Register();
            let mut adapter = adapter.lock().unwrap();
            adapter.channel_write(0, vec![register_message]);
        }
        // Process incoming messages
        node.tick();

        // Check that the channel was created
        let channels = node.channels.clone();
        assert!(channels.len() == 1, "1 New channel was created");
        let channel = channels.values().next().unwrap();
        {
            let channel = channel.lock().unwrap();
            let recv_msgs = adapter.lock().unwrap().channel_read(channel.id);
            assert!(recv_msgs.len() == 1, "1 Welcome message was received");
        }

        // Send Subscribe message
        {
            let topic = TopicKey::from_str("test");
            let subscribe_message = PubSubMessage::Subscribe(SubscribeMessage::new(topic));
            let mut adapter = adapter.lock().unwrap();
            let channel = channel.lock().unwrap();
            adapter.channel_write(channel.id, vec![subscribe_message]);
        }

        // Process incoming messages
        node.tick();

        {
            let channel = channel.lock().unwrap();
            let topic = TopicKey::from_str("test");
            // Check that the topic is subscribed to
            assert!(
                channel.publishing_topics.contains(&topic),
                "Topic is subscribed to, {:?}",
                channel.subscribing_topics
            );
        }

        // Publish a datapoint
        {
            let topic = TopicKey::from_str("test");
            datastore
                .lock()
                .unwrap()
                .add_primitive(&topic, Timepoint::now(), Primitives::from(1));
        }

        // Process incoming messages
        node.tick();

        // Check that the channel received the datapoint
        {
            let topic = TopicKey::from_str("test");
            let channel = channel.lock().unwrap();
            let recv_msgs = adapter.lock().unwrap().channel_read(channel.id);
            assert!(
                recv_msgs.len() == 1,
                "1 Datapoint was received, {:?}",
                recv_msgs
            );

            let datapoint = recv_msgs.first().unwrap();
            let publish_message = match datapoint {
                PubSubMessage::Publish(msg) => msg,
                _ => panic!("Expected publish message"),
            };
            let datapoint = publish_message.messages.first().unwrap();
            assert_eq!(
                datapoint.topic,
                topic.handle(),
                "Datapoint topic is correct"
            );
            assert_eq!(
                datapoint.value,
                Primitives::from(1),
                "Datapoint value is correct"
            );
        }
    }

    #[test]
    fn test_node_publish() {
        let topic = TopicKey::from_str("test");
        let datastore = Arc::new(Mutex::new(Datastore::new()));
        let adapter = Arc::new(Mutex::new(MockPubSubAdapter::new()));
        let mut node = Node::new("test".to_string(), adapter.clone(), datastore.clone());

        // Send register message
        {
            let register_message = PubSubMessage::Register();
            let mut adapter = adapter.lock().unwrap();
            adapter.channel_write(0, vec![register_message]);
        }
        // Process incoming messages
        node.tick();

        // Check that the channel was created
        let channels = node.channels.clone();
        assert!(channels.len() == 1, "1 New channel was created");
        let channel = channels.values().next().unwrap();
        {
            let channel = channel.lock().unwrap();
            let recv_msgs = adapter.lock().unwrap().channel_read(channel.id);
            assert!(recv_msgs.len() == 1, "1 Welcome message was received");
        }

        // Locally subscribe to topic
        let sub_callback = TestSubCallback::default();
        let sub_callback_handle = Arc::new(Mutex::new(sub_callback));
        node.add_sub_callback(topic.clone(), sub_callback_handle.clone());

        // Publish a datapoint remotely
        {
            let datapoint = Datapoint::new(&topic, Timepoint::now(), Primitives::from(1));
            let publish_message = PublishMessage::single(datapoint);
            let publish_message = PubSubMessage::Publish(publish_message);
            let mut adapter = adapter.lock().unwrap();
            let channel = channel.lock().unwrap();
            adapter.channel_write(channel.id, vec![publish_message]);
        }

        // Process incoming messages
        node.tick();

        // Check that the sub callback received the datapoint
        let sub_callback = sub_callback_handle.lock().unwrap();
        assert!(sub_callback.msgs.len() == 1, "1 Datapoint was received");
        let datapoints = sub_callback.msgs.first().unwrap();
        let mut values = datapoints.values();
        assert!(values.len() == 1, "1 Datapoint was received");
        let value = values.next().unwrap();
        assert_eq!(
            value.value,
            Primitives::from(1),
            "Datapoint value is correct"
        );
        assert_eq!(value.topic, topic.handle(), "Datapoint topic is correct");
    }
}
