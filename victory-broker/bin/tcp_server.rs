use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::info;
use victory_broker::{
    adapters::tcp::{TCPServerAdapter, TCPServerOptions},
    node::{sub_callback::SubCallback, Node},
};
use victory_data_store::{database::Datastore, datapoints::Datapoint, topics::TopicKey};
use victory_wtf::Timespan;

pub struct TCPNodeSubscriber {}

impl SubCallback for TCPNodeSubscriber {
    fn on_update(&mut self, datapoints: &victory_data_store::datapoints::DatapointMap) {
        let mut table_string = String::new();
        for (topic, datapoint) in datapoints.iter() {
            table_string.push_str(&format!("\t {:?} -> {:?}\n", topic, datapoint.value));
        }
        info!("Received datapoints:\n{}", table_string);
    }
}
#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let server = TCPServerAdapter::new(TCPServerOptions {
        port: 7001,
        address: "0.0.0.0".to_string(),
        update_interval: Timespan::new_hz(50.0),
    });
    let server_handle = Arc::new(Mutex::new(server));
    let datastore = Datastore::new().handle();
    let mut pubsub = Node::new("TCP Server".to_string(), server_handle, datastore);
    let topic_key = TopicKey::from_str("test_topic");
    let subscriber = TCPNodeSubscriber {};
    let handle = Arc::new(Mutex::new(subscriber));

    pubsub.add_sub_callback(topic_key, handle);

    loop {
        tokio::time::sleep(Duration::from_secs_f32(0.25)).await;
        pubsub.tick();
    }
}
