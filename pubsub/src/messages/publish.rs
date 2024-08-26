use datastore::{
    datapoints::Datapoint,
    primitives::Primitives,
    time::VicInstantHandle,
    topics::{TopicKeyHandle, TopicKeyProvider},
};

use super::PubSubMessage;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishMessage {
    pub topic: TopicKeyHandle,
    pub messages: Vec<Datapoint>,
}

impl PublishMessage {
    pub fn new<T: TopicKeyProvider>(topic: &T, messages: Vec<Datapoint>) -> Self {
        PublishMessage {
            topic: topic.handle(),
            messages,
        }
    }
    pub fn single(message: Datapoint) -> Self {
        PublishMessage {
            topic: message.topic.clone(),
            messages: vec![message],
        }
    }

    pub fn primitive<T: TopicKeyProvider>(
        topic: &T,
        time: VicInstantHandle,
        value: Primitives,
    ) -> Self {
        PublishMessage {
            topic: topic.handle(),
            messages: vec![Datapoint::new(topic, time, value)],
        }
    }
}

impl From<PubSubMessage> for PublishMessage {
    fn from(msg: PubSubMessage) -> Self {
        match msg {
            PubSubMessage::Publish(publish) => publish,
            _ => panic!("Invalid conversion from PubSubMessage to PublishMessage"),
        }
    }
}

impl From<PublishMessage> for PubSubMessage {
    fn from(msg: PublishMessage) -> Self {
        PubSubMessage::Publish(msg)
    }
}
