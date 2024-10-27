use super::PubSubMessage;
use serde::{Deserialize, Serialize};
use victory_data_store::topics::TopicKey;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeMessage {
    pub topic: TopicKey,
}

impl SubscribeMessage {
    pub fn new(topic: TopicKey) -> Self {
        SubscribeMessage { topic }
    }
}

impl From<SubscribeMessage> for PubSubMessage {
    fn from(message: SubscribeMessage) -> Self {
        PubSubMessage::Subscribe(message)
    }
}

impl From<PubSubMessage> for SubscribeMessage {
    fn from(message: PubSubMessage) -> Self {
        match message {
            PubSubMessage::Subscribe(subscribe) => subscribe,
            _ => panic!("Invalid conversion from PubSubMessage to SubscribeMessage"),
        }
    }
}

#[cfg(test)]
mod tests {
    use victory_data_store::topics::TopicKey;

    use super::SubscribeMessage;

    #[test]
    fn test_subscribe_message() {
        let topic = TopicKey::from_str("test_topic");
        let message = SubscribeMessage::new(topic.clone());
        assert_eq!(message.topic, topic);
    }
}
