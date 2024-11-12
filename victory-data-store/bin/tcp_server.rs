use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::info;

use victory_data_store::{
    database::Datastore,
    datapoints::Datapoint,
    sync::{adapters::tcp::tcp_server::TcpSyncServer, config::SyncConfig},
    test_util::BigState,
    topics::TopicKey,
};
use victory_wtf::{Timepoint, Timespan};

pub struct TCPNodeSubscriber {}

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// IP address to bind to
    #[arg(short, long, default_value = "0.0.0.0")]
    address: String,

    /// Port to listen on
    #[arg(short, long, default_value = "7000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let args = Args::parse();
    let bind_addr = format!("{}:{}", args.address, args.port);

    let server = TcpSyncServer::new(&bind_addr).await;
    let server_handle = Arc::new(Mutex::new(server));
    let datastore = Datastore::new().handle();

    let topic_filter = TopicKey::empty();

    let sync_config = SyncConfig {
        client_name: "TCP Sync Server".to_string(),
        subscriptions: vec![topic_filter.display_name()],
    };
    datastore
        .lock()
        .unwrap()
        .setup_sync(sync_config, server_handle);

    loop {
        tokio::time::sleep(Duration::from_secs_f32(0.5)).await;
        datastore.lock().unwrap().run_sync();

        // Print value of topic
        let topic = TopicKey::from_str("test");
        let value: Result<BigState, _> = datastore.lock().unwrap().get_struct(&topic);
        match value {
            Ok(state) => println!("Value of topic: {:?}", state),
            Err(e) => {}
        }
    }
}
