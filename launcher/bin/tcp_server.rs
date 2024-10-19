use std::sync::Arc;

use admin::admin_server::AdminServer;
use victory_time_rs::{ Timespan};
use pubsub::{
    adapters::tcp::{TCPServerAdapter, TCPServerOptions},
    server::PubSubServer,
};
use tokio::sync::Mutex;

pub type MutexType<T> = Arc<tokio::sync::Mutex<T>>;
pub type RwLockType<T> = Arc<tokio::sync::RwLock<T>>;

#[tokio::main]
async fn main() {
    env_logger::init();
    let server = TCPServerAdapter::new(TCPServerOptions {
        port: 7001,
        address: "0.0.0.0".to_string(),
        update_interval: Timespan::new_hz(50.0),
    });

    let mut pubsub = PubSubServer::new();

    pubsub.add_adapter(Arc::new(Mutex::new(server)));
    let pubsub = Arc::new(tokio::sync::RwLock::new(pubsub));

    AdminServer::start(pubsub.clone()).await.unwrap();

    loop {
        pubsub.write().await.tick();
        // Sleep 1 second
        tokio::time::sleep(std::time::Duration::from_secs_f32(0.25)).await;
    }
}
