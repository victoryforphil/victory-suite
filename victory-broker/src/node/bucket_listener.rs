use std::fmt::Debug;

use log::{debug, warn};
use victory_data_store::{database::listener::DataStoreListener, datapoints::Datapoint};

pub struct NodeBucketListener {
    msgs: Vec<Datapoint>,
}
impl Debug for NodeBucketListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeBucketListener")
    }
}
impl DataStoreListener for NodeBucketListener {
    fn on_datapoint(&mut self, datapoint: &Datapoint) {
        debug!("NodeBucketListener received datapoint: {}", datapoint.topic);
        self.msgs.push(datapoint.clone());

        // if over 2048, drop older 512
        if self.msgs.len() > 2048 {
            warn!("NodeBucketListener queue overflow, dropping 512 datapoints");
            self.msgs.drain(..512);
        }
    }

    fn on_bucket_update(&mut self, bucket: &victory_data_store::buckets::BucketHandle) {}
}

impl NodeBucketListener {
    pub fn new() -> Self {
        NodeBucketListener { msgs: Vec::new() }
    }

    pub fn drain(&mut self) -> Vec<Datapoint> {
        self.msgs.drain(..).collect()
    }

    pub fn get_n_latest(&mut self, n: usize) -> Vec<Datapoint> {
        self.msgs.drain(..n).collect()
    }
}
