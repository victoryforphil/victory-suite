use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{info, warn};

use victory_broker::{adapters::tcp::tcp_server::TcpBrokerServer, broker::Broker, commander::linear::LinearBrokerCommander};
use victory_data_store::{
    database::Datastore,
    datapoints::Datapoint,
    sync::{adapters::tcp::tcp_server::TcpSyncServer, config::SyncConfig},
    test_util::{BigState, BigStateVector},
    topics::TopicKey,
};
use victory_wtf::{Timepoint, Timespan};
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// IP address to bind to
    #[arg(short, long, default_value = "0.0.0.0")]
    address: String,

    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let args = Args::parse();
    let bind_addr = format!("{}:{}", args.address, args.port);


    info!("Broker Test TCP Server // Binding to {}", bind_addr);

    let server: TcpBrokerServer = TcpBrokerServer::new(&bind_addr).await.unwrap();
    let mut broker = Broker::new(LinearBrokerCommander::new());
    broker.add_adapter(Arc::new(Mutex::new(server)));
    loop {
        match broker.tick() {
            Ok(_) => (),
            Err(e) => {
                warn!("Broker // Error: {:?}", e);
            }
        }
        // Sleep for 100ms
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
