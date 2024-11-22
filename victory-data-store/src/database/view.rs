use crate::{
    buckets::BucketHandle,
    primitives::{
        serde::{deserializer::PrimitiveDeserializer, serialize::to_map},
        Primitives,
    },
    topics::{TopicKey, TopicKeyHandle, TopicKeyProvider},
};

use log::warn;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

use super::{Datastore, DatastoreError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataView {
    pub maps: HashMap<TopicKey, Primitives>,
    bucket_cache: HashMap<TopicKeyHandle, Vec<BucketHandle>>,
}

impl Default for DataView {
    fn default() -> Self {
        Self::new()
    }
}

impl DataView {
    pub fn new() -> DataView {
        DataView {
            maps: HashMap::new(),
            bucket_cache: HashMap::new(),
        }
    }
    #[tracing::instrument(skip_all)]
    pub fn add_query(
        mut self,
        datastore: &mut Datastore,
        topic: &TopicKey,
    ) -> Result<DataView, DatastoreError> {
        let buckets = datastore.get_buckets_matching_cached(topic)?;
        for bucket in buckets {
            let bucket = bucket.read().unwrap();

            if let Some(value) = bucket.get_latest_datapoint() {
                let key = value.topic.key().clone();
                self.maps.insert(key, value.value.clone());
            }
        }

        Ok(self)
    }
    #[tracing::instrument(skip_all)]
    pub fn get_latest_map<T: TopicKeyProvider>(
        &self,
        topic: &T,
    ) -> Result<HashMap<TopicKey, Primitives>, DatastoreError> {
        let map = self
            .maps
            .iter()
            .filter_map(|(k, v)| {
                if k.key().is_child_of(topic.key()) {
                    Some((k.key().clone(), v.clone()))
                } else if k.key() == topic.key() {
                    Some((k.key().clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<TopicKey, Primitives>>();
        Ok(map)
    }
    #[tracing::instrument(skip_all)]
    pub fn get_latest<T: TopicKeyProvider, S: DeserializeOwned>(
        &self,
        topic: &T,
    ) -> Result<S, DatastoreError> {
        let value_map = self
            .maps
            .iter()
            .filter_map(|(k, v)| {
                if k.key().is_child_of(topic.key()) {
                    let key = k.key().remove_prefix(topic.key().clone()).unwrap();
                    Some((key.handle(), v.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<TopicKeyHandle, Primitives>>();

        // Deserialize the value map into the struct
        let mut deserializer = PrimitiveDeserializer::new(&value_map);
        let result = S::deserialize(&mut deserializer)
            .map_err(|e| DatastoreError::Generic(format!("Error deserializing struct: {:?}", e)));

        match result {
            Ok(s) => Ok(s),
            Err(e) => Err(DatastoreError::Generic(format!(
                "Error deserializing struct: {:?}",
                e
            ))),
        }
    }
    #[tracing::instrument(skip_all)]
    pub fn add_latest<T: TopicKeyProvider, S: Serialize>(
        &mut self,
        topic: &T,
        value: S,
    ) -> Result<(), DatastoreError> {
        let topic_key = topic.key().clone();
        let value_map = to_map(&value).unwrap();
        for (key, value) in value_map {
            let full_key = key.add_prefix(topic_key.clone());
            self.maps.insert(full_key, value);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use serde::{Deserialize, Serialize};
    use victory_wtf::Timepoint;

    use crate::{
        database::{view::DataView, Datastore},
        topics::TopicKey,
    };

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
    #[test]
    pub fn test_dataview_add_latest() {
        let topic: TopicKey = "/test/topic".into();

        let test_struct = TestStructA {
            a: 42,
            b: "test".to_string(),
        };
        let mut view = DataView::new();
        view.add_latest(&topic, test_struct.clone()).unwrap();
        let result: TestStructA = view.get_latest(&topic).unwrap();
        assert_eq!(result, test_struct);
    }
    #[test]
    pub fn test_datastore_view() {
        sensible_env_logger::safe_init!();

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

        let view = DataView::new()
            .add_query(&mut datastore, &topic_a)
            .unwrap()
            .add_query(&mut datastore, &topic_b)
            .unwrap();

        let result: TestStructA = view.get_latest(&topic_a).unwrap();

        assert_eq!(result, test_struct_a);

        let result: TestStructB = view.get_latest(&topic_b).unwrap();
        assert_eq!(result, test_struct_b);

        let mut new_datastore = Datastore::new();
        new_datastore.apply_view(view).unwrap();

        let result: TestStructA = new_datastore.get_struct(&topic_a).unwrap();
        assert_eq!(result, test_struct_a);

        let result: TestStructB = new_datastore.get_struct(&topic_b).unwrap();
        assert_eq!(result, test_struct_b);
    }
}
