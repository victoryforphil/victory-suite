use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crate::{
    primitives::Primitives,
    time::VicInstantHandle,
    topics::{TopicKeyHandle, TopicKeyProvider},
};
#[derive(Debug, Clone)]
pub struct Datapoint {
    pub topic: TopicKeyHandle,
    pub time: VicInstantHandle,
    pub value: Primitives,
}

pub type DatapointMap = BTreeMap<TopicKeyHandle, Datapoint>;

pub type DatapointHandle = Arc<RwLock<Datapoint>>;

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
        primitives::Primitives,
        time::VicInstant,
        topics::{TopicKey, TopicKeyProvider},
    };
    #[test]
    fn test_datapoint() {
        let topic = TopicKey::from_str("test_topic");
        let now = VicInstant::now();
        let time = now;
        let value = Primitives::Integer(42);
        let datapoint = Datapoint::new(&topic, time.handle(), value);
        assert_eq!(datapoint.topic.key(), &TopicKey::from_str("test_topic"));
        assert_eq!(datapoint.value, 42.into());
    }
}
