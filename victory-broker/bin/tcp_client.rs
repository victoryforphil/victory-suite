use std::{collections::HashMap, sync::Arc, time::Duration};

use log::{debug, info};
use pubsub::{
    adapters::{
        tcp::{TCPClientAdapter, TCPClientOptions},
        PubSubAdapter,
    },
    messages::PubSubMessage,
};

pub type MutexType<T> = Arc<tokio::sync::Mutex<T>>;
pub type RwLockType<T> = Arc<tokio::sync::RwLock<T>>;

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut client = TCPClientAdapter::new(TCPClientOptions::from_url("0.0.0.0:7001")).await;

    let register = PubSubMessage::Register();
    let mut map = HashMap::new();
    map.insert(0, vec![register]);
    client.write(map);

    loop {
        let mut map = client.read();
        for (id, messages) in map.iter_mut() {
            for message in messages.iter_mut() {
                info!("Received message: {:?}", message);
            }
        }
        // Sleep 1 second
        tokio::time::sleep(Duration::from_secs_f32(0.25)).await;
    }
}
