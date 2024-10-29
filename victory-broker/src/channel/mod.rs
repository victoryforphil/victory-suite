use std::{
    collections::{BTreeSet, HashMap},
    sync::{Arc, Mutex},
    vec,
};

use log::{debug, info, trace};
use thiserror::Error;
use victory_data_store::{
    buckets::BucketHandle,
    database::Datastore,
    datapoints::Datapoint,
    topics::{TopicKeyHandle, TopicKeyProvider},
};

use crate::{
    messages::{PubSubMessage, PublishMessage, SubscribeMessage},
    MutexType,
};

pub type PubSubChannelIDType = u16;

#[derive(Clone)]
pub struct PubSubChannel {
    pub id: PubSubChannelIDType,
    pub publishing_topics: BTreeSet<TopicKeyHandle>,
    pub subscribing_topics: BTreeSet<TopicKeyHandle>,

    send_queue: HashMap<PubSubChannelIDType, Vec<PubSubMessage>>,
    recv_queue: Vec<Datapoint>,
}

pub type PubSubChannelHandle = MutexType<PubSubChannel>;

#[derive(Error, Debug)]
pub enum PubSubChannelError {
    #[error("Generic PubSubChannel Error: {0}")]
    Generic(String),
    #[error("Failed to create new PubSubChannel")]
    CreateFailed,
}
impl PubSubChannel {
    pub fn new() -> Self {
        let id = rand::random::<PubSubChannelIDType>();

        info!("Created new PubSubChannel {}", id);
        let mut channel = PubSubChannel {
            id,
            publishing_topics: BTreeSet::new(),
            subscribing_topics: BTreeSet::new(),
            send_queue: HashMap::new(),
            recv_queue: Vec::new(),
        };
        channel.send_welcome_message();
        channel
    }

    pub fn new_with_id(id: PubSubChannelIDType) -> Self {
        info!("Created new PubSubChannel {}", id);
        let mut channel = PubSubChannel {
            id,
            publishing_topics: BTreeSet::new(),
            subscribing_topics: BTreeSet::new(),
            send_queue: HashMap::new(),
            recv_queue: Vec::new(),
        };
        channel
    }
    pub fn handle(&self) -> PubSubChannelHandle {
        Arc::new(Mutex::new(self.clone()))
    }

    pub fn drain_send_queue(&mut self) -> HashMap<PubSubChannelIDType, Vec<PubSubMessage>> {
        let mut to_send = HashMap::new();
        for (channel_id, messages) in self.send_queue.iter_mut() {
            to_send.insert(*channel_id, messages.drain(..).take(16).collect());
        }
        to_send
    }

    pub fn drain_recv_queue(&mut self) -> Vec<Datapoint> {
        self.recv_queue.drain(..).collect()
    }

    pub fn on_datapoint(&mut self, datapoint: &Datapoint) {
        // Check to see if we shuld publish this topic
        let topic = datapoint.topic.clone();

        let mut publish = false;
        for pub_topic in &self.publishing_topics {
            if pub_topic.is_parent_of(&topic) {
                publish = true;
                break;
            }

            if topic.is_parent_of(pub_topic) {
                publish = true;
                break;
            }

            if topic == pub_topic.clone() {
                publish = true;
                break;
            }
        }
        if publish {
            debug!(
                "Channel #{} publishing datapoint {:?}",
                self.id, datapoint.topic
            );
            self.send_publish_message(datapoint.clone());
        } else {
            debug!(
                "Channel #{} received datapoint {:?}, but is not publishing it",
                self.id, datapoint.topic
            );
        }
    }

    pub fn add_subscribe(&mut self, topic: TopicKeyHandle) {
        info!("Channel #{} subscribing to topic {}", self.id, topic);
        self.subscribing_topics.insert(topic.clone());
        self.send_subscribe_message(topic.clone());
    }

    pub fn on_publish(&mut self, datapoints: &Vec<Datapoint>) {
        let mut new_datapoints = Vec::new();

        for datapoint in datapoints {
            let mut update_topic = false;
            for sub_topic in &self.subscribing_topics {
                if sub_topic.is_child_of(&datapoint.topic)
                    || sub_topic == &datapoint.topic
                    || datapoint.topic.is_child_of(sub_topic)
                {
                    update_topic = true;
                    break;
                }
            }
            if update_topic {
                debug!(
                    "Channel #{} received publish update for {:?}",
                    self.id, datapoint.topic
                );

                new_datapoints.push(datapoint.clone());
            }
        }
        self.recv_queue.append(&mut new_datapoints);
    }

    pub fn on_subscribe(&mut self, topic: TopicKeyHandle) {
        info!(
            "Channel #{} got subscribe, will publish: {:?}",
            self.id, topic
        );
        self.publishing_topics.insert(topic);
    }
}

impl PubSubChannel {
    fn send_welcome_message(&mut self) {
        let welcome_message = PubSubMessage::Welcome(self.id);
        self.send_queue
            .entry(self.id)
            .or_insert(Vec::new())
            .push(welcome_message);
        debug!("Added welcome message to channel send queue {}", self.id);
    }

    fn send_publish_message(&mut self, datapoint: Datapoint) {
        debug!(
            "Channel #{} sending publish message for {:?}",
            self.id, datapoint.topic
        );
        let publish_message = PubSubMessage::Publish(PublishMessage::single(datapoint));
        self.send_queue
            .entry(self.id)
            .or_insert(Vec::new())
            .push(publish_message);
    }

    fn send_subscribe_message(&mut self, topic: TopicKeyHandle) {
        let subscribe_message = PubSubMessage::Subscribe(SubscribeMessage {
            topic: topic.key().clone(),
        });
        self.send_queue
            .entry(self.id)
            .or_insert(Vec::new())
            .push(subscribe_message);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_pubsub_channel() {}
}
