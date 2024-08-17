use datastore::{
    datapoints::Datapoint,
    primitives::{timestamp::VicInstantHandle, Primitives},
    topics::{TopicKeyHandle, TopicKeyProvider},
};
#[derive(Debug, Clone)]
pub struct UpdateMessage {
    pub topic: TopicKeyHandle,
    pub messages: Datapoint,
}

impl UpdateMessage {
    pub fn new(message: Datapoint) -> Self {
        UpdateMessage {
            topic: message.topic.clone(),
            messages: message,
        }
    }

    pub fn primitive<T: TopicKeyProvider>(
        topic: &T,
        time: VicInstantHandle,
        value: Primitives,
    ) -> Self {
        UpdateMessage {
            topic: topic.handle(),
            messages: Datapoint::new(topic, time, value),
        }
    }
}
