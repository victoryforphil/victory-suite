use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use admin::{admin_server::AdminServer, proto::pubsub_admin};
use datastore::{
    primitives::timestamp::{VicDuration, VicInstant},
    topics::TopicKey,
};
pub type MutexType<T> = Arc<tokio::sync::Mutex<T>>;
pub type RwLockType<T> = Arc<tokio::sync::RwLock<T>>;

use log::{debug, info};
use pubsub::{adapters::mock::MockPubSubAdapter, messages::*, server::PubSubServer};

#[tokio::main]
async fn main() {
    env_logger::init();
}
#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_pubsub_server() {
        main();
    }
}
