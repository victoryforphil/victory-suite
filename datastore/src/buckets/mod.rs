use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use log::{debug, trace};

use crate::{
    datapoints::Datapoint,
    primitives::{timestamp::VicInstantHandle, Primitives},
    topics::{TopicKeyHandle, TopicKeyProvider},
};
#[derive(Debug)]
/// A bucket is a collection of datapoints for a specific topic
pub struct Bucket {
    pub topic: TopicKeyHandle,
    pub values: BTreeMap<VicInstantHandle, Datapoint>,
}

pub type BucketHandle = Arc<RwLock<Bucket>>;

impl Bucket {
    pub fn new<T: TopicKeyProvider>(topic: &T) -> BucketHandle {
        debug!("Creating new bucket for topic {:?}", topic.key());
        Arc::new(RwLock::new(Bucket {
            topic: topic.handle(),
            values: BTreeMap::new(),
        }))
    }

    pub fn add_primitive(&mut self, time: VicInstantHandle, value: Primitives) {
        let data_point = Datapoint {
            topic: self.topic.clone(),
            time: time.clone(),
            value,
        };

        self.add_datapoint(data_point);
    }

    pub fn add_datapoint(&mut self, data_point: Datapoint) {
        trace!("Adding data point {:?}", data_point);
        self.values.insert(data_point.time.clone(), data_point);
    }

    pub fn get_latest_datapoint(&self) -> Option<&Datapoint> {
        self.values.iter().last().map(|(_, v)| v)
    }

    pub fn get_datapoints_ref(&self) -> Vec<&Datapoint> {
        self.values.values().collect()
    }

    pub fn get_datapoints(&self) -> Vec<Datapoint> {
        self.values.values().cloned().collect()
    }

    pub fn get_latest_value(&self) -> Option<&Primitives> {
        self.get_latest_datapoint().map(|d| &d.value)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        buckets::Bucket,
        datapoints::Datapoint,
        primitives::{
            timestamp::{VicInstant, VicTimecode},
            Primitives,
        },
        topics::{TopicKey, TopicKeyProvider},
    };

    #[test]
    fn test_bucket_creation() {
        let topic = TopicKey::from_str("test/topic").handle();
        let bucket = Bucket::new(&topic);
        assert_eq!(bucket.read().unwrap().values.len(), 0);
        assert_eq!(bucket.read().unwrap().topic, topic.handle());
    }

    #[test]
    fn test_bucket_read_write_primitives() {
        let topic = TopicKey::from_str("test/topic");
        let bucket = Bucket::new(&topic);

        let time = VicInstant::new(VicTimecode::new_secs(1.0)).handle();

        let test_data = Primitives::Text("Test String value".to_string());

        bucket
            .write()
            .unwrap()
            .add_primitive(time.clone(), test_data.clone());

        //Test read using Latest Value
        {
            let bucket_read = bucket.read().unwrap();
            let latest = bucket_read.get_latest_value().unwrap();
            assert_eq!(
                latest, &test_data,
                "Latest value is not the same as the test data"
            );
        }
    }

    #[test]
    fn test_bucket_read_write_datapoints() {
        let topic = TopicKey::from_str("test/topic");
        let bucket = Bucket::new(&topic);

        let time = VicInstant::new(VicTimecode::new_secs(1.0)).handle();

        let test_data = Primitives::Text("Test String value".to_string());

        let data_point = Datapoint {
            topic: topic.handle(),
            time: time.clone(),
            value: test_data.clone(),
        };

        bucket.write().unwrap().add_datapoint(data_point.clone());

        //Test read using Latest Value
        {
            let bucket_read = bucket.read().unwrap();
            let latest = bucket_read.get_latest_datapoint().unwrap();
            assert_eq!(
                &data_point.topic, &latest.topic,
                "Latest value is not the same as the test data"
            );
            assert_eq!(
                &data_point.time, &latest.time,
                "Latest value is not the same as the test data"
            );
            assert_eq!(
                &data_point.value, &latest.value,
                "Latest value is not the same as the test data"
            );
        }

        //Test read using all data points
        {
            let bucket_read = bucket.read().unwrap();
            let values = bucket_read.get_datapoints_ref();
            assert_eq!(
                values.len(),
                1,
                "There should be only one value in the bucket"
            );
            assert_eq!(
                values[0].value, test_data,
                "The value in the bucket is not the same as the test data"
            );

            let values = bucket_read.get_datapoints();
            assert_eq!(
                values.len(),
                1,
                "There should be only one value in the bucket"
            );

            assert_eq!(
                values[0].value, test_data,
                "The value in the bucket is not the same as the test data"
            );
        }
    }
}
