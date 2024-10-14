use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use log::{debug, trace};
use victory_time_rs::Timepoint;

use crate::{
    datapoints::Datapoint,
    primitives::Primitives,
    topics::{TopicKeyHandle, TopicKeyProvider},
};
#[derive(Debug)]
/// A bucket is a collection of datapoints for a specific topic
pub struct Bucket {
    pub topic: TopicKeyHandle,
    pub values: BTreeMap<Timepoint, Datapoint>,
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

    pub fn add_primitive(&mut self, time: Timepoint, value: Primitives) {
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

    pub fn get_values_after(&self, time: &Timepoint) -> Vec<&Primitives> {
        self.get_data_points_after(time)
            .iter()
            .map(|v| &v.value)
            .collect()
    }

    pub fn get_values_before(&self, time: &Timepoint) -> Vec<&Primitives> {
        self.get_data_points_before(time)
            .iter()
            .map(|v| &v.value)
            .collect()
    }

    pub fn get_updated_value(&self, time: &Timepoint) -> Option<&Primitives> {
        // Get the nearest value before the time
        let before = self
            .values
            .range(..time.clone())
            .last()
            .map(|(_, v)| &v.value);
        before.or_else(|| self.get_latest_value())
    }

    pub fn get_updated_datapoint(&self, time: &Timepoint) -> Option<&Datapoint> {
        // Get the nearest value before the time
        let before = self.values.range(..time.clone()).last().map(|(_, v)| v);
        before.or_else(|| self.get_latest_datapoint())
    }

    pub fn get_data_points_after(&self, time: &Timepoint) -> Vec<&Datapoint> {
        self.values.range(time.clone()..).map(|(_, v)| v).collect()
    }

    pub fn get_data_points_before(&self, time: &Timepoint) -> Vec<&Datapoint> {
        self.values.range(..time.clone()).map(|(_, v)| v).collect()
    }
}

#[cfg(test)]
mod tests {

    use victory_time_rs::{Timecode, Timepoint};

    use crate::{
        buckets::Bucket,
        datapoints::Datapoint,
        primitives::Primitives,
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

        let time = Timepoint::new(Timecode::new_secs(1.0));

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

        let time = Timepoint::new(Timecode::new_secs(1.0));

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
