use std::{
    collections::HashMap, sync::{Arc, Mutex}, thread, time::Duration
};

use log::{debug, info};
use victory_broker::{
    adapters::tcp::{TCPClientAdapter, TCPClientOptions},
    node::Node,
};
use victory_data_store::{database::Datastore, test_util::BigState, topics::TopicKey};
use victory_wtf::Timepoint;


fn main() {
    pretty_env_logger::init();
    let mut client = TCPClientAdapter::new(TCPClientOptions::from_url("0.0.0.0:7001"));

    while client.is_err() {
        info!("Failed to connect to server, retrying...");
        thread::sleep(Duration::from_secs_f32(1.0));
        client = TCPClientAdapter::new(TCPClientOptions::from_url("0.0.0.0:7001"));
    }
    let client = client.unwrap();

    let client_handle = Arc::new(Mutex::new(client));

    let topic_key = TopicKey::from_str("test_topic");
    let datastore = Datastore::new().handle();
    let mut node = Node::new("TCP Client".to_string(), client_handle, datastore.clone());
    node.register();

    // Spawn test send thread
    let datastore = datastore.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs_f32(0.5));
            let big_struct = BigState::new();
            datastore.lock().unwrap().add_struct(
                &topic_key,
                Timepoint::now(),
                big_struct,
            );

        }
    });

    loop {
        thread::sleep(Duration::from_secs_f32(0.01));
        node.tick();
       
    }
}
