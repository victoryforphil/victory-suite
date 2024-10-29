use std::fmt::Debug;

use log::debug;
use victory_data_store::{buckets::listener::BucketListener, datapoints::Datapoint};

pub struct NodeBucketListener {
    msgs: Vec<Datapoint>,
}
impl Debug for NodeBucketListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeBucketListener")
    }
}
impl BucketListener for NodeBucketListener {
    fn on_datapoint(&mut self, datapoint: &Datapoint) {
        debug!("NodeBucketListener received datapoint: {}", datapoint.topic);
        self.msgs.push(datapoint.clone());
    }
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
