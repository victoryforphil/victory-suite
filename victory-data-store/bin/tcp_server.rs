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
    test_util::{BigState, BigStateVector},
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

    let test_topic = TopicKey::from_str("test/server");
    let client_topic = TopicKey::from_str("test/client");
    let mut test_struct = BigStateVector::default();

    loop {
        tokio::time::sleep(Duration::from_secs_f32(1.5)).await;
        datastore.lock().unwrap().run_sync();

        // Print value of topic
        test_struct.x += -1.0;
        test_struct.y += -2.0;
        test_struct.z += -3.0;

        // Write server struct to topic
        datastore
            .lock()
            .unwrap()
            .add_struct(&test_topic, Timepoint::now(), test_struct);

        // Read client struct from topic
        let value: Result<BigStateVector, _> = datastore.lock().unwrap().get_struct(&client_topic);
        match value {
            Ok(state) => println!("Value of client: {:?}", state),
            Err(e) => {}
        }
    }
}
