use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::info;

use victory_data_store::{
    database::Datastore,
    datapoints::Datapoint,
    sync::{
        adapters::tcp::{tcp_client::TCPClient, tcp_server::TcpSyncServer},
        config::SyncConfig,
    },
    test_util::BigState,
    topics::TopicKey,
};
use victory_wtf::{Timepoint, Timespan};

pub struct TCPNodeSubscriber {}

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address to connect to
    #[arg(short, long, default_value = "0.0.0.0")]
    address: String,

    /// Port to connect to
    #[arg(short, long, default_value = "7000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let args = Args::parse();
    let connect_addr = format!("{}:{}", args.address, args.port);

    let client = TCPClient::new(connect_addr)
        .await
        .expect("Failed to create TCP client");
    let client_handle = Arc::new(Mutex::new(client));
    let datastore = Datastore::new().handle();

    let topic_filter = TopicKey::empty();

    let sync_config = SyncConfig {
        client_name: "TCP Sync Client".to_string(),
        subscriptions: vec![],
    };
    datastore
        .lock()
        .unwrap()
        .setup_sync(sync_config, client_handle);
    let topic = TopicKey::from_str("test");
    loop {
        tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
        datastore.lock().unwrap().run_sync();
        // Write a message to the topic
        let test_struct = BigState::new();
        datastore
            .lock()
            .unwrap()
            .add_struct(&topic, Timepoint::now(), test_struct);
    }
}
