use crate::{
    buckets::{Bucket, BucketHandle},
    datapoints::Datapoint,
    primitives::{
        serde::{deserializer::PrimitiveDeserializer, serialize::to_map},
        Primitives,
    },
    topics::{TopicKey, TopicKeyHandle, TopicKeyProvider},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use victory_time_rs::Timepoint;

#[derive(Debug, Clone)]
pub struct Datastore {
    buckets: BTreeMap<TopicKeyHandle, BucketHandle>,
}

pub struct DataView {
    pub buckets: Vec<BucketHandle>,
}

impl DataView {
    pub fn new() -> DataView {
        DataView {
            buckets: Vec::new(),
        }
    }
    pub fn add_query(mut self,datastore: &Datastore, query: &TopicKey) -> Result<DataView, DatastoreError> {
        let buckets = datastore.get_buckets_matching(query)?;
        self.buckets.extend(buckets);
        Ok(self)
    }
    pub fn get_latest<T: TopicKeyProvider, S: DeserializeOwned>(
        &self,
        topic: &T,
    ) -> Result<S, DatastoreError> {
        let mut value_map: BTreeMap<String, Primitives> = BTreeMap::new();
        for bucket in self.buckets.iter() {
            let bucket = bucket.read().unwrap();
            if !bucket.topic.key().is_child_of(topic.key()) {
                continue;
            }

            if let Some(value) = bucket.get_latest_value() {
                let bucket_topic = bucket.topic.key().display_name();
                let parent_topic = topic.key().display_name() + "/";

                // Add a trailing/
                let key = format!("{}/", bucket_topic.replace(&parent_topic, ""));
                value_map.insert(key.to_string(), value.clone());
            }
        }
        // Deserialize the value map into the struct
        let mut deserializer = PrimitiveDeserializer::new(&value_map);
        let result = match Deserialize::deserialize(&mut deserializer) {
            Ok(s) => Ok(s),
            Err(e) => Err(DatastoreError::Generic(format!(
                "Error deserializing struct: {:?}",
                e
            ))),
        };

        match result {
            Ok(s) => Ok(s),
            Err(e) => Err(DatastoreError::Generic(format!(
                "Error deserializing struct: {:?}",
                e
            ))),
        }
    }
}
#[derive(Error, Debug)]
pub enum DatastoreError {
    #[error("Generic Datastore Error: {0}")]
    Generic(String),
    #[error("Bucket not found for topic {0}")]
    BucketNotFound(TopicKey),
}

impl Datastore {
    pub fn new() -> Datastore {
        Datastore {
            buckets: BTreeMap::new(),
        }
    }

    pub fn create_bucket<T: TopicKeyProvider>(&mut self, topic: &T) {
        if !self.buckets.contains_key(&topic.handle()) {
            let bucket = Bucket::new(topic);
            self.buckets.insert(topic.handle().clone(), bucket);
        }
    }

    pub fn get_bucket<T: TopicKeyProvider>(
        &self,
        topic: &T,
    ) -> Result<BucketHandle, DatastoreError> {
        self.buckets
            .get(&topic.handle())
            .cloned()
            .ok_or_else(|| DatastoreError::BucketNotFound(topic.key().clone()))
    }

    pub fn get_buckets_matching<T: TopicKeyProvider>(
        &self,
        parent_topic: &T,
    ) -> Result<Vec<BucketHandle>, DatastoreError> {
        Ok(self
            .buckets
            .iter()
            .filter_map(|(k, v)| {
                if k.key().is_child_of(parent_topic.key()) {
                    Some(v.clone())
                } else if k.key() == parent_topic.key() {
                    Some(v.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<BucketHandle>>())
    }

    pub fn get_struct<T, S>(&self, topic: &T) -> Result<S, DatastoreError>
    where
        T: TopicKeyProvider,
        S: DeserializeOwned,
    {
        // Get all the buckets that match the topic
        let buckets = self.get_buckets_matching(topic)?;

        let mut value_map: BTreeMap<String, Primitives> = BTreeMap::new();
        for bucket in buckets {
            let bucket = bucket.read().unwrap();

            if let Some(value) = bucket.get_latest_value() {
                let bucket_topic = bucket.topic.key().display_name();
                let parent_topic = topic.key().display_name() + "/";

                // Add a trailing/
                let key = format!("{}/", bucket_topic.replace(&parent_topic, ""));
                value_map.insert(key.to_string(), value.clone());
            }
        }
        // Deserialize the value map into the struct
        let mut deserializer = PrimitiveDeserializer::new(&value_map);
        let result = match Deserialize::deserialize(&mut deserializer) {
            Ok(s) => Ok(s),
            Err(e) => Err(DatastoreError::Generic(format!(
                "Error deserializing struct: {:?}",
                e
            ))),
        };

        match result {
            Ok(s) => Ok(s),
            Err(e) => Err(DatastoreError::Generic(format!(
                "Error deserializing struct: {:?}",
                e
            ))),
        }
    }

    pub fn add_struct<T: TopicKeyProvider, S: Serialize>(
        &mut self,
        topic: &T,
        time: Timepoint,
        value: S,
    ) -> Result<(), DatastoreError> {
        let topic = topic.handle();
        let value_map = to_map(&value)
            .map_err(|e| DatastoreError::Generic(format!("Error serializing struct: {:?}", e)))?;

        for (key, value) in value_map {
            let key = TopicKey::from_str(&key);
            let full_key = key.add_prefix(topic.key().to_owned());
            // Remove the leading slash
            self.add_primitive(&full_key, time.clone(), value);
        }
        Ok(())
    }

    pub fn add_primitive<T: TopicKeyProvider>(
        &mut self,
        topic: &T,
        time: Timepoint,
        value: Primitives,
    ) {
        let topic = topic.handle();
        let time = time.clone();

        if !self.buckets.contains_key(&topic) {
            let bucket = Bucket::new(&topic);
            self.buckets.insert(topic.clone(), bucket);
        }

        let bucket = self.buckets.get_mut(&topic).unwrap();

        bucket.write().unwrap().add_primitive(time, value);
    }

    pub fn get_latest_primitive<T: TopicKeyProvider>(&self, topic: &T) -> Option<Primitives> {
        let topic = topic.handle();
        self.buckets
            .get(&topic)
            .and_then(|b| b.read().unwrap().get_latest_value().cloned())
    }

    pub fn get_latest_primitives<T: TopicKeyProvider>(
        &self,
        topics: BTreeSet<T>,
    ) -> BTreeMap<TopicKey, Primitives> {
        topics
            .iter()
            .filter_map(|t| self.get_latest_primitive(t).map(|p| (t.key().clone(), p)))
            .collect()
    }

    pub fn get_datapoints<T: TopicKeyProvider>(
        &self,
        topic: &T,
    ) -> Result<Vec<Datapoint>, DatastoreError> {
        let topic = topic.handle();
        let buckets = self.get_buckets_matching(&topic)?;
        let mut datapoints = Vec::new();
        for bucket in buckets {
            let bucket = bucket.read().unwrap();
            datapoints.extend(bucket.get_datapoints());
        }

        Ok(datapoints)
    }
    pub fn get_all_keys(&self) -> Vec<TopicKeyHandle> {
        self.buckets.keys().cloned().collect()
    }

    pub fn get_all_display_names(&self) -> BTreeSet<String> {
        self.buckets
            .keys()
            .map(|k| k.key().display_name())
            .collect()
    }
    pub fn get_updated_keys<T: TopicKeyProvider>(
        &self,
        topic: &T,
        time: &Timepoint,
    ) -> Result<Vec<TopicKeyHandle>, DatastoreError> {
        let topic = topic.handle();
        let bucket = self.get_bucket(&topic)?;
        let bucket = bucket.read().unwrap();
        let new_values = bucket.get_data_points_after(&time.clone());
        Ok(new_values.iter().map(|v| v.topic.handle()).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::database::*;

    #[test]
    pub fn test_datastore_creation() {
        let datastore = Datastore::new();
        assert_eq!(datastore.buckets.len(), 0);
    }

    #[test]
    pub fn test_datastore_create_bucket() {
        let mut datastore = Datastore::new();
        let topic = TopicKey::from_str("test/topic");
        datastore.create_bucket(&topic);
        assert_eq!(datastore.buckets.len(), 1);
        assert!(datastore.buckets.contains_key(&topic.handle()));
    }

    #[test]
    pub fn test_datastore_get_bucket() {
        let mut datastore = Datastore::new();
        let topic = TopicKey::from_str("test/topic");

        let bucket_failed = datastore.get_bucket(&topic);
        assert_eq!(bucket_failed.is_err(), true);

        datastore.create_bucket(&topic);

        let bucket = datastore.get_bucket(&topic);
        assert_eq!(bucket.is_ok(), true);
        assert_eq!(bucket.unwrap().read().unwrap().topic, topic.handle());
    }

    #[test]
    pub fn test_datastore_get_buckets_matching() {
        let mut datastore = Datastore::new();

        let topic_parent = TopicKey::from_str("test/topic");
        let topic_child_a = TopicKey::from_str("test/topic/b");
        let topic_child_b = TopicKey::from_str("test/topic/a");
        let topic_other = TopicKey::from_str("other/topic");
        let topic_other2 = TopicKey::from_str("test/other");
        let topic_child_a1 = TopicKey::from_str("test/topic/a/1");

        datastore.create_bucket(&topic_parent);
        datastore.create_bucket(&topic_child_a);
        datastore.create_bucket(&topic_child_b);
        datastore.create_bucket(&topic_other);
        datastore.create_bucket(&topic_other2);
        datastore.create_bucket(&topic_child_a1);

        let buckets = datastore.get_buckets_matching(&topic_parent).unwrap();
        assert_eq!(buckets.len(), 4);
        for bucket in &buckets {
            let bucket = bucket.read().unwrap();
            if bucket.topic.key() == &topic_parent {
                continue;
            }
            assert!(
                bucket.topic.is_child_of(&topic_parent),
                "Bucket {:?} is not a child of {:?}",
                bucket.topic,
                topic_parent
            );
        }

        let keys = buckets
            .iter()
            .map(|b| b.read().unwrap().topic.key().clone())
            .collect::<Vec<TopicKey>>();
        assert!(keys.contains(&topic_child_a));
        assert!(keys.contains(&topic_child_b));
        assert!(keys.contains(&topic_child_a1));
        assert!(!keys.contains(&topic_other));
        assert!(!keys.contains(&topic_other2));
        assert!(keys.contains(&topic_parent));
    }

    #[test]
    pub fn test_datastore_add_primitive() {
        let mut datastore = Datastore::new();
        let topic: TopicKey = "test/topic".into();
        let time = Timepoint::now();
        datastore.add_primitive(&topic, time.clone(), 42.into());

        let bucket = datastore.get_bucket(&topic).unwrap();
        let bucket = bucket.read().unwrap();
        let datapoints = bucket.get_datapoints();
        assert_eq!(datapoints.len(), 1);
        assert_eq!(datapoints[0].time, time.clone().into());
        assert_eq!(datapoints[0].value, 42.into());
    }

    #[test]
    pub fn test_datastore_get_set_struct() {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
        struct TestStruct {
            a: i32,
            b: String,
        }

        let mut datastore = Datastore::new();
        let topic: TopicKey = "/test/topic".into();
        let time = Timepoint::now();
        let test_struct = TestStruct {
            a: 42,
            b: "test".to_string(),
        };

        datastore
            .add_struct(&topic, time.clone(), test_struct.clone())
            .unwrap();
        // Log datastore keys

        let result: TestStruct = datastore.get_struct(&topic).unwrap();
        assert_eq!(result, test_struct);
    }

    #[test]
    pub fn test_datastore_view() {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
        struct TestStructA {
            a: i32,
            b: String,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
        struct TestStructB {
            c: i32,
            d: String,
        }

        let mut datastore = Datastore::new();
        let topic_a: TopicKey = "/test/a".into();
        let topic_b: TopicKey = "/test/b".into();
        let time = Timepoint::now();
        let test_struct_a = TestStructA {
            a: 42,
            b: "test".to_string(),
        };
        let test_struct_b = TestStructB {
            c: 42,
            d: "test".to_string(),
        };
        datastore
            .add_struct(&topic_a, time.clone(), test_struct_a.clone())
            .unwrap();
        datastore
            .add_struct(&topic_b, time.clone(), test_struct_b.clone())
            .unwrap();

        let view = DataView::new().add_query(&datastore, &topic_a).unwrap().add_query(&datastore, &topic_b).unwrap();

        let result: TestStructA = view.get_latest(&topic_a).unwrap();
        assert_eq!(result, test_struct_a);

        let result: TestStructB = view.get_latest(&topic_b).unwrap();
        assert_eq!(result, test_struct_b);
    }
}
