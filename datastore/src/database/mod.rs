use std::collections::BTreeMap;

use crate::{
    buckets::{Bucket, BucketHandle},
    primitives::{timestamp::VicInstant, Primitives},
    topics::{TopicKey, TopicKeyHandle},
};
#[derive(Debug, Clone)]
pub struct Datastore {
    buckets: BTreeMap<TopicKeyHandle, BucketHandle>,
}

impl Datastore {
    pub fn new() -> Datastore {
        Datastore {
            buckets: BTreeMap::new(),
        }
    }

    pub fn add_primitive(&mut self, topic: TopicKey, time: VicInstant, value: Primitives) {
        let topic = topic.handle();
        let time = time.handle();

        if !self.buckets.contains_key(&topic) {
            let bucket = Bucket::new(topic.clone());
            self.buckets.insert(topic.clone(), bucket);
        }

        let bucket = self.buckets.get_mut(&topic).unwrap();

        bucket.write().unwrap().add_primitive(time, value);
    }
}
