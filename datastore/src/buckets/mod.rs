use std::{collections::BTreeMap, sync::{Arc, RwLock}};

use log::debug;

use crate::{datapoints::Datapoint, primitives::{timestamp::{VicInstant, VicInstantHandle}, Primitives}, topics::TopicKeyHandle};
#[derive(Debug)]
pub struct Bucket{
    pub topic: TopicKeyHandle,
    pub values: BTreeMap<VicInstantHandle, Datapoint>
}

pub type BucketHandle = Arc<RwLock<Bucket>>;


impl Bucket{
    pub fn new(topic: TopicKeyHandle) -> BucketHandle{
        debug!("Creating new bucket for topic {:?}", topic);
        Arc::new(RwLock::new(Bucket{
            topic,
            values: BTreeMap::new()
        }))
    }

    pub fn add_primitive(&mut self, time: VicInstantHandle, value: Primitives){
       let data_point = Datapoint{
           topic: self.topic.clone(),
           time: time.clone(),
           value
       };

       self.values.insert(time, data_point);
    }

    pub fn add_datapoint(&mut self, data_point: Datapoint){
        self.values.insert(data_point.time.clone(), data_point);
    }

}


