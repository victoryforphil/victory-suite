use super::PubSubMessage;
use serde::{Deserialize, Serialize};
use victory_data_store::{
    datapoints::Datapoint,
    primitives::Primitives,
    topics::{TopicKeyHandle, TopicKeyProvider},
};
use victory_wtf::Timepoint;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishMessage {
    pub messages: Vec<Datapoint>,
}

impl PublishMessage {
    pub fn new(messages: Vec<Datapoint>) -> Self {
        PublishMessage { messages }
    }
    pub fn single(message: Datapoint) -> Self {
        PublishMessage {
            messages: vec![message],
        }
    }

    pub fn primitive<T: TopicKeyProvider>(topic: &T, time: Timepoint, value: Primitives) -> Self {
        PublishMessage {
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
