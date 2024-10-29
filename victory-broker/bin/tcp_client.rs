use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{debug, info};
use victory_broker::{
    adapters::tcp::{TCPClientAdapter, TCPClientOptions},
    node::Node,
};
use victory_data_store::{database::Datastore, topics::TopicKey};
use victory_wtf::Timepoint;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let mut client = TCPClientAdapter::new(TCPClientOptions::from_url("0.0.0.0:7001")).await;

    while client.is_err() {
        info!("Failed to connect to server, retrying...");
        tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
        client = TCPClientAdapter::new(TCPClientOptions::from_url("0.0.0.0:7001")).await;
    }
    let client = client.unwrap();

    let client_handle = Arc::new(Mutex::new(client));

    let topic_key = TopicKey::from_str("test_topic");
    let datastore = Datastore::new().handle();
    let mut node = Node::new("TCP Client".to_string(), client_handle, datastore.clone());
    node.register();
    loop {
        tokio::time::sleep(Duration::from_secs_f32(0.25)).await;
        node.tick();
        datastore.lock().unwrap().add_primitive(
            &topic_key,
            Timepoint::now(),
            Timepoint::now().secs().into(),
        );
    }
}
