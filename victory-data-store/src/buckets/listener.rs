use crate::{datapoints::Datapoint, topics::TopicKey};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

pub trait BucketListener: Debug + Send {
    fn on_datapoint(&mut self, datapoint: &Datapoint);
    fn get_filter(&self) -> Option<TopicKey> {
        None
    }
}

pub struct MockBucketListener {
    filter: Option<TopicKey>,
    pub updates: Vec<Datapoint>,
}

impl Default for MockBucketListener {
    fn default() -> Self {
        MockBucketListener {
            filter: None,
            updates: Vec::new(),
        }
    }
}

impl MockBucketListener {
    pub fn new(filter: Option<TopicKey>) -> Self {
        MockBucketListener {
            filter,
            updates: Vec::new(),
        }
    }

    pub fn as_handle(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }
}

impl Debug for MockBucketListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockBucketListener")
    }
}

impl BucketListener for MockBucketListener {
    fn on_datapoint(&mut self, datapoint: &Datapoint) {
        self.updates.push(datapoint.clone());
    }

    fn get_filter(&self) -> Option<TopicKey> {
        self.filter.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
