use crate::{
    primitives::{timestamp::VicInstantHandle, Primitives},
    topics::{TopicKeyHandle, TopicKeyProvider},
};
#[derive(Debug, Clone)]
pub struct Datapoint {
    pub topic: TopicKeyHandle,
    pub time: VicInstantHandle,
    pub value: Primitives,
}

impl Datapoint {
    pub fn new<T: TopicKeyProvider>(
        topic: &T,
        time: VicInstantHandle,
        value: Primitives,
    ) -> Datapoint {
        Datapoint {
            topic: topic.handle().clone(),
            time,
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        datapoints::Datapoint,
        primitives::{timestamp::VicInstant, Primitives},
        topics::{TopicKey, TopicKeyProvider},
    };
    #[test]
    fn test_datapoint() {
        let topic = TopicKey::from_str("test_topic");
        let time = VicInstant::now();
        let value = Primitives::Integer(42);
        let datapoint = Datapoint::new(&topic, time.handle(), value);
        assert_eq!(datapoint.topic.key(), &TopicKey::from_str("test_topic"));
        assert_eq!(datapoint.value, 42.into());
    }
}
