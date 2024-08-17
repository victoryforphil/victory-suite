use datastore::{
    datapoints::Datapoint,
    primitives::{timestamp::VicInstantHandle, Primitives},
    topics::{TopicKeyHandle, TopicKeyProvider},
};
#[derive(Debug, Clone)]
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
